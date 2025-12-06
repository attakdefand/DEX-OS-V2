//! Reward distribution implementation using Priority Queue for the DEX-OS core engine
//!
//! This module implements the Priority 2 feature from DEX-OS-V1.csv:
//! "Core Trading,Oracle,Oracle,Priority Queue,Reward Distribution,Medium"
//!
//! It provides functionality for distributing rewards to oracle providers
//! using a priority queue to ensure fair and efficient distribution.

use crate::types::{Quantity, TokenId, TraderId};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use thiserror::Error;

/// Represents a reward claim that can be processed
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewardClaim {
    /// Priority of the claim (higher values processed first)
    pub priority: u64,
    /// Oracle provider who is claiming the rewards
    pub provider_id: TraderId,
    /// Token being claimed
    pub token_id: TokenId,
    /// Amount of rewards to claim
    pub amount: Quantity,
    /// Timestamp when the claim was created
    pub timestamp: u64,
}

impl Ord for RewardClaim {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority values come first
        self.priority
            .cmp(&other.priority)
            .then_with(|| other.timestamp.cmp(&self.timestamp))
    }
}

impl PartialOrd for RewardClaim {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Manages reward claims using a priority queue
#[derive(Debug, Clone)]
pub struct RewardDistributionManager {
    /// Priority queue of reward claims
    claims: BinaryHeap<RewardClaim>,
}

impl RewardDistributionManager {
    /// Create a new reward distribution manager
    pub fn new() -> Self {
        Self {
            claims: BinaryHeap::new(),
        }
    }

    /// Add a new reward claim to the queue
    pub fn add_claim(&mut self, claim: RewardClaim) {
        self.claims.push(claim);
    }

    /// Process the next reward claim (highest priority)
    pub fn process_next_claim(&mut self) -> Option<RewardClaim> {
        self.claims.pop()
    }

    /// Get the number of pending reward claims
    pub fn pending_claims_count(&self) -> usize {
        self.claims.len()
    }

    /// Check if there are any pending reward claims
    pub fn has_pending_claims(&self) -> bool {
        !self.claims.is_empty()
    }

    /// Get a reference to the highest priority claim without removing it
    pub fn peek_next_claim(&self) -> Option<&RewardClaim> {
        self.claims.peek()
    }

    /// Remove all claims for a specific provider
    pub fn remove_claims_for_provider(&mut self, provider_id: &TraderId) -> Vec<RewardClaim> {
        let mut removed_claims = Vec::new();
        let mut remaining_claims = BinaryHeap::new();

        while let Some(claim) = self.claims.pop() {
            if &claim.provider_id == provider_id {
                removed_claims.push(claim);
            } else {
                remaining_claims.push(claim);
            }
        }

        self.claims = remaining_claims;
        removed_claims
    }

    /// Get all pending claims for a specific provider
    pub fn get_claims_for_provider(&self, provider_id: &TraderId) -> Vec<&RewardClaim> {
        self.claims
            .iter()
            .filter(|claim| &claim.provider_id == provider_id)
            .collect()
    }

    /// Get the total amount of rewards pending distribution
    pub fn total_pending_rewards(&self) -> Quantity {
        self.claims.iter().map(|claim| claim.amount).sum()
    }

    /// Clear all pending claims
    pub fn clear_pending_claims(&mut self) {
        self.claims.clear();
    }
}

/// Manages reward distribution for multiple token types
#[derive(Debug, Clone)]
pub struct MultiTokenRewardManager {
    /// Managers for different token types
    managers: std::collections::HashMap<TokenId, RewardDistributionManager>,
}

impl MultiTokenRewardManager {
    /// Create a new multi-token reward manager
    pub fn new() -> Self {
        Self {
            managers: std::collections::HashMap::new(),
        }
    }

    /// Add a reward claim for a specific token
    pub fn add_claim(&mut self, token_id: TokenId, claim: RewardClaim) {
        self.managers
            .entry(token_id)
            .or_insert_with(RewardDistributionManager::new)
            .add_claim(claim);
    }

    /// Process the next reward claim for a specific token
    pub fn process_next_claim(&mut self, token_id: &TokenId) -> Option<RewardClaim> {
        if let Some(manager) = self.managers.get_mut(token_id) {
            manager.process_next_claim()
        } else {
            None
        }
    }

    /// Process the next reward claim across all tokens (highest priority overall)
    pub fn process_next_claim_global(&mut self) -> Option<(TokenId, RewardClaim)> {
        let mut highest_priority_claim: Option<(&TokenId, &RewardClaim)> = None;

        // Find the highest priority claim across all tokens
        for (token_id, manager) in &self.managers {
            if let Some(claim) = manager.peek_next_claim() {
                if let Some((_, highest_claim)) = highest_priority_claim {
                    if claim.priority > highest_claim.priority
                        || (claim.priority == highest_claim.priority
                            && claim.timestamp < highest_claim.timestamp)
                    {
                        highest_priority_claim = Some((token_id, claim));
                    }
                } else {
                    highest_priority_claim = Some((token_id, claim));
                }
            }
        }

        // Process the highest priority claim
        if let Some((token_id, _)) = highest_priority_claim {
            let token_id_clone = token_id.clone();
            if let Some(claim) = self.process_next_claim(&token_id_clone) {
                return Some((token_id_clone, claim));
            }
        }

        None
    }

    /// Get the number of pending claims for a specific token
    pub fn pending_claims_count(&self, token_id: &TokenId) -> usize {
        if let Some(manager) = self.managers.get(token_id) {
            manager.pending_claims_count()
        } else {
            0
        }
    }

    /// Get the total number of pending claims across all tokens
    pub fn total_pending_claims_count(&self) -> usize {
        self.managers
            .values()
            .map(|manager| manager.pending_claims_count())
            .sum()
    }

    /// Check if there are any pending claims for a specific token
    pub fn has_pending_claims(&self, token_id: &TokenId) -> bool {
        if let Some(manager) = self.managers.get(token_id) {
            manager.has_pending_claims()
        } else {
            false
        }
    }

    /// Check if there are any pending claims across all tokens
    pub fn has_any_pending_claims(&self) -> bool {
        self.managers
            .values()
            .any(|manager| manager.has_pending_claims())
    }

    /// Get the total amount of rewards pending for a specific token
    pub fn total_pending_rewards(&self, token_id: &TokenId) -> Quantity {
        if let Some(manager) = self.managers.get(token_id) {
            manager.total_pending_rewards()
        } else {
            0
        }
    }

    /// Get the total amount of rewards pending across all tokens
    pub fn total_pending_rewards_global(&self) -> Quantity {
        self.managers
            .values()
            .map(|manager| manager.total_pending_rewards())
            .sum()
    }

    /// Remove all claims for a specific provider across all tokens
    pub fn remove_claims_for_provider(
        &mut self,
        provider_id: &TraderId,
    ) -> std::collections::HashMap<TokenId, Vec<RewardClaim>> {
        let mut removed_claims = std::collections::HashMap::new();

        for (token_id, manager) in &mut self.managers {
            let claims = manager.remove_claims_for_provider(provider_id);
            if !claims.is_empty() {
                removed_claims.insert(token_id.clone(), claims);
            }
        }

        removed_claims
    }
}

/// Errors that can occur during reward distribution operations
#[derive(Debug, Error)]
pub enum RewardDistributionError {
    #[error("No pending reward claims")]
    NoPendingClaims,
    #[error("Invalid claim priority")]
    InvalidPriority,
    #[error("Insufficient rewards available for claim")]
    InsufficientRewards,
    #[error("Provider not found")]
    ProviderNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reward_distribution_manager_creation() {
        let mut manager = RewardDistributionManager::new();
        assert_eq!(manager.pending_claims_count(), 0);
        assert!(!manager.has_pending_claims());
        assert!(manager.peek_next_claim().is_none());
        assert!(manager.process_next_claim().is_none());
    }

    #[test]
    fn test_add_and_process_claims() {
        let mut manager = RewardDistributionManager::new();

        // Add some reward claims
        let claim1 = RewardClaim {
            priority: 10,
            provider_id: "oracle1".to_string(),
            token_id: "BTC".to_string(),
            amount: 1000,
            timestamp: 1000000,
        };

        let claim2 = RewardClaim {
            priority: 20,
            provider_id: "oracle2".to_string(),
            token_id: "ETH".to_string(),
            amount: 2000,
            timestamp: 1000001,
        };

        let claim3 = RewardClaim {
            priority: 15,
            provider_id: "oracle3".to_string(),
            token_id: "USDC".to_string(),
            amount: 1500,
            timestamp: 1000002,
        };

        manager.add_claim(claim1.clone());
        manager.add_claim(claim2.clone());
        manager.add_claim(claim3.clone());

        assert_eq!(manager.pending_claims_count(), 3);
        assert!(manager.has_pending_claims());
        assert_eq!(manager.total_pending_rewards(), 4500);

        // Process claims - should be processed in priority order (20, then 15, then 10)
        let processed_claim = manager.process_next_claim().unwrap();
        assert_eq!(processed_claim.priority, 20);
        assert_eq!(processed_claim.provider_id, "oracle2");

        let processed_claim = manager.process_next_claim().unwrap();
        assert_eq!(processed_claim.priority, 15);
        assert_eq!(processed_claim.provider_id, "oracle3");

        let processed_claim = manager.process_next_claim().unwrap();
        assert_eq!(processed_claim.priority, 10);
        assert_eq!(processed_claim.provider_id, "oracle1");

        // No more claims
        assert!(manager.process_next_claim().is_none());
        assert_eq!(manager.pending_claims_count(), 0);
        assert_eq!(manager.total_pending_rewards(), 0);
    }

    #[test]
    fn test_remove_claims_for_provider() {
        let mut manager = RewardDistributionManager::new();

        // Add claims for different providers
        let claim1 = RewardClaim {
            priority: 10,
            provider_id: "oracle1".to_string(),
            token_id: "BTC".to_string(),
            amount: 1000,
            timestamp: 1000000,
        };

        let claim2 = RewardClaim {
            priority: 20,
            provider_id: "oracle2".to_string(),
            token_id: "ETH".to_string(),
            amount: 2000,
            timestamp: 1000001,
        };

        let claim3 = RewardClaim {
            priority: 15,
            provider_id: "oracle1".to_string(),
            token_id: "USDC".to_string(),
            amount: 1500,
            timestamp: 1000002,
        };

        manager.add_claim(claim1.clone());
        manager.add_claim(claim2.clone());
        manager.add_claim(claim3.clone());

        assert_eq!(manager.pending_claims_count(), 3);

        // Remove claims for oracle1
        let removed_claims = manager.remove_claims_for_provider(&"oracle1".to_string());
        assert_eq!(removed_claims.len(), 2);
        assert_eq!(manager.pending_claims_count(), 1);

        // Only oracle2's claim should remain
        let remaining_claim = manager.process_next_claim().unwrap();
        assert_eq!(remaining_claim.provider_id, "oracle2");
    }

    #[test]
    fn test_get_claims_for_provider() {
        let mut manager = RewardDistributionManager::new();

        // Add claims for different providers
        let claim1 = RewardClaim {
            priority: 10,
            provider_id: "oracle1".to_string(),
            token_id: "BTC".to_string(),
            amount: 1000,
            timestamp: 1000000,
        };

        let claim2 = RewardClaim {
            priority: 20,
            provider_id: "oracle2".to_string(),
            token_id: "ETH".to_string(),
            amount: 2000,
            timestamp: 1000001,
        };

        let claim3 = RewardClaim {
            priority: 15,
            provider_id: "oracle1".to_string(),
            token_id: "USDC".to_string(),
            amount: 1500,
            timestamp: 1000002,
        };

        manager.add_claim(claim1.clone());
        manager.add_claim(claim2.clone());
        manager.add_claim(claim3.clone());

        // Get claims for oracle1
        let oracle1_claims = manager.get_claims_for_provider(&"oracle1".to_string());
        assert_eq!(oracle1_claims.len(), 2);

        // Get claims for oracle2
        let oracle2_claims = manager.get_claims_for_provider(&"oracle2".to_string());
        assert_eq!(oracle2_claims.len(), 1);
        assert_eq!(oracle2_claims[0].provider_id, "oracle2");
    }

    #[test]
    fn test_multi_token_reward_manager() {
        let mut manager = MultiTokenRewardManager::new();

        // Add claims for different tokens
        let claim1 = RewardClaim {
            priority: 10,
            provider_id: "oracle1".to_string(),
            token_id: "BTC".to_string(),
            amount: 1000,
            timestamp: 1000000,
        };

        let claim2 = RewardClaim {
            priority: 20,
            provider_id: "oracle2".to_string(),
            token_id: "ETH".to_string(),
            amount: 2000,
            timestamp: 1000001,
        };

        manager.add_claim("BTC".to_string(), claim1.clone());
        manager.add_claim("ETH".to_string(), claim2.clone());

        assert_eq!(manager.pending_claims_count(&"BTC".to_string()), 1);
        assert_eq!(manager.pending_claims_count(&"ETH".to_string()), 1);
        assert_eq!(manager.total_pending_claims_count(), 2);
        assert!(manager.has_pending_claims(&"BTC".to_string()));
        assert!(manager.has_pending_claims(&"ETH".to_string()));
        assert!(manager.has_any_pending_claims());
        assert_eq!(manager.total_pending_rewards(&"BTC".to_string()), 1000);
        assert_eq!(manager.total_pending_rewards(&"ETH".to_string()), 2000);
        assert_eq!(manager.total_pending_rewards_global(), 3000);
    }

    #[test]
    fn test_process_next_claim_global() {
        let mut manager = MultiTokenRewardManager::new();

        // Add claims with different priorities
        let claim1 = RewardClaim {
            priority: 10,
            provider_id: "oracle1".to_string(),
            token_id: "BTC".to_string(),
            amount: 1000,
            timestamp: 1000000,
        };

        let claim2 = RewardClaim {
            priority: 20,
            provider_id: "oracle2".to_string(),
            token_id: "ETH".to_string(),
            amount: 2000,
            timestamp: 1000001,
        };

        let claim3 = RewardClaim {
            priority: 15,
            provider_id: "oracle3".to_string(),
            token_id: "USDC".to_string(),
            amount: 1500,
            timestamp: 1000002,
        };

        manager.add_claim("BTC".to_string(), claim1.clone());
        manager.add_claim("ETH".to_string(), claim2.clone());
        manager.add_claim("USDC".to_string(), claim3.clone());

        // Process claims globally - should process highest priority first (20)
        let processed = manager.process_next_claim_global().unwrap();
        assert_eq!(processed.0, "ETH".to_string());
        assert_eq!(processed.1.priority, 20);

        // Next should be priority 15
        let processed = manager.process_next_claim_global().unwrap();
        assert_eq!(processed.0, "USDC".to_string());
        assert_eq!(processed.1.priority, 15);

        // Next should be priority 10
        let processed = manager.process_next_claim_global().unwrap();
        assert_eq!(processed.0, "BTC".to_string());
        assert_eq!(processed.1.priority, 10);

        // No more claims
        assert!(manager.process_next_claim_global().is_none());
    }

    #[test]
    fn test_remove_claims_for_provider_global() {
        let mut manager = MultiTokenRewardManager::new();

        // Add claims for the same provider across different tokens
        let claim1 = RewardClaim {
            priority: 10,
            provider_id: "oracle1".to_string(),
            token_id: "BTC".to_string(),
            amount: 1000,
            timestamp: 1000000,
        };

        let claim2 = RewardClaim {
            priority: 20,
            provider_id: "oracle2".to_string(),
            token_id: "ETH".to_string(),
            amount: 2000,
            timestamp: 1000001,
        };

        let claim3 = RewardClaim {
            priority: 15,
            provider_id: "oracle1".to_string(),
            token_id: "USDC".to_string(),
            amount: 1500,
            timestamp: 1000002,
        };

        manager.add_claim("BTC".to_string(), claim1.clone());
        manager.add_claim("ETH".to_string(), claim2.clone());
        manager.add_claim("USDC".to_string(), claim3.clone());

        assert_eq!(manager.total_pending_claims_count(), 3);

        // Remove all claims for oracle1
        let removed_claims = manager.remove_claims_for_provider(&"oracle1".to_string());
        assert_eq!(removed_claims.len(), 2);
        assert!(removed_claims.contains_key(&"BTC".to_string()));
        assert!(removed_claims.contains_key(&"USDC".to_string()));
        assert_eq!(manager.total_pending_claims_count(), 1);

        // Only oracle2's claim should remain
        let remaining_claim = manager.process_next_claim_global().unwrap();
        assert_eq!(remaining_claim.1.provider_id, "oracle2");
    }
}
