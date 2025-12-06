//! Whitelist/Blacklist Module for Security Layer 4
//!
//! Implements IP and User whitelisting/blacklisting.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::net::IpAddr;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ListError {
    #[error("Item already exists in list")]
    AlreadyExists,
    #[error("Item not found in list")]
    NotFound,
    #[error("Conflict: Item exists in opposing list")]
    Conflict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ListType {
    Whitelist,
    Blacklist,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityType {
    IP(String),
    User(String),
    Country(String),
    Token(String),
}

#[derive(Debug, Clone)]
pub struct WhitelistBlacklistManager {
    whitelists: Arc<RwLock<HashMap<String, HashSet<String>>>>, // type -> set of values
    blacklists: Arc<RwLock<HashMap<String, HashSet<String>>>>, // type -> set of values
    enabled: Arc<RwLock<bool>>,
}

impl WhitelistBlacklistManager {
    pub fn new() -> Self {
        Self {
            whitelists: Arc::new(RwLock::new(HashMap::new())),
            blacklists: Arc::new(RwLock::new(HashMap::new())),
            enabled: Arc::new(RwLock::new(true)),
        }
    }

    /// Add item to whitelist
    pub fn add_to_whitelist(&self, category: &str, value: &str) -> Result<(), ListError> {
        let mut blacklists = self.blacklists.write().unwrap();
        if let Some(list) = blacklists.get(category) {
            if list.contains(value) {
                return Err(ListError::Conflict);
            }
        }
        drop(blacklists);

        let mut whitelists = self.whitelists.write().unwrap();
        let list = whitelists.entry(category.to_string()).or_insert_with(HashSet::new);
        
        if list.contains(value) {
            return Err(ListError::AlreadyExists);
        }
        
        list.insert(value.to_string());
        Ok(())
    }

    /// Add item to blacklist
    pub fn add_to_blacklist(&self, category: &str, value: &str) -> Result<(), ListError> {
        let mut whitelists = self.whitelists.write().unwrap();
        if let Some(list) = whitelists.get(category) {
            if list.contains(value) {
                return Err(ListError::Conflict);
            }
        }
        drop(whitelists);

        let mut blacklists = self.blacklists.write().unwrap();
        let list = blacklists.entry(category.to_string()).or_insert_with(HashSet::new);
        
        if list.contains(value) {
            return Err(ListError::AlreadyExists);
        }
        
        list.insert(value.to_string());
        Ok(())
    }

    /// Remove item from whitelist
    pub fn remove_from_whitelist(&self, category: &str, value: &str) -> Result<(), ListError> {
        let mut whitelists = self.whitelists.write().unwrap();
        if let Some(list) = whitelists.get_mut(category) {
            if list.remove(value) {
                return Ok(());
            }
        }
        Err(ListError::NotFound)
    }

    /// Remove item from blacklist
    pub fn remove_from_blacklist(&self, category: &str, value: &str) -> Result<(), ListError> {
        let mut blacklists = self.blacklists.write().unwrap();
        if let Some(list) = blacklists.get_mut(category) {
            if list.remove(value) {
                return Ok(());
            }
        }
        Err(ListError::NotFound)
    }

    /// Check if item is allowed
    /// Returns true if allowed, false if blocked
    /// Logic:
    /// 1. If in blacklist -> Blocked
    /// 2. If whitelist exists and not empty:
    ///    - If in whitelist -> Allowed
    ///    - If not in whitelist -> Blocked
    /// 3. If whitelist empty -> Allowed (default allow unless blacklisted)
    pub fn is_allowed(&self, category: &str, value: &str) -> bool {
        if !*self.enabled.read().unwrap() {
            return true;
        }

        // Check blacklist first
        let blacklists = self.blacklists.read().unwrap();
        if let Some(list) = blacklists.get(category) {
            if list.contains(value) {
                return false;
            }
        }
        drop(blacklists);

        // Check whitelist
        let whitelists = self.whitelists.read().unwrap();
        if let Some(list) = whitelists.get(category) {
            if !list.is_empty() {
                return list.contains(value);
            }
        }

        // Default allow
        true
    }

    /// Helper for IP checking
    pub fn is_ip_allowed(&self, ip: &str) -> bool {
        self.is_allowed("ip", ip)
    }

    /// Helper for User checking
    pub fn is_user_allowed(&self, user_id: &str) -> bool {
        self.is_allowed("user", user_id)
    }
}

impl Default for WhitelistBlacklistManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blacklist_blocks() {
        let manager = WhitelistBlacklistManager::new();
        manager.add_to_blacklist("ip", "192.168.1.1").unwrap();
        
        assert!(!manager.is_ip_allowed("192.168.1.1"));
        assert!(manager.is_ip_allowed("192.168.1.2"));
    }

    #[test]
    fn test_whitelist_enforces() {
        let manager = WhitelistBlacklistManager::new();
        manager.add_to_whitelist("user", "admin").unwrap();
        
        assert!(manager.is_user_allowed("admin"));
        assert!(!manager.is_user_allowed("guest")); // Not in whitelist
    }

    #[test]
    fn test_conflict_prevention() {
        let manager = WhitelistBlacklistManager::new();
        manager.add_to_whitelist("ip", "10.0.0.1").unwrap();
        
        let result = manager.add_to_blacklist("ip", "10.0.0.1");
        assert_eq!(result, Err(ListError::Conflict));
    }
}
