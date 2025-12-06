//! StableSwap AMM implementation for the DEX-OS core engine
//!
//! This module implements the StableSwap invariant (x^3 * y + y^3 * x = k)
//! for providing low slippage trades between pegged assets.
//! This implements the Priority 2 feature from DEX-OS-V1.csv:
//! "Core Trading,AMM,AMM,Curve Fitting,StableSwap,High"

use crate::types::{Quantity, TokenId};
use std::collections::HashMap;
use thiserror::Error;

/// StableSwap AMM implementation
#[derive(Debug, Clone)]
pub struct StableSwapAMM {
    /// Reserves of tokens in the pool
    pub reserves: HashMap<TokenId, Quantity>,
    /// Total supply of liquidity tokens
    pub total_supply: Quantity,
    /// Fee percentage (in basis points, so 30 = 0.3%)
    pub fee: u32,
    /// Amplification coefficient (A) - higher values make the curve more flat
    pub amplification: u64,
}

impl StableSwapAMM {
    /// Create a new StableSwap AMM pool
    pub fn new(fee: u32, amplification: u64) -> Self {
        Self {
            reserves: HashMap::new(),
            total_supply: 0,
            fee,
            amplification,
        }
    }

    /// Add liquidity to the pool
    pub fn add_liquidity(
        &mut self,
        token_a: TokenId,
        amount_a: Quantity,
        token_b: TokenId,
        amount_b: Quantity,
    ) -> Result<Quantity, StableSwapError> {
        // Initialize reserves if they don't exist
        self.reserves.entry(token_a.clone()).or_insert(0);
        self.reserves.entry(token_b.clone()).or_insert(0);

        let reserve_a = *self.reserves.get(&token_a).unwrap();
        let reserve_b = *self.reserves.get(&token_b).unwrap();

        let liquidity_tokens = if self.total_supply == 0 {
            // First liquidity provider
            // Calculate initial liquidity tokens as geometric mean
            ((amount_a as f64 * amount_b as f64).sqrt() as Quantity).max(1)
        } else {
            // Subsequent liquidity providers
            // Calculate liquidity tokens based on proportional contribution using StableSwap invariant
            let d = self.calculate_invariant(reserve_a, reserve_b)?;
            let new_reserve_a = reserve_a + amount_a;
            let new_reserve_b = reserve_b + amount_b;
            let new_d = self.calculate_invariant(new_reserve_a, new_reserve_b)?;

            ((new_d - d) * self.total_supply) / d
        };

        // Update reserves
        *self.reserves.get_mut(&token_a).unwrap() += amount_a;
        *self.reserves.get_mut(&token_b).unwrap() += amount_b;

        // Update total supply
        self.total_supply += liquidity_tokens;

        Ok(liquidity_tokens)
    }

    /// Remove liquidity from the pool
    pub fn remove_liquidity(
        &mut self,
        token_a: TokenId,
        token_b: TokenId,
        liquidity_tokens: Quantity,
    ) -> Result<(Quantity, Quantity), StableSwapError> {
        if liquidity_tokens > self.total_supply {
            return Err(StableSwapError::InsufficientLiquidity);
        }

        let reserve_a = *self.reserves.get(&token_a).unwrap_or(&0);
        let reserve_b = *self.reserves.get(&token_b).unwrap_or(&0);

        let amount_a = (liquidity_tokens * reserve_a) / self.total_supply;
        let amount_b = (liquidity_tokens * reserve_b) / self.total_supply;

        // Update reserves
        *self.reserves.get_mut(&token_a).unwrap() -= amount_a;
        *self.reserves.get_mut(&token_b).unwrap() -= amount_b;

        // Update total supply
        self.total_supply -= liquidity_tokens;

        Ok((amount_a, amount_b))
    }

    /// Swap tokens in the pool using StableSwap invariant
    pub fn swap(
        &mut self,
        from_token: TokenId,
        to_token: TokenId,
        amount_in: Quantity,
    ) -> Result<Quantity, StableSwapError> {
        let reserve_in = *self
            .reserves
            .get(&from_token)
            .ok_or(StableSwapError::InvalidToken)?;
        let reserve_out = *self
            .reserves
            .get(&to_token)
            .ok_or(StableSwapError::InvalidToken)?;

        if reserve_in == 0 || reserve_out == 0 {
            return Err(StableSwapError::InsufficientLiquidity);
        }

        // Calculate amount out with fee using StableSwap invariant
        let amount_in_with_fee = (amount_in * (10000 - self.fee) as u64) / 10000;

        // Using simplified approach for StableSwap calculation
        // In a production implementation, this would use Newton-Raphson method for precision
        let d = self.calculate_invariant(reserve_in, reserve_out)?;

        // New reserve of input token
        let new_reserve_in = reserve_in + amount_in_with_fee;

        // Calculate new reserve of output token using the invariant
        let new_reserve_out = self.calculate_y_given_d_and_x(d, new_reserve_in)?;

        let amount_out = reserve_out - new_reserve_out;

        if amount_out >= reserve_out {
            return Err(StableSwapError::InsufficientLiquidity);
        }

        // Update reserves
        *self.reserves.get_mut(&from_token).unwrap() += amount_in;
        *self.reserves.get_mut(&to_token).unwrap() -= amount_out;

        Ok(amount_out)
    }

    /// Calculate the StableSwap invariant (D) using Curve's iterative method
    fn calculate_invariant(&self, x: Quantity, y: Quantity) -> Result<Quantity, StableSwapError> {
        if self.amplification == 0 {
            return Err(StableSwapError::InvalidAmplification);
        }

        if x == 0 && y == 0 {
            return Ok(0);
        }
        if x == 0 {
            return Ok(y);
        }
        if y == 0 {
            return Ok(x);
        }

        const N_COINS: u128 = 2;
        const MAX_ITERATIONS: u32 = 256;

        let x_u = x as u128;
        let y_u = y as u128;
        let sum = x_u + y_u;
        if sum == 0 {
            return Ok(0);
        }

        let amp = self.amplification as u128;
        let ann = amp * N_COINS * N_COINS; // A * n^n (n = 2 -> 4)
        let mut d = sum;

        for _ in 0..MAX_ITERATIONS {
            let mut d_p = d;
            d_p = d_p
                .checked_mul(d)
                .and_then(|v| v.checked_div(x_u * N_COINS))
                .ok_or(StableSwapError::NumericalOverflow)?;
            d_p = d_p
                .checked_mul(d)
                .and_then(|v| v.checked_div(y_u * N_COINS))
                .ok_or(StableSwapError::NumericalOverflow)?;

            let d_prev = d;

            let numerator = ann
                .checked_mul(sum)
                .and_then(|v| v.checked_add(d_p * N_COINS))
                .ok_or(StableSwapError::NumericalOverflow)?;
            let denominator = ann
                .checked_sub(1)
                .and_then(|v| v.checked_mul(d))
                .and_then(|v| v.checked_add((N_COINS + 1) * d_p))
                .ok_or(StableSwapError::NumericalOverflow)?;

            d = numerator
                .checked_mul(d)
                .and_then(|v| v.checked_div(denominator))
                .ok_or(StableSwapError::NumericalOverflow)?;

            if d > d_prev {
                if d - d_prev <= 1 {
                    break;
                }
            } else if d_prev - d <= 1 {
                break;
            }
        }

        Ok(d as Quantity)
    }

    /// Calculate y given D and x using the StableSwap invariant with Newton-Raphson method
    ///
    /// This implements the Newton-Raphson method for numerical computation
    /// as specified in DEX-OS-V1.csv for AMM Numerical Computation:
    /// "Core Trading,AMM,AMM,Newton-Raphson Method,Numerical Computation,Medium"
    fn calculate_y_given_d_and_x(
        &self,
        d: Quantity,
        x: Quantity,
    ) -> Result<Quantity, StableSwapError> {
        if self.amplification == 0 {
            return Err(StableSwapError::InvalidAmplification);
        }

        if d == 0 {
            return Ok(0);
        }

        const N_COINS: u128 = 2;
        const MAX_ITERATIONS: u32 = 256;

        let amp = self.amplification as u128;
        if amp == 0 {
            return Err(StableSwapError::InvalidAmplification);
        }

        let d_u = d as u128;
        let x_u = x as u128;
        if x_u == 0 {
            return Ok(d);
        }

        let ann = amp * N_COINS * N_COINS;

        // Compute c and b as per Curve StableSwap formula
        let mut c = d_u;
        c = c
            .checked_mul(d_u)
            .and_then(|v| v.checked_div(x_u * N_COINS))
            .ok_or(StableSwapError::NumericalOverflow)?;
        c = c
            .checked_mul(d_u)
            .and_then(|v| v.checked_div(ann * N_COINS))
            .ok_or(StableSwapError::NumericalOverflow)?;

        let b = x_u
            .checked_add(
                d_u.checked_div(ann)
                    .ok_or(StableSwapError::NumericalOverflow)?,
            )
            .ok_or(StableSwapError::NumericalOverflow)?;

        let mut y = d_u;
        for _ in 0..MAX_ITERATIONS {
            let numerator = y
                .checked_mul(y)
                .and_then(|v| v.checked_add(c))
                .ok_or(StableSwapError::NumericalOverflow)?;
            let denominator = y
                .checked_mul(2)
                .and_then(|v| v.checked_add(b))
                .and_then(|v| v.checked_sub(d_u))
                .ok_or(StableSwapError::NumericalOverflow)?;

            if denominator == 0 {
                return Err(StableSwapError::NumericalOverflow);
            }

            let y_prev = y;
            y = numerator
                .checked_div(denominator)
                .ok_or(StableSwapError::NumericalOverflow)?;

            if y > y_prev {
                if y - y_prev <= 1 {
                    break;
                }
            } else if y_prev - y <= 1 {
                break;
            }
        }

        Ok(y.min(u64::MAX as u128) as Quantity)
    }

    /// Get the price of one token in terms of another
    pub fn get_price(
        &self,
        from_token: &TokenId,
        to_token: &TokenId,
    ) -> Result<f64, StableSwapError> {
        let reserve_in = *self
            .reserves
            .get(from_token)
            .ok_or(StableSwapError::InvalidToken)?;
        let reserve_out = *self
            .reserves
            .get(to_token)
            .ok_or(StableSwapError::InvalidToken)?;

        if reserve_in == 0 {
            return Err(StableSwapError::InsufficientLiquidity);
        }

        Ok(reserve_out as f64 / reserve_in as f64)
    }
}

/// Errors that can occur when working with the StableSwap AMM
#[derive(Debug, Error)]
pub enum StableSwapError {
    #[error("Invalid token")]
    InvalidToken,
    #[error("Insufficient liquidity")]
    InsufficientLiquidity,
    #[error("Invalid amplification coefficient")]
    InvalidAmplification,
    #[error("Numerical overflow during calculation")]
    NumericalOverflow,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stableswap_creation() {
        let amm = StableSwapAMM::new(30, 100); // 0.3% fee, amplification factor 100
        assert_eq!(amm.total_supply, 0);
        assert_eq!(amm.fee, 30);
        assert_eq!(amm.amplification, 100);
        assert!(amm.reserves.is_empty());
    }

    #[test]
    fn test_add_liquidity() {
        let mut amm = StableSwapAMM::new(30, 100);
        let token_a = "DAI".to_string();
        let token_b = "USDC".to_string();

        let liquidity_tokens = amm
            .add_liquidity(
                token_a.clone(),
                1000000, // 1,000,000 DAI
                token_b.clone(),
                1000000, // 1,000,000 USDC
            )
            .unwrap();

        assert!(liquidity_tokens > 0);
        assert_eq!(amm.total_supply, liquidity_tokens);
        assert_eq!(*amm.reserves.get(&token_a).unwrap(), 1000000);
        assert_eq!(*amm.reserves.get(&token_b).unwrap(), 1000000);
    }

    #[test]
    fn test_swap() {
        let mut amm = StableSwapAMM::new(30, 100);
        let token_a = "DAI".to_string();
        let token_b = "USDC".to_string();

        // Add initial liquidity
        amm.add_liquidity(token_a.clone(), 1000000, token_b.clone(), 1000000)
            .unwrap();

        // Perform a swap
        let amount_out = amm
            .swap(
                token_a.clone(),
                token_b.clone(),
                10000, // Swap 10,000 DAI
            )
            .unwrap();

        assert!(amount_out > 0);
        assert!(amount_out < 10000); // Amount out should be less due to fee and slippage
    }

    #[test]
    fn test_newton_raphson_invariant_calculation() {
        let amm = StableSwapAMM::new(100, 1000); // 1% fee, amplification factor 1000

        // Test with equal reserves
        let d = amm.calculate_invariant(1000000, 1000000).unwrap();
        assert!(d > 0);
        // For equal reserves with high amplification, D should be close to sum of reserves
        assert!((d as i64 - 2000000).abs() < 10000);

        // Test with unequal reserves
        let d = amm.calculate_invariant(1000000, 2000000).unwrap();
        assert!(d > 0);
        // D should be between sum of reserves and 2 * geometric mean
        let sum = 1000000 + 2000000;
        let geom_mean = ((1000000 as f64 * 2000000 as f64).sqrt()) as Quantity;
        assert!(d > 2 * geom_mean);
        assert!(d < sum);
    }

    #[test]
    fn test_newton_raphson_y_calculation() {
        let amm = StableSwapAMM::new(100, 1000); // 1% fee, amplification factor 1000

        // Calculate D for initial state
        let d = amm.calculate_invariant(1000000, 1000000).unwrap();

        // Calculate y given D and x
        let y = amm.calculate_y_given_d_and_x(d, 1000000).unwrap();

        // Should be close to original y value
        assert!((y as i64 - 1000000).abs() < 1000);

        // Test with different x value
        let y = amm.calculate_y_given_d_and_x(d, 1500000).unwrap();
        assert!(y > 0);
        // When x increases, y should decrease to maintain invariant
        assert!(y < 1000000);
    }

    #[test]
    fn test_newton_raphson_precision() {
        let amm = StableSwapAMM::new(50, 500); // 0.5% fee, amplification factor 500

        // Test that the invariant calculation is self-consistent
        let x = 1000000;
        let y = 2000000;
        let d = amm.calculate_invariant(x, y).unwrap();

        // Verify that calculating y from D and x gives us back approximately y
        let calculated_y = amm.calculate_y_given_d_and_x(d, x).unwrap();
        assert!((calculated_y as i64 - y as i64).abs() < 1000);

        // Verify that calculating x from D and y gives us back approximately x
        let calculated_x = amm.calculate_y_given_d_and_x(d, y).unwrap();
        assert!((calculated_x as i64 - x as i64).abs() < 1000);
    }

    #[test]
    fn test_edge_cases() {
        let amm = StableSwapAMM::new(100, 1000);

        // Test with zero amplification (should error)
        let zero_amp_amm = StableSwapAMM::new(100, 0);
        assert!(zero_amp_amm.calculate_invariant(1000, 1000).is_err());
        assert!(zero_amp_amm.calculate_y_given_d_and_x(1000, 500).is_err());

        // Test with zero reserves
        let d = amm.calculate_invariant(0, 0).unwrap();
        assert_eq!(d, 0);

        // Test with one zero reserve
        let d = amm.calculate_invariant(1000, 0).unwrap();
        assert!(d > 0);
    }
}
