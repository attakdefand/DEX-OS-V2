//! Token Validation Module for Security Layer 6 - Identity and Access Management
//!
//! Implements JWT-like token management using Ed25519 signatures.
//! Features:
//! - JWT Creation (Signing)
//! - JWT Validation (Verification)
//! - Claims Management
//! - Expiration Checks

use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::time::{SystemTime, UNIX_EPOCH};
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey, Signature};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::rngs::OsRng;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum TokenValidationError {
    #[error("Token has expired")]
    Expired,
    #[error("Token signature invalid")]
    InvalidSignature,
    #[error("Token format invalid")]
    InvalidFormat,
    #[error("Token issuer invalid")]
    InvalidIssuer,
    #[error("Token audience invalid")]
    InvalidAudience,
    #[error("Key error: {0}")]
    KeyError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenHeader {
    pub alg: String,
    pub typ: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,       // Subject (User ID)
    pub iss: String,       // Issuer
    pub aud: String,       // Audience
    pub exp: u64,          // Expiration
    pub iat: u64,          // Issued At
    pub roles: Vec<String>, // User Roles
}

#[derive(Debug, Clone)]
pub struct TokenValidator {
    issuer: String,
    audience: String,
    verifying_key: VerifyingKey,
}

#[derive(Debug)]
pub struct TokenManager {
    validator: TokenValidator,
    signing_key: SigningKey,
}

impl TokenManager {
    pub fn new(issuer: impl Into<String>, audience: impl Into<String>) -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();

        Self {
            validator: TokenValidator {
                issuer: issuer.into(),
                audience: audience.into(),
                verifying_key,
            },
            signing_key,
        }
    }

    /// Create a new token
    pub fn create_token(&self, user_id: &str, roles: Vec<String>, ttl_seconds: u64) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let claims = TokenClaims {
            sub: user_id.to_string(),
            iss: self.validator.issuer.clone(),
            aud: self.validator.audience.clone(),
            exp: now + ttl_seconds,
            iat: now,
            roles,
        };

        let header = TokenHeader {
            alg: "EdDSA".to_string(),
            typ: "JWT".to_string(),
        };

        let header_json = serde_json::to_string(&header).unwrap();
        let claims_json = serde_json::to_string(&claims).unwrap();

        let header_b64 = URL_SAFE_NO_PAD.encode(header_json);
        let claims_b64 = URL_SAFE_NO_PAD.encode(claims_json);

        let message = format!("{}.{}", header_b64, claims_b64);
        let signature = self.signing_key.sign(message.as_bytes());
        let signature_b64 = URL_SAFE_NO_PAD.encode(signature.to_bytes());

        format!("{}.{}", message, signature_b64)
    }

    pub fn get_validator(&self) -> TokenValidator {
        self.validator.clone()
    }
}

impl TokenValidator {
    pub fn new(issuer: impl Into<String>, audience: impl Into<String>, public_key_bytes: &[u8]) -> Result<Self, TokenValidationError> {
        let verifying_key = VerifyingKey::from_bytes(public_key_bytes.try_into().map_err(|_| TokenValidationError::KeyError("Invalid key length".to_string()))?)
            .map_err(|e| TokenValidationError::KeyError(e.to_string()))?;

        Ok(Self {
            issuer: issuer.into(),
            audience: audience.into(),
            verifying_key,
        })
    }

    /// Validate a JWT token
    pub fn validate_token(&self, token: &str) -> Result<TokenClaims, TokenValidationError> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(TokenValidationError::InvalidFormat);
        }

        let header_b64 = parts[0];
        let claims_b64 = parts[1];
        let signature_b64 = parts[2];

        // 1. Verify Signature
        let message = format!("{}.{}", header_b64, claims_b64);
        let signature_bytes = URL_SAFE_NO_PAD.decode(signature_b64)
            .map_err(|_| TokenValidationError::InvalidFormat)?;
        
        let signature = Signature::from_bytes(signature_bytes.as_slice().try_into().map_err(|_| TokenValidationError::InvalidSignature)?);

        self.verifying_key.verify(message.as_bytes(), &signature)
            .map_err(|_| TokenValidationError::InvalidSignature)?;

        // 2. Decode Claims
        let claims_json = URL_SAFE_NO_PAD.decode(claims_b64)
            .map_err(|_| TokenValidationError::InvalidFormat)?;
        let claims: TokenClaims = serde_json::from_slice(&claims_json)
            .map_err(|_| TokenValidationError::InvalidFormat)?;

        // 3. Validate Claims
        self.validate_claims(&claims)?;

        Ok(claims)
    }

    pub fn validate_claims(&self, claims: &TokenClaims) -> Result<(), TokenValidationError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if claims.exp < now {
            return Err(TokenValidationError::Expired);
        }

        if claims.iss != self.issuer {
            return Err(TokenValidationError::InvalidIssuer);
        }

        if claims.aud != self.audience {
            return Err(TokenValidationError::InvalidAudience);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation_and_validation() {
        let manager = TokenManager::new("dex-os", "dex-client");
        let token = manager.create_token("user1", vec!["admin".to_string()], 3600);
        
        let validator = manager.get_validator();
        let claims = validator.validate_token(&token).unwrap();
        
        assert_eq!(claims.sub, "user1");
        assert_eq!(claims.iss, "dex-os");
        assert!(claims.roles.contains(&"admin".to_string()));
    }

    #[test]
    fn test_token_expiration() {
        let manager = TokenManager::new("dex-os", "dex-client");
        // Create token that expires in 0 seconds (effectively immediately or slightly in past if we could)
        // Since we use u64 seconds, let's use 0 TTL.
        let token = manager.create_token("user1", vec![], 0);
        
        // Wait 1 second
        std::thread::sleep(std::time::Duration::from_secs(1));
        
        let validator = manager.get_validator();
        let result = validator.validate_token(&token);
        assert_eq!(result.err(), Some(TokenValidationError::Expired));
    }

    #[test]
    fn test_token_tampering() {
        let manager = TokenManager::new("dex-os", "dex-client");
        let token = manager.create_token("user1", vec![], 3600);
        
        let mut parts: Vec<&str> = token.split('.').collect();
        // Tamper with payload (middle part)
        // We'll just replace it with some other valid base64 but different content
        let tampered_payload = URL_SAFE_NO_PAD.encode(r#"{"sub":"hacker"}"#);
        parts[1] = &tampered_payload;
        
        let tampered_token = parts.join(".");
        
        let validator = manager.get_validator();
        let result = validator.validate_token(&tampered_token);
        assert_eq!(result.err(), Some(TokenValidationError::InvalidSignature));
    }
}
