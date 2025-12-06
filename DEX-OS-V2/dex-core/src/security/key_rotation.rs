//! Key Rotation for Security Layer 5 - Data Security
//!
//! Automatic key rotation for encryption keys.

use rand::RngCore;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RotationSchedule {
    pub interval_days: u32,
    pub last_rotation: u64,
    pub next_rotation: u64,
}

#[derive(Debug, Clone)]
pub struct KeyRotationManager {
    current_key_version: u32,
    keys: HashMap<u32, [u8; 32]>,
    rotation_schedule: RotationSchedule,
}

impl KeyRotationManager {
    pub fn new(interval_days: u32) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Generate initial key
        let mut initial_key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut initial_key);

        let mut keys = HashMap::new();
        keys.insert(1, initial_key);

        Self {
            current_key_version: 1,
            keys,
            rotation_schedule: RotationSchedule {
                interval_days,
                last_rotation: now,
                next_rotation: now + (interval_days as u64 * 86400),
            },
        }
    }

    /// Get the current active key
    pub fn get_current_key(&self) -> Option<&[u8; 32]> {
        self.keys.get(&self.current_key_version)
    }

    /// Get a specific version of a key (for decrypting old data)
    pub fn get_key_version(&self, version: u32) -> Option<&[u8; 32]> {
        self.keys.get(&version)
    }

    /// Get all key versions
    pub fn get_key_versions(&self) -> Vec<u32> {
        self.keys.keys().cloned().collect()
    }

    /// Rotate the key manually
    pub fn rotate_key(&mut self) -> u32 {
        let mut new_key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut new_key);
        
        self.current_key_version += 1;
        self.keys.insert(self.current_key_version, new_key);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.rotation_schedule.last_rotation = now;
        self.rotation_schedule.next_rotation = now + (self.rotation_schedule.interval_days as u64 * 86400);
        
        self.current_key_version
    }

    /// Check if rotation is needed based on schedule
    pub fn is_rotation_needed(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        now >= self.rotation_schedule.next_rotation
    }
}

impl Default for KeyRotationManager {
    fn default() -> Self {
        Self::new(90) // 90 days default
    }
}
