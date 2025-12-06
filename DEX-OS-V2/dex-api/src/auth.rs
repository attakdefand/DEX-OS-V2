//! JWT authentication helpers for Warp filters.

use ethers_core::{
    types::{Address, Signature},
    utils::hash_message,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    str::FromStr,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use thiserror::Error;
use warp::reject::Reject;

/// Shared authentication manager that validates bearer tokens.
#[derive(Clone)]
pub struct AuthManager {
    decoding_key: Arc<DecodingKey>,
    encoding_key: Arc<EncodingKey>,
    validation: Validation,
    issuer: Arc<String>,
    allowed_audiences: Vec<String>,
    require_audience: bool,
}

impl AuthManager {
    pub fn new(
        secret: &SecretString,
        issuer: impl Into<String>,
        allowed_audiences: Vec<String>,
        require_audience: bool,
    ) -> Self {
        let decoding_key = DecodingKey::from_secret(secret.expose_secret().as_bytes());
        let encoding_key = EncodingKey::from_secret(secret.expose_secret().as_bytes());
        let mut validation = Validation::default();
        validation.validate_exp = true;
        Self {
            decoding_key: Arc::new(decoding_key),
            encoding_key: Arc::new(encoding_key),
            validation,
            issuer: Arc::new(issuer.into()),
            allowed_audiences,
            require_audience,
        }
    }

    pub fn verify_bearer(&self, header_value: &str) -> Result<Claims, AuthError> {
        let token = header_value
            .strip_prefix("Bearer ")
            .ok_or(AuthError::MissingBearer)?;

        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map_err(|err| AuthError::InvalidToken(err.to_string()))?;
        let claims = token_data.claims;

        // Enforce expiration explicitly to avoid type/codec edge cases
        let now = current_unix_timestamp().map_err(|_| AuthError::TimeSource)? as usize;
        if claims.exp <= now {
            return Err(AuthError::InvalidToken("expired".into()));
        }

        // Enforce issuer
        match &claims.iss {
            Some(iss) if iss == &*self.issuer => {}
            _ => return Err(AuthError::InvalidToken("invalid issuer".into())),
        }

        // Optionally enforce audience
        if self.require_audience {
            match &claims.aud {
                Some(aud) => {
                    if !self.allowed_audiences.is_empty()
                        && !self.allowed_audiences.iter().any(|a| a == aud)
                    {
                        return Err(AuthError::InvalidToken("invalid audience".into()));
                    }
                }
                None => return Err(AuthError::InvalidToken("missing audience".into())),
            }
        }

        Ok(claims)
    }

    pub fn issue_token(
        &self,
        subject: impl Into<String>,
        ttl: Duration,
        audience: Option<String>,
    ) -> Result<IssuedToken, AuthError> {
        let ttl = if ttl.is_zero() {
            Duration::from_secs(60)
        } else {
            ttl
        };
        let now = current_unix_timestamp().map_err(|_| AuthError::TimeSource)?;
        let expires_at = now + ttl.as_secs();
        let claims = Claims {
            sub: subject.into(),
            exp: expires_at as usize,
            aud: audience,
            iss: Some((*self.issuer).clone()),
            iat: Some(now as usize),
        };
        let token = encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|err| AuthError::TokenIssuance(err.to_string()))?;
        Ok(IssuedToken { token, expires_at })
    }
}

/// JWT claims we expect from clients.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    #[serde(default)]
    pub aud: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iat: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct IssuedToken {
    pub token: String,
    pub expires_at: u64,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("authorization header must use the Bearer scheme")]
    MissingBearer,
    #[error("invalid token: {0}")]
    InvalidToken(String),
    #[error("failed to issue token: {0}")]
    TokenIssuance(String),
    #[error("wallet signature invalid: {0}")]
    InvalidSignature(String),
    #[error("invalid wallet address")]
    InvalidAddress,
    #[error("wallet signature does not match address")]
    SignatureMismatch,
    #[error("system clock unavailable")]
    TimeSource,
}

#[derive(Debug)]
pub struct AuthRejection(pub AuthError);

impl Reject for AuthRejection {}

impl fmt::Display for AuthRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "authentication failed: {}", self.0)
    }
}

pub fn clamp_ttl(requested: Option<u64>, default_ttl: u64, max_ttl: u64) -> Duration {
    let default = default_ttl.max(60);
    let max_allowed = max_ttl.max(default);
    let requested = requested.unwrap_or(default).max(60).min(max_allowed);
    Duration::from_secs(requested)
}

pub fn verify_wallet_signature(
    address: &str,
    message: &str,
    signature: &str,
) -> Result<(), AuthError> {
    let normalized_address = normalize_address(address)?;
    let target = Address::from_str(&normalized_address).map_err(|_| AuthError::InvalidAddress)?;
    let signature = signature.trim_start_matches("0x");
    let sig = Signature::from_str(signature)
        .map_err(|err| AuthError::InvalidSignature(err.to_string()))?;
    let hash = hash_message(message);
    let recovered = sig
        .recover(hash)
        .map_err(|err| AuthError::InvalidSignature(err.to_string()))?;
    if recovered != target {
        return Err(AuthError::SignatureMismatch);
    }
    Ok(())
}

pub fn normalize_address(input: &str) -> Result<String, AuthError> {
    let trimmed = input.trim().to_lowercase();
    if trimmed.len() != 42
        || !trimmed.starts_with("0x")
        || !trimmed.chars().skip(2).all(|c| c.is_ascii_hexdigit())
    {
        return Err(AuthError::InvalidAddress);
    }
    Ok(trimmed)
}

fn current_unix_timestamp() -> Result<u64, std::time::SystemTimeError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
}
