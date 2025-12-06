//! Advanced DeFi Features Module
//!
//! This module integrates all the advanced DeFi features to create a comprehensive
//! DeFi platform similar to Blackhole DEX.

use crate::governance::{VeNFTRegistry, vote_escrow::{VeNFT, VeNFTType}};
use crate::genesis_pool::{GenesisPool, GenesisPoolManager};
use crate::concentrated_liquidity::{ConcentratedLiquidityManager, ConcentratedPool, Amount};
use crate::types::{TokenId, TraderId, Price};

/// Main interface for advanced DeFi features
pub struct AdvancedDeFiPlatform {
    pub ve_registry: VeNFTRegistry,
    pub genesis_manager: GenesisPoolManager,
    pub cl_manager: ConcentratedLiquidityManager,
}

impl AdvancedDeFiPlatform {
    /// Create a new Advanced DeFi Platform
    pub fn new() -> Self {
        Self {
            ve_registry: VeNFTRegistry::new(),
            genesis_manager: GenesisPoolManager::new(),
            cl_manager: ConcentratedLiquidityManager::new(),
        }
    }

    /// Create a Vote Escrow NFT
    pub fn create_venft(
        &mut self,
        owner: String,
        amount: u128,
        lock_duration: u64,
        nft_type: VeNFTType,
    ) -> u64 {
        self.ve_registry.create_venft(owner, amount, lock_duration, nft_type)
    }

    /// Get total voting power for an owner
    pub fn get_owner_voting_power(&self, owner: &str) -> f64 {
        self.ve_registry.get_owner_total_voting_power(owner)
    }

    /// Create a Genesis Pool
    pub fn create_genesis_pool(
        &mut self,
        id: String,
        project_token_id: TokenId,
        contribution_token_id: TokenId,
        project_token_amount: u128,
        contribution_target: u128,
        min_contribution: u128,
        max_contribution: u128,
        start_time: u64,
        end_time: u64,
    ) {
        let pool = GenesisPool::new(
            id.clone(),
            project_token_id,
            contribution_token_id,
            project_token_amount,
            contribution_target,
            min_contribution,
            max_contribution,
            start_time,
            end_time,
        );
        self.genesis_manager.add_pool(pool);
    }

    /// Contribute to a Genesis Pool
    pub fn contribute_to_genesis_pool(
        &mut self,
        pool_id: &str,
        trader_id: TraderId,
        amount: Amount,
        current_time: u64,
    ) -> Result<(), String> {
        let pool = self.genesis_manager.get_pool_mut(pool_id)
            .ok_or("Pool not found")?;
        pool.contribute(trader_id, amount as u128, current_time)
    }

    /// Create a Concentrated Liquidity Pool
    pub fn create_concentrated_pool(
        &mut self,
        id: String,
        token0: TokenId,
        token1: TokenId,
        fee: u32,
        tick_spacing: i32,
        initial_price: Price,
    ) -> String {
        self.cl_manager.create_pool(id, token0, token1, fee, tick_spacing, initial_price)
    }

    /// Add liquidity to a concentrated pool
    pub fn add_concentrated_liquidity(
        &mut self,
        pool_id: &str,
        owner: String,
        tick_lower: i32,
        tick_upper: i32,
        amount0: Amount,
        amount1: Amount,
    ) -> Result<String, String> {
        let pool = self.cl_manager.get_pool_mut(pool_id)
            .ok_or("Pool not found")?;
        pool.create_position(owner, tick_lower, tick_upper, amount0, amount1)
    }

    /// Execute a swap in a concentrated pool
    pub fn swap(
        &mut self,
        pool_id: &str,
        zero_for_one: bool,
        amount_specified: Amount,
    ) -> Result<(Amount, Amount), String> {
        let pool = self.cl_manager.get_pool_mut(pool_id)
            .ok_or("Pool not found")?;
        pool.swap(zero_for_one, amount_specified)
    }

    /// Get platform statistics
    pub fn get_platform_stats(&self) -> PlatformStats {
        PlatformStats {
            total_venfts: self.ve_registry.nfts.len(),
            total_genesis_pools: self.genesis_manager.get_all_pools().len(),
            total_cl_pools: self.cl_manager.pools.len(),
        }
    }
}

/// Platform statistics
#[derive(Debug, Clone)]
pub struct PlatformStats {
    pub total_venfts: usize,
    pub total_genesis_pools: usize,
    pub total_cl_pools: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advanced_defi_platform() {
        let mut platform = AdvancedDeFiPlatform::new();
        
        // Create a veNFT
        let venft_id = platform.create_venft(
            "user1".to_string(),
            1000,
            365 * 24 * 60 * 60, // 1 year
            VeNFTType::Supermassive,
        );
        
        assert_eq!(venft_id, 1);
        assert_eq!(platform.get_owner_voting_power("user1"), 1100.0); // 10% boost
        
        // Create a Genesis Pool
        platform.create_genesis_pool(
            "pool1".to_string(),
            "PROJECT".to_string(),
            "USDC".to_string(),
            1000000,
            5000000,
            1000,
            100000,
            1000000,
            2000000,
        );
        
        // Contribute to the pool
        let result = platform.contribute_to_genesis_pool(
            "pool1",
            "user1".to_string(),
            50000,
            1500000,
        );
        assert!(result.is_ok());
        
        // Create a concentrated liquidity pool
        let pool_id = platform.create_concentrated_pool(
            "cl_pool1".to_string(),
            "TOKEN0".to_string(),
            "TOKEN1".to_string(),
            30, // 0.3% fee
            10,
            1000000, // 1:1 price (scaled)
        );
        
        assert_eq!(pool_id, "cl_pool1");
        
        // Add liquidity
        let result = platform.add_concentrated_liquidity(
            &pool_id,
            "user1".to_string(),
            -100,
            100,
            10000,
            10000,
        );
        assert!(result.is_ok());
        
        // Check stats
        let stats = platform.get_platform_stats();
        assert_eq!(stats.total_venfts, 1);
        assert_eq!(stats.total_genesis_pools, 1);
        assert_eq!(stats.total_cl_pools, 1);
    }
}
