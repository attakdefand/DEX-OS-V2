use secp256k1::{Secp256k1, Message, SecretKey, PublicKey, ecdsa::Signature};
use secp256k1::rand::rngs::OsRng;
use sha2::{Sha256, Digest};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Key generation error")]
    KeyGenerationError,
    #[error("Signing error")]
    SigningError,
    #[error("Verification error")]
    VerificationError,
    #[error("Invalid key")]
    InvalidKey,
    #[error("Invalid signature")]
    InvalidSignature,
}

pub struct EcdsaManager {
    secp: Secp256k1<secp256k1::All>,
}

impl EcdsaManager {
    pub fn new() -> Self {
        Self {
            secp: Secp256k1::new(),
        }
    }

    /// Generate a new random keypair
    pub fn generate_keypair(&self) -> Result<(Vec<u8>, Vec<u8>), CryptoError> {
        let (secret_key, public_key) = self.secp.generate_keypair(&mut OsRng);
        Ok((secret_key.secret_bytes().to_vec(), public_key.serialize().to_vec()))
    }

    /// Sign a message using a private key
    pub fn sign(&self, message: &[u8], private_key_bytes: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let secret_key = SecretKey::from_slice(private_key_bytes)
            .map_err(|_| CryptoError::InvalidKey)?;
        
        let mut hasher = Sha256::new();
        hasher.update(message);
        let result = hasher.finalize();
        
        let message_hash = Message::from_digest_slice(&result)
            .map_err(|_| CryptoError::SigningError)?;
            
        let signature = self.secp.sign_ecdsa(&message_hash, &secret_key);
        Ok(signature.serialize_compact().to_vec())
    }

    /// Verify a signature using a public key
    pub fn verify(&self, message: &[u8], signature_bytes: &[u8], public_key_bytes: &[u8]) -> Result<bool, CryptoError> {
        let public_key = PublicKey::from_slice(public_key_bytes)
            .map_err(|_| CryptoError::InvalidKey)?;
            
        let signature = Signature::from_compact(signature_bytes)
            .map_err(|_| CryptoError::InvalidSignature)?;
            
        let mut hasher = Sha256::new();
        hasher.update(message);
        let result = hasher.finalize();
        
        let message_hash = Message::from_digest_slice(&result)
            .map_err(|_| CryptoError::VerificationError)?;
            
        Ok(self.secp.verify_ecdsa(&message_hash, &signature, &public_key).is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let manager = EcdsaManager::new();
        let result = manager.generate_keypair();
        assert!(result.is_ok());
        let (sk, pk) = result.unwrap();
        assert_eq!(sk.len(), 32);
        assert_eq!(pk.len(), 33); // Compressed public key
    }

    #[test]
    fn test_sign_and_verify() {
        let manager = EcdsaManager::new();
        let (sk, pk) = manager.generate_keypair().unwrap();
        let message = b"Hello, Blockchain!";
        
        let signature = manager.sign(message, &sk).unwrap();
        assert_eq!(signature.len(), 64); // Compact signature
        
        let valid = manager.verify(message, &signature, &pk).unwrap();
        assert!(valid);
    }

    #[test]
    fn test_verify_invalid_signature() {
        let manager = EcdsaManager::new();
        let (sk, pk) = manager.generate_keypair().unwrap();
        let message = b"Hello, Blockchain!";
        
        let mut signature = manager.sign(message, &sk).unwrap();
        // Tamper with signature
        signature[0] ^= 0xFF;
        
        // It might return Ok(false) or Err(InvalidSignature) depending on how broken it is.
        // In this case, if it's not a valid compact signature, it returns Err.
        // If it is valid format but wrong signature, it returns Ok(false).
        // Flipping a bit usually makes it invalid or fail verification.
        
        let result = manager.verify(message, &signature, &pk);
        if let Ok(valid) = result {
            assert!(!valid);
        } else {
            assert!(result.is_err());
        }
    }
    
    #[test]
    fn test_verify_wrong_message() {
        let manager = EcdsaManager::new();
        let (sk, pk) = manager.generate_keypair().unwrap();
        let message = b"Hello, Blockchain!";
        let wrong_message = b"Hello, World!";
        
        let signature = manager.sign(message, &sk).unwrap();
        
        let valid = manager.verify(wrong_message, &signature, &pk).unwrap();
        assert!(!valid);
    }
}
