//! Configuration loading for the DEX API service.
//!
//! Centralizes environment parsing and keeps sensitive values wrapped in
//! secrecy primitives.

use dotenvy::dotenv;
use secrecy::SecretString;
use std::{collections::HashMap, env, num::ParseIntError};
use thiserror::Error;

/// Runtime configuration for the API service.
#[derive(Clone)]
pub struct Config {
    pub database_url: SecretString,
    pub jwt_secret: SecretString,
    pub jwt_issuer: String,
    pub jwt_default_ttl_seconds: u64,
    pub jwt_max_ttl_seconds: u64,
    pub wallet_challenge_ttl_seconds: u64,
    pub trader_secrets: HashMap<String, SecretString>,
    pub server_port: u16,
    // Security/CORS
    pub cors_allowed_origins: Vec<String>,
    pub jwt_require_audience: bool,
    pub jwt_allowed_audiences: Vec<String>,
    /// DAO members that will be pre-enrolled with voting power and optional emergency privileges.
    pub dao_members: Vec<DaoMemberConfig>,
}

impl Config {
    /// Load configuration from environment variables, honoring values supplied
    /// via a `.env` file when present.
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();

        let database_url =
            env::var("DATABASE_URL").map_err(|_| ConfigError::Missing("DATABASE_URL"))?;
        let jwt_secret = env::var("JWT_SECRET").map_err(|_| ConfigError::Missing("JWT_SECRET"))?;

        let server_port = parse_server_port(env::var("SERVER_PORT").ok())?;
        let jwt_issuer = env::var("JWT_ISSUER").unwrap_or_else(|_| "dex-os-api".to_string());
        let jwt_default_ttl_seconds = parse_u64("JWT_TTL_SECONDS", 900)?;
        let jwt_max_ttl_seconds = parse_u64("JWT_MAX_TTL_SECONDS", 3600)?;
        let wallet_challenge_ttl_seconds = parse_u64("WALLET_CHALLENGE_TTL_SECONDS", 300)?;
        let trader_secrets = parse_trader_secrets(env::var("TRADER_SECRETS").ok())?;
        let cors_allowed_origins = parse_csv(env::var("CORS_ALLOWED_ORIGINS").ok());
        let jwt_require_audience = parse_bool("JWT_REQUIRE_AUDIENCE", false);
        let jwt_allowed_audiences = parse_csv(env::var("JWT_ALLOWED_AUDIENCES").ok());
        let dao_members = parse_dao_members(env::var("DAO_MEMBERS").ok())?;

        Ok(Self {
            database_url: SecretString::from(database_url),
            jwt_secret: SecretString::from(jwt_secret),
            jwt_issuer,
            jwt_default_ttl_seconds: jwt_default_ttl_seconds.max(60),
            jwt_max_ttl_seconds: jwt_max_ttl_seconds.max(jwt_default_ttl_seconds),
            wallet_challenge_ttl_seconds: wallet_challenge_ttl_seconds.max(60),
            trader_secrets,
            server_port,
            cors_allowed_origins,
            jwt_require_audience,
            jwt_allowed_audiences,
            dao_members,
        })
    }
}

fn parse_server_port(raw: Option<String>) -> Result<u16, ConfigError> {
    match raw {
        Some(value) => value
            .parse::<u16>()
            .map_err(|err| ConfigError::InvalidPort { value, err }),
        None => Ok(3030),
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required environment variable {0}")]
    Missing(&'static str),
    #[error("invalid SERVER_PORT value {value}: {err}")]
    InvalidPort { value: String, err: ParseIntError },
    #[error("invalid numeric value for {var}: {err}")]
    InvalidNumber {
        var: &'static str,
        err: ParseIntError,
    },
    #[error("invalid TRADER_SECRETS entry '{entry}', expected trader:secret")]
    InvalidTraderSecret { entry: String },
    #[error("invalid DAO_MEMBERS entry '{entry}', expected trader|power|council_flag")]
    InvalidDaoMember { entry: String },
}

fn parse_u64(var: &'static str, default: u64) -> Result<u64, ConfigError> {
    match env::var(var) {
        Ok(value) => value
            .parse::<u64>()
            .map_err(|err| ConfigError::InvalidNumber { var, err }),
        Err(_) => Ok(default),
    }
}

fn parse_trader_secrets(raw: Option<String>) -> Result<HashMap<String, SecretString>, ConfigError> {
    let mut map = HashMap::new();
    if let Some(raw) = raw {
        for entry in raw.split(',') {
            if entry.trim().is_empty() {
                continue;
            }
            let mut parts = entry.splitn(2, ':').map(|p| p.trim().to_string());
            let trader = parts.next().unwrap_or_default();
            let secret = parts
                .next()
                .ok_or_else(|| ConfigError::InvalidTraderSecret {
                    entry: entry.to_string(),
                })?;
            if trader.is_empty() || secret.is_empty() {
                return Err(ConfigError::InvalidTraderSecret {
                    entry: entry.to_string(),
                });
            }
            map.insert(trader, SecretString::from(secret));
        }
    }
    Ok(map)
}

fn parse_bool(var: &'static str, default: bool) -> bool {
    match env::var(var) {
        Ok(v) => matches!(v.as_str(), "1" | "true" | "TRUE" | "True"),
        Err(_) => default,
    }
}

fn parse_csv(raw: Option<String>) -> Vec<String> {
    match raw {
        Some(s) => s
            .split(',')
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .collect(),
        None => Vec::new(),
    }
}

fn parse_dao_members(raw: Option<String>) -> Result<Vec<DaoMemberConfig>, ConfigError> {
    let mut members = Vec::new();
    if let Some(raw) = raw {
        for entry in raw.split(',') {
            let trimmed = entry.trim();
            if trimmed.is_empty() {
                continue;
            }
            let mut parts = trimmed.split('|').map(str::trim);
            let trader_id = parts.next().unwrap_or("");
            let power = parts.next().unwrap_or("");
            let council_flag = parts.next().unwrap_or("");

            if trader_id.is_empty() || power.is_empty() || council_flag.is_empty() {
                return Err(ConfigError::InvalidDaoMember {
                    entry: entry.to_string(),
                });
            }

            let voting_power = power.parse::<u64>().map_err(|_| ConfigError::InvalidDaoMember {
                entry: entry.to_string(),
            })?;
            if voting_power == 0 {
                return Err(ConfigError::InvalidDaoMember {
                    entry: entry.to_string(),
                });
            }

            let is_council_member = matches!(
                council_flag.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes"
            );

            members.push(DaoMemberConfig {
                trader_id: trader_id.to_string(),
                voting_power,
                is_council_member,
            });
        }
    }
    Ok(members)
}

/// Pre-configured DAO membership supplied through environment variables.
#[derive(Debug, Clone)]
pub struct DaoMemberConfig {
    pub trader_id: String,
    pub voting_power: u64,
    pub is_council_member: bool,
}
