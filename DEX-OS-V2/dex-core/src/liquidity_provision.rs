//! Liquidity provision module with LP token issuance and management
//!
//! Implements Priority 4 feature from DEX-OS-V2.csv:
//! - Liquidity & Incentive,Liquidity Provision,Liquidity Provision,LP Token Issuance,LP Token Management,High

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Metadata for LP tokens issued to liquidity providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LPToken {
    /// Symbol (e.g., "ETH/USDC-LP")
    pub symbol: String,
    /// Display name
    pub name: String,
    /// Decimal precision
    pub decimals: u8,
}

impl LPToken {
    /// Create standard LP token metadata for a pair
    pub fn new(symbol: &str, name: &str, decimals: u8) -> Self {
        Self {
            symbol: symbol.to_string(),
            name: name.to_string(),
            decimals,
        }
    }
}

/// Record of a provider's LP balance
#[derive(Debug, Clone)]
pub struct ProviderPosition {
    /// Provider identifier (e.g., wallet ID)
    pub provider_id: String,
    /// LP token balance
    pub lp_balance: u128,
}

/// Errors from liquidity provision operations
#[derive(Debug, Error, PartialEq)]
pub enum LiquidityProvisionError {
    /// Trying to register a pool that already exists
    #[error("Liquidity pool already exists")]
    PoolAlreadyExists,
    /// A referenced pool does not exist
    #[error("Liquidity pool not found")]
    PoolNotFound,
    /// Deposit amounts must be greater than zero
    #[error("Deposit amounts must be greater than zero")]
    InvalidDeposit,
    /// Deposits must follow the existing reserve ratio
    #[error("Deposit must match the reserve ratio")]
    RatioMismatch,
    /// Mint amount could not be calculated (too small or overflow)
    #[error("Minted LP amount is insufficient")]
    InsufficientMint,
    /// Provider does not hold enough LP tokens for the withdrawal
    #[error("Insufficient LP tokens")]
    InsufficientLp,
    /// Arithmetic overflow during calculations
    #[error("Arithmetic overflow during liquidity calculation")]
    MathOverflow,
}

/// Liquidity pool state
#[derive(Debug, Clone)]
pub struct LiquidityPool {
    pub id: String,
    pub asset_a: String,
    pub asset_b: String,
    pub reserve_a: u128,
    pub reserve_b: u128,
    pub total_lp_tokens: u128,
    pub lp_token: LPToken,
    pub providers: HashMap<String, ProviderPosition>,
}

impl LiquidityPool {
    fn new(id: &str, asset_a: &str, asset_b: &str) -> Self {
        let symbol = format!("{}/{}-LP", asset_a, asset_b);
        let name = format!("Liquidity Provider Token ({}/{})", asset_a, asset_b);
        Self {
            id: id.to_string(),
            asset_a: asset_a.to_string(),
            asset_b: asset_b.to_string(),
            reserve_a: 0,
            reserve_b: 0,
            total_lp_tokens: 0,
            lp_token: LPToken::new(&symbol, &name, 18),
            providers: HashMap::new(),
        }
    }

    fn validate_deposit(&self, deposit_a: u128, deposit_b: u128) -> Result<(), LiquidityProvisionError> {
        if deposit_a == 0 || deposit_b == 0 {
            return Err(LiquidityProvisionError::InvalidDeposit);
        }

        if self.total_lp_tokens > 0 {
            // ensure ratio stays consistent: deposit_a / deposit_b == reserve_a / reserve_b
            if self
                .reserve_a
                .checked_mul(deposit_b)
                .ok_or(LiquidityProvisionError::MathOverflow)?
                != self
                    .reserve_b
                    .checked_mul(deposit_a)
                    .ok_or(LiquidityProvisionError::MathOverflow)?
            {
                return Err(LiquidityProvisionError::RatioMismatch);
            }
        }

        Ok(())
    }

    fn compute_minted(&self, deposit_a: u128, deposit_b: u128) -> Result<u128, LiquidityProvisionError> {
        if self.total_lp_tokens == 0 {
            deposit_a
                .checked_add(deposit_b)
                .ok_or(LiquidityProvisionError::MathOverflow)
                .and_then(|sum| {
                    if sum == 0 {
                        Err(LiquidityProvisionError::InsufficientMint)
                    } else {
                        Ok(sum)
                    }
                })
        } else {
            let minted_a = deposit_a
                .checked_mul(self.total_lp_tokens)
                .ok_or(LiquidityProvisionError::MathOverflow)?
                .checked_div(self.reserve_a)
                .ok_or(LiquidityProvisionError::MathOverflow)?;
            let minted_b = deposit_b
                .checked_mul(self.total_lp_tokens)
                .ok_or(LiquidityProvisionError::MathOverflow)?
                .checked_div(self.reserve_b)
                .ok_or(LiquidityProvisionError::MathOverflow)?;
            let minted = minted_a.min(minted_b);
            if minted == 0 {
                Err(LiquidityProvisionError::InsufficientMint)
            } else {
                Ok(minted)
            }
        }
    }
}

/// Service that manages liquidity pools and LP issuance
#[derive(Debug, Default)]
pub struct LiquidityProvisionService {
    pools: HashMap<String, LiquidityPool>,
}

impl LiquidityProvisionService {
    /// Create a new service instance
    pub fn new() -> Self {
        Self {
            pools: HashMap::new(),
        }
    }

    /// Register a new liquidity pool
    pub fn create_pool(
        &mut self,
        pool_id: &str,
        asset_a: &str,
        asset_b: &str,
    ) -> Result<&LiquidityPool, LiquidityProvisionError> {
        if self.pools.contains_key(pool_id) {
            return Err(LiquidityProvisionError::PoolAlreadyExists);
        }

        let pool = LiquidityPool::new(pool_id, asset_a, asset_b);
        self.pools.insert(pool_id.to_string(), pool);
        Ok(self.pools.get(pool_id).unwrap())
    }

    /// Get a pool by its identifier
    pub fn get_pool(&self, pool_id: &str) -> Option<&LiquidityPool> {
        self.pools.get(pool_id)
    }

    /// Add liquidity and mint LP tokens for the provider
    pub fn add_liquidity(
        &mut self,
        pool_id: &str,
        provider_id: &str,
        deposit_a: u128,
        deposit_b: u128,
    ) -> Result<u128, LiquidityProvisionError> {
        let pool = self
            .pools
            .get_mut(pool_id)
            .ok_or(LiquidityProvisionError::PoolNotFound)?;

        pool.validate_deposit(deposit_a, deposit_b)?;
        let minted = pool.compute_minted(deposit_a, deposit_b)?;

        pool.reserve_a = pool
            .reserve_a
            .checked_add(deposit_a)
            .ok_or(LiquidityProvisionError::MathOverflow)?;
        pool.reserve_b = pool
            .reserve_b
            .checked_add(deposit_b)
            .ok_or(LiquidityProvisionError::MathOverflow)?;
        pool.total_lp_tokens = pool
            .total_lp_tokens
            .checked_add(minted)
            .ok_or(LiquidityProvisionError::MathOverflow)?;

        let position = pool
            .providers
            .entry(provider_id.to_string())
            .or_insert(ProviderPosition {
                provider_id: provider_id.to_string(),
                lp_balance: 0,
            });
        position.lp_balance = position
            .lp_balance
            .checked_add(minted)
            .ok_or(LiquidityProvisionError::MathOverflow)?;

        Ok(minted)
    }

    /// Remove liquidity and burn LP tokens
    pub fn remove_liquidity(
        &mut self,
        pool_id: &str,
        provider_id: &str,
        lp_amount: u128,
    ) -> Result<(u128, u128), LiquidityProvisionError> {
        let pool = self
            .pools
            .get_mut(pool_id)
            .ok_or(LiquidityProvisionError::PoolNotFound)?;

        if lp_amount == 0 {
            return Err(LiquidityProvisionError::InvalidDeposit);
        }

        let position = pool
            .providers
            .get_mut(provider_id)
            .ok_or(LiquidityProvisionError::InsufficientLp)?;

        if position.lp_balance < lp_amount {
            return Err(LiquidityProvisionError::InsufficientLp);
        }

        let amount_a = pool
            .reserve_a
            .checked_mul(lp_amount)
            .ok_or(LiquidityProvisionError::MathOverflow)?
            .checked_div(pool.total_lp_tokens)
            .ok_or(LiquidityProvisionError::MathOverflow)?;
        let amount_b = pool
            .reserve_b
            .checked_mul(lp_amount)
            .ok_or(LiquidityProvisionError::MathOverflow)?
            .checked_div(pool.total_lp_tokens)
            .ok_or(LiquidityProvisionError::MathOverflow)?;

        pool.reserve_a = pool
            .reserve_a
            .checked_sub(amount_a)
            .ok_or(LiquidityProvisionError::MathOverflow)?;
        pool.reserve_b = pool
            .reserve_b
            .checked_sub(amount_b)
            .ok_or(LiquidityProvisionError::MathOverflow)?;
        pool.total_lp_tokens = pool
            .total_lp_tokens
            .checked_sub(lp_amount)
            .ok_or(LiquidityProvisionError::MathOverflow)?;

        position.lp_balance = position
            .lp_balance
            .checked_sub(lp_amount)
            .ok_or(LiquidityProvisionError::MathOverflow)?;

        Ok((amount_a, amount_b))
    }

    /// Get a provider's LP token balance
    pub fn provider_balance(
        &self,
        pool_id: &str,
        provider_id: &str,
    ) -> Option<&ProviderPosition> {
        self.pools
            .get(pool_id)
            .and_then(|pool| pool.providers.get(provider_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_pool_and_mint() {
        let mut service = LiquidityProvisionService::new();
        service
            .create_pool("pool_eth_usdc", "ETH", "USDC")
            .unwrap();

        let minted = service
            .add_liquidity("pool_eth_usdc", "provider_1", 1_000_000, 2_000_000)
            .unwrap();
        assert_eq!(minted, 3_000_000);

        let pool = service.get_pool("pool_eth_usdc").unwrap();
        assert_eq!(pool.total_lp_tokens, minted);
        let position = service.provider_balance("pool_eth_usdc", "provider_1").unwrap();
        assert_eq!(position.lp_balance, minted);
    }

    #[test]
    fn test_proportional_minting_with_existing_pool() {
        let mut service = LiquidityProvisionService::new();
        service
            .create_pool("pool_eth_usdc", "ETH", "USDC")
            .unwrap();

        let first = service
            .add_liquidity("pool_eth_usdc", "provider_1", 1_000, 1_000)
            .unwrap();

        let minted_second = service
            .add_liquidity("pool_eth_usdc", "provider_2", 500, 500)
            .unwrap();
        assert_eq!(minted_second, 500);

        let pool = service.get_pool("pool_eth_usdc").unwrap();
        assert_eq!(pool.total_lp_tokens, first + minted_second);
    }

    #[test]
    fn test_remove_liquidity_returns_assets() {
        let mut service = LiquidityProvisionService::new();
        service
            .create_pool("pool_eth_usdc", "ETH", "USDC")
            .unwrap();

        let minted = service
            .add_liquidity("pool_eth_usdc", "provider_1", 2_000, 2_000)
            .unwrap();
        let (returned_a, returned_b) = service
            .remove_liquidity("pool_eth_usdc", "provider_1", minted / 2)
            .unwrap();

        assert_eq!(returned_a, 1_000);
        assert_eq!(returned_b, 1_000);
        let position = service.provider_balance("pool_eth_usdc", "provider_1").unwrap();
        assert_eq!(position.lp_balance, minted / 2);
    }

    #[test]
    fn test_remove_liquidity_requires_balance() {
        let mut service = LiquidityProvisionService::new();
        service
            .create_pool("pool_eth_usdc", "ETH", "USDC")
            .unwrap();
        let minted = service
            .add_liquidity("pool_eth_usdc", "provider_1", 1_000, 1_000)
            .unwrap();

        let err = service
            .remove_liquidity("pool_eth_usdc", "provider_1", minted + 1)
            .unwrap_err();
        assert_eq!(err, LiquidityProvisionError::InsufficientLp);
    }
}
