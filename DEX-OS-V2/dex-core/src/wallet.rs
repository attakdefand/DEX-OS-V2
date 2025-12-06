use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use rand::{rngs::OsRng, RngCore};
use secp256k1::{
    ecdsa::{RecoverableSignature, RecoveryId},
    Message, PublicKey, Secp256k1, SecretKey,
};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use thiserror::Error;

/// Errors surfaced by wallet signing or session management.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum WalletError {
    #[error("private key must be 32-byte hex")]
    InvalidPrivateKey,
    #[error("invalid wallet address")]
    InvalidAddress,
    #[error("signature malformed")]
    InvalidSignature,
    #[error("signature does not match address")]
    SignatureMismatch,
    #[error("session not found")]
    SessionNotFound,
    #[error("session token mismatch")]
    SessionMismatch,
    #[error("session expired")]
    SessionExpired,
    #[error("entropy unavailable")]
    EntropyUnavailable,
    #[error("invalid chain ID")]
    InvalidChainId,
}

/// Ethereum-style wallet signer that produces personal_sign compatible signatures.
#[derive(Clone)]
pub struct WalletSigner {
    secret_key: SecretKey,
    address: String,
    chain_id: Option<u64>, // Add chain ID support
}

impl WalletSigner {
    /// Construct a signer from a 0x-prefixed or raw hex private key.
    pub fn from_private_key_hex(private_key: &str) -> Result<Self, WalletError> {
        let key_bytes = decode_hex_32(private_key)?;
        let secret_key =
            SecretKey::from_slice(&key_bytes).map_err(|_| WalletError::InvalidPrivateKey)?;
        let secp = Secp256k1::new();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let address = checksum_address(&public_key)?;
        Ok(Self {
            secret_key,
            address,
            chain_id: None, // Default to no chain ID
        })
    }
    
    /// Construct a signer with a specific chain ID
    pub fn from_private_key_hex_with_chain_id(private_key: &str, chain_id: u64) -> Result<Self, WalletError> {
        let mut signer = Self::from_private_key_hex(private_key)?;
        signer.chain_id = Some(chain_id);
        Ok(signer)
    }
    
    /// Set the chain ID for this signer
    pub fn set_chain_id(&mut self, chain_id: u64) {
        self.chain_id = Some(chain_id);
    }
    
    /// Get the chain ID for this signer
    pub fn chain_id(&self) -> Option<u64> {
        self.chain_id
    }

    /// Return the normalized 0x-prefixed address for this signer.
    pub fn address(&self) -> &str {
        &self.address
    }

    /// Produce a personal_sign compatible signature for the provided message.
    pub fn sign_personal_message(&self, message: &str) -> Result<String, WalletError> {
        let message_hash = message_hash_bytes(message);
        let msg = Message::from_slice(&message_hash).map_err(|_| WalletError::InvalidSignature)?;
        let secp = Secp256k1::new();
        let signature: RecoverableSignature = secp.sign_ecdsa_recoverable(&msg, &self.secret_key);
        let mut bytes = [0u8; 65];
        let (recovery_id, sig_bytes) = signature.serialize_compact();
        bytes[..64].copy_from_slice(&sig_bytes);
        bytes[64] = recovery_id.to_i32() as u8;
        Ok(format!("0x{}", hex::encode(bytes)))
    }
    
    /// Produce a chain-specific signature for the provided message.
    /// This includes the chain ID in the signature to prevent replay attacks across chains.
    pub fn sign_message_with_chain_id(&self, message: &str) -> Result<String, WalletError> {
        let chain_id = self.chain_id.ok_or(WalletError::InvalidChainId)?;
        let chain_specific_message = format!("{}#chainid:{}", message, chain_id);
        self.sign_personal_message(&chain_specific_message)
    }

    /// Verify a personal_sign signature against an address.
    pub fn verify_personal_message(
        address: &str,
        message: &str,
        signature: &str,
    ) -> Result<(), WalletError> {
        let normalized_address = normalize_address(address)?;
        let sig_bytes = decode_hex(signature).ok_or(WalletError::InvalidSignature)?;
        if sig_bytes.len() != 65 {
            return Err(WalletError::InvalidSignature);
        }
        let recovery_id = RecoveryId::from_i32(sig_bytes[64] as i32)
            .map_err(|_| WalletError::InvalidSignature)?;
        let compact: [u8; 64] = sig_bytes[..64]
            .try_into()
            .map_err(|_| WalletError::InvalidSignature)?;
        let signature = RecoverableSignature::from_compact(&compact, recovery_id)
            .map_err(|_| WalletError::InvalidSignature)?;
        let message_hash = message_hash_bytes(message);
        let msg = Message::from_slice(&message_hash).map_err(|_| WalletError::InvalidSignature)?;
        let secp = Secp256k1::new();
        let recovered_key = secp
            .recover_ecdsa(&msg, &signature)
            .map_err(|_| WalletError::InvalidSignature)?;
        let recovered_address = checksum_address(&recovered_key)?;
        if recovered_address != normalized_address {
            return Err(WalletError::SignatureMismatch);
        }
        secp.verify_ecdsa(&msg, &signature.to_standard(), &recovered_key)
            .map_err(|_| WalletError::InvalidSignature)?;
        Ok(())
    }
    
    /// Verify a chain-specific signature against an address and chain ID.
    /// This prevents replay attacks across different chains.
    pub fn verify_message_with_chain_id(
        address: &str,
        message: &str,
        signature: &str,
        chain_id: u64,
    ) -> Result<(), WalletError> {
        let chain_specific_message = format!("{}#chainid:{}", message, chain_id);
        Self::verify_personal_message(address, &chain_specific_message, signature)
    }
}

/// In-memory session token manager keyed by wallet address.
#[derive(Default)]
pub struct SessionManager {
    sessions: HashMap<String, WalletSession>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalletSession {
    pub address: String,
    pub token: String,
    pub expires_at: u64,
    pub chain_id: Option<u64>, // Add chain ID to session
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Issue a short-lived session token for an address. TTL is clamped to a minimum of 60s.
    pub fn issue_session(
        &mut self,
        address: &str,
        ttl_seconds: u64,
    ) -> Result<WalletSession, WalletError> {
        self.issue_session_with_chain_id(address, ttl_seconds, None)
    }
    
    /// Issue a short-lived session token for an address with a specific chain ID.
    pub fn issue_session_with_chain_id(
        &mut self,
        address: &str,
        ttl_seconds: u64,
        chain_id: Option<u64>,
    ) -> Result<WalletSession, WalletError> {
        let normalized = normalize_address(address)?;
        let ttl = ttl_seconds.max(60);
        let now = now_seconds();
        let expires_at = now + ttl;
        let token = random_token()?;
        let session = WalletSession {
            address: normalized.clone(),
            token,
            expires_at,
            chain_id,
        };
        self.sessions.insert(normalized.clone(), session.clone());
        Ok(session)
    }

    /// Validate an existing session token.
    pub fn validate_session(&self, address: &str, token: &str) -> Result<(), WalletError> {
        self.validate_session_with_chain_id(address, token, None)
    }
    
    /// Validate an existing session token with chain ID verification.
    /// If a chain ID is provided, it must match the session's chain ID.
    pub fn validate_session_with_chain_id(&self, address: &str, token: &str, chain_id: Option<u64>) -> Result<(), WalletError> {
        let normalized = normalize_address(address)?;
        let session = self
            .sessions
            .get(&normalized)
            .ok_or(WalletError::SessionNotFound)?;
        if session.token != token {
            return Err(WalletError::SessionMismatch);
        }
        if session.expires_at <= now_seconds() {
            return Err(WalletError::SessionExpired);
        }
        // If a chain ID is provided for validation, it must match the session's chain ID
        if let Some(expected_chain_id) = chain_id {
            if session.chain_id != Some(expected_chain_id) {
                return Err(WalletError::InvalidChainId);
            }
        }
        Ok(())
    }

    #[cfg(test)]
    fn expire_for_test(&mut self, address: &str) {
        if let Some(session) = self.sessions.get_mut(address) {
            session.expires_at = now_seconds().saturating_sub(1);
        }
    }
}

fn decode_hex_32(input: &str) -> Result<[u8; 32], WalletError> {
    let bytes = decode_hex(input).ok_or(WalletError::InvalidPrivateKey)?;
    if bytes.len() != 32 {
        return Err(WalletError::InvalidPrivateKey);
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn decode_hex(input: &str) -> Option<Vec<u8>> {
    let trimmed = input.trim().trim_start_matches("0x");
    hex::decode(trimmed).ok()
}

fn normalize_address(input: &str) -> Result<String, WalletError> {
    let trimmed = input.trim().to_lowercase();
    if trimmed.len() != 42
        || !trimmed.starts_with("0x")
        || !trimmed.chars().skip(2).all(|c| c.is_ascii_hexdigit())
    {
        return Err(WalletError::InvalidAddress);
    }
    Ok(trimmed)
}

fn checksum_address(public_key: &PublicKey) -> Result<String, WalletError> {
    let public_bytes = public_key.serialize_uncompressed();
    let hash = keccak256(&public_bytes[1..]); // skip leading 0x04
    let address = &hash[12..]; // last 20 bytes
    Ok(format!("0x{}", hex::encode(address)))
}

fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    hasher.finalize().into()
}

fn message_hash_bytes(message: &str) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    let prefix = format!("\x19Ethereum Signed Message:\n{}", message.as_bytes().len());
    hasher.update(prefix.as_bytes());
    hasher.update(message.as_bytes());
    let digest = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn random_token() -> Result<String, WalletError> {
    let mut bytes = [0u8; 32];
    OsRng
        .try_fill_bytes(&mut bytes)
        .map_err(|_| WalletError::EntropyUnavailable)?;
    Ok(hex::encode(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_signature() {
        let signer = WalletSigner::from_private_key_hex(
            "0x4c0883a69102937d6231471b5dbb6204fe5129617082796f9b8f0b62f5d7c6c0",
        )
        .expect("valid key");
        assert_eq!(
            signer.address(),
            "0x90f8bf6a479f320ead074411a4b0e7944ea8c9c1"
        );
        let message = "Authenticate me";
        let signature = signer.sign_personal_message(message).expect("sign message");
        WalletSigner::verify_personal_message(signer.address(), message, &signature)
            .expect("verify succeeds");
    }

    #[test]
    fn rejects_wrong_address() {
        let signer = WalletSigner::from_private_key_hex(
            "0x4c0883a69102937d6231471b5dbb6204fe5129617082796f9b8f0b62f5d7c6c0",
        )
        .expect("valid key");
        let signature = signer.sign_personal_message("test").expect("sign message");
        let err = WalletSigner::verify_personal_message(
            "0x0000000000000000000000000000000000000000",
            "test",
            &signature,
        )
        .expect_err("mismatch");
        assert_eq!(err, WalletError::SignatureMismatch);
    }

    #[test]
    fn session_lifecycle() {
        let mut manager = SessionManager::new();
        let address = "0x90f8bf6a479f320ead074411a4b0e7944ea8c9c1";
        let issued = manager
            .issue_session(address, 120)
            .expect("session created");
        manager
            .validate_session(address, &issued.token)
            .expect("session valid");

        let err = manager
            .validate_session(address, "deadbeef")
            .expect_err("token mismatch");
        assert_eq!(err, WalletError::SessionMismatch);

        manager.expire_for_test(address);
        let err = manager
            .validate_session(address, &issued.token)
            .expect_err("expired");
        assert_eq!(err, WalletError::SessionExpired);
    }
}
