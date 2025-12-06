//! Vote Escrow NFT system for advanced governance
//!
//! This module implements the ve(3,3) governance model with dual veNFT system
//! similar to what's found in Blackhole DEX and other advanced DeFi protocols.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

/// Represents a Vote Escrow NFT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VeNFT {
    pub id: u64,
    pub owner: String,
    pub token_amount: u128,
    pub lock_start: u64,
    pub lock_end: u64,
    pub nft_type: VeNFTType,
}

/// Types of Vote Escrow NFTs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VeNFTType {
    Singularity,  // Standard veNFT with decaying voting power
    Supermassive, // Permanent veNFT with boosted benefits
}

impl VeNFT {
    /// Calculate voting power based on lock duration and type
    pub fn voting_power(&self) -> f64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        match self.nft_type {
            VeNFTType::Singularity => {
                // Voting power decays linearly over time
                if now >= self.lock_end {
                    0.0
                } else {
                    let lock_duration = (self.lock_end - self.lock_start) as f64;
                    let time_left = (self.lock_end - now) as f64;
                    (self.token_amount as f64) * (time_left / lock_duration)
                }
            },
            VeNFTType::Supermassive => {
                // Permanent voting power with 10% boost
                (self.token_amount as f64) * 1.1
            }
        }
    }
    
    /// Get rebase boost multiplier
    pub fn rebase_boost(&self) -> f64 {
        match self.nft_type {
            VeNFTType::Supermassive => 1.1, // 10% boost
            VeNFTType::Singularity => 1.0,  // No boost
        }
    }
    
    /// Check if the veNFT is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        match self.nft_type {
            VeNFTType::Singularity => now >= self.lock_end,
            VeNFTType::Supermassive => false, // Supermassive never expires
        }
    }
}

/// Registry for managing Vote Escrow NFTs
#[derive(Debug)]
pub struct VeNFTRegistry {
    pub nfts: HashMap<u64, VeNFT>,
    next_id: u64,
}

impl VeNFTRegistry {
    /// Create a new VeNFT registry
    pub fn new() -> Self {
        Self {
            nfts: HashMap::new(),
            next_id: 1,
        }
    }
    
    /// Create a new Vote Escrow NFT
    pub fn create_venft(
        &mut self, 
        owner: String, 
        amount: u128, 
        lock_duration: u64, 
        nft_type: VeNFTType
    ) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        let venft = VeNFT {
            id: self.next_id,
            owner,
            token_amount: amount,
            lock_start: now,
            lock_end: now + lock_duration,
            nft_type,
        };
        
        let id = self.next_id;
        self.nfts.insert(id, venft);
        self.next_id += 1;
        id
    }
    
    /// Get voting power for a specific veNFT
    pub fn get_voting_power(&self, id: u64) -> Option<f64> {
        self.nfts.get(&id).map(|nft| nft.voting_power())
    }
    
    /// Get rebase boost for a specific veNFT
    pub fn get_rebase_boost(&self, id: u64) -> Option<f64> {
        self.nfts.get(&id).map(|nft| nft.rebase_boost())
    }
    
    /// Get all veNFTs owned by a specific address
    pub fn get_owner_venfts(&self, owner: &str) -> Vec<&VeNFT> {
        self.nfts
            .values()
            .filter(|nft| nft.owner == owner)
            .collect()
    }
    
    /// Get total voting power for an owner
    pub fn get_owner_total_voting_power(&self, owner: &str) -> f64 {
        self.nfts
            .values()
            .filter(|nft| nft.owner == owner)
            .map(|nft| nft.voting_power())
            .sum()
    }
    
    /// Burn a Supermassive veNFT (permanently lock tokens)
    pub fn burn_supermassive(&mut self, id: u64) -> Result<(), String> {
        if let Some(venft) = self.nfts.get(&id) {
            if matches!(venft.nft_type, VeNFTType::Supermassive) {
                // In a real implementation, this would burn the actual tokens
                // For now, we just remove it from the registry
                self.nfts.remove(&id);
                Ok(())
            } else {
                Err("Only Supermassive veNFTs can be burned".to_string())
            }
        } else {
            Err("veNFT not found".to_string())
        }
    }
    
    /// Extend lock duration for a Singularity veNFT
    pub fn extend_lock(&mut self, id: u64, additional_duration: u64) -> Result<(), String> {
        if let Some(venft) = self.nfts.get_mut(&id) {
            if matches!(venft.nft_type, VeNFTType::Singularity) {
                venft.lock_end += additional_duration;
                Ok(())
            } else {
                Err("Only Singularity veNFTs can have their lock extended".to_string())
            }
        } else {
            Err("veNFT not found".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_venft() {
        let mut registry = VeNFTRegistry::new();
        let id = registry.create_venft(
            "user1".to_string(),
            1000,
            365 * 24 * 60 * 60, // 1 year
            VeNFTType::Singularity
        );
        
        assert_eq!(id, 1);
        assert!(registry.nfts.contains_key(&1));
    }
    
    #[test]
    fn test_voting_power_decay() {
        let mut registry = VeNFTRegistry::new();
        let id = registry.create_venft(
            "user1".to_string(),
            1000,
            365 * 24 * 60 * 60, // 1 year
            VeNFTType::Singularity
        );
        
        let voting_power = registry.get_voting_power(id).unwrap();
        // Initially should be close to full voting power
        assert!(voting_power > 999.0);
        assert!(voting_power <= 1000.0);
    }
    
    #[test]
    fn test_supermassive_boost() {
        let mut registry = VeNFTRegistry::new();
        let id = registry.create_venft(
            "user1".to_string(),
            1000,
            365 * 24 * 60 * 60, // 1 year
            VeNFTType::Supermassive
        );
        
        let voting_power = registry.get_voting_power(id).unwrap();
        // Should have 10% boost
        assert_eq!(voting_power, 1100.0);
        
        let boost = registry.get_rebase_boost(id).unwrap();
        assert_eq!(boost, 1.1);
    }
    
    #[test]
    fn test_owner_voting_power() {
        let mut registry = VeNFTRegistry::new();
        
        // Create multiple veNFTs for the same owner
        registry.create_venft("user1".to_string(), 1000, 365 * 24 * 60 * 60, VeNFTType::Singularity);
        registry.create_venft("user1".to_string(), 500, 365 * 24 * 60 * 60, VeNFTType::Supermassive);
        registry.create_venft("user2".to_string(), 2000, 365 * 24 * 60 * 60, VeNFTType::Singularity);
        
        let user1_power = registry.get_owner_total_voting_power("user1");
        let user2_power = registry.get_owner_total_voting_power("user2");
        
        // user1 should have ~1000 (Singularity) + 550 (Supermassive with boost) = 1550
        assert!(user1_power > 1549.0);
        assert!(user1_power <= 1550.0);
        
        // user2 should have ~2000
        assert!(user2_power > 1999.0);
        assert!(user2_power <= 2000.0);
    }
}