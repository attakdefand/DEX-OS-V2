use rand::{distributions::Alphanumeric, Rng};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ChallengeStore {
    ttl: Duration,
    inner: Arc<RwLock<HashMap<String, ChallengeEntry>>>,
}

struct ChallengeEntry {
    message: String,
    expires_at: Instant,
}

pub struct IssuedChallenge {
    pub challenge: String,
    pub expires_at: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum ChallengeError {
    #[error("no challenge registered for address")]
    Missing,
    #[error("challenge expired")]
    Expired,
}

impl ChallengeStore {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            ttl: Duration::from_secs(ttl_seconds.max(60)),
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn issue(&self, address: &str) -> IssuedChallenge {
        let nonce: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(24)
            .map(char::from)
            .collect();
        let message = format!(
            "Sign in to DEX-OS\nAddress: {}\nNonce: {}\nIssued At: {}",
            address,
            nonce,
            current_unix_timestamp().unwrap_or(0)
        );
        let expires_at = Instant::now() + self.ttl;
        let expires_epoch = current_unix_timestamp().unwrap_or(0) + self.ttl.as_secs();
        let mut guard = self.inner.write().await;
        guard.insert(
            address.to_string(),
            ChallengeEntry {
                message: message.clone(),
                expires_at,
            },
        );
        drop(guard);
        IssuedChallenge {
            challenge: message,
            expires_at: expires_epoch,
        }
    }

    pub async fn take(&self, address: &str) -> Result<String, ChallengeError> {
        let mut guard = self.inner.write().await;
        let entry = guard.remove(address).ok_or(ChallengeError::Missing)?;
        if Instant::now() > entry.expires_at {
            return Err(ChallengeError::Expired);
        }
        Ok(entry.message)
    }
}

fn current_unix_timestamp() -> Result<u64, std::time::SystemTimeError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
}
