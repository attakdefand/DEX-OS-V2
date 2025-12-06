//! Impermanent Loss Protection (ILP) Insurance for liquidity providers
//!
//! Implements the Priority 5 feature from DEX-OS-V2.csv:
//! "Liquidity & Incentive, Liquidity Provision, Impermanent Loss Protection (ILP) Insurance"
//! with an application-security focus (Layer 4) via strict input validation, lockups,
//! and capped payouts backed by a risk buffer.

use crate::types::{Quantity, TraderId, TradingPair};
use std::collections::HashMap;
use thiserror::Error;

const DEFAULT_COVERAGE_GROWTH_SECS: u64 = 90 * 24 * 60 * 60; // 90 days to full coverage
const DEFAULT_MIN_LOCKUP_SECS: u64 = 7 * 24 * 60 * 60; // 7 days mandatory lockup
const MIN_PRICE_THRESHOLD: f64 = 1e-9;
const BALANCE_TOLERANCE: f64 = 0.05; // 5% imbalance allowed between base and quote deposits

/// Represents an insured liquidity position
#[derive(Debug, Clone)]
pub struct InsuredPosition {
    /// Unique policy identifier
    pub policy_id: u64,
    /// Liquidity provider ID
    pub provider: TraderId,
    /// Token pair covered by the policy
    pub pair: TradingPair,
    /// Amount of base token contributed
    pub base_amount: Quantity,
    /// Amount of quote token contributed
    pub quote_amount: Quantity,
    /// Entry price (quote per base) when liquidity was added
    pub entry_price: f64,
    /// When the liquidity was deposited
    pub deposit_timestamp: u64,
    /// Lockup end timestamp before ILP can be claimed
    pub lockup_until: u64,
    /// Seconds required to reach full coverage
    pub coverage_growth_secs: u64,
    /// Maximum portion of IL that can be covered (1.0 = 100%)
    pub max_coverage_pct: f64,
    /// Fees earned by the position that offset IL
    pub fees_earned: f64,
    /// Whether the policy is active
    pub active: bool,
}

impl InsuredPosition {
    /// Calculate IL metrics for the position at a given price
    fn calculate_il(&self, current_price: f64) -> Result<IlpComputation, IlpError> {
        if self.entry_price <= MIN_PRICE_THRESHOLD || current_price <= MIN_PRICE_THRESHOLD {
            return Err(IlpError::InvalidPrice);
        }

        // Impermanent loss math assumes a roughly balanced deposit
        let base_value_at_entry = self.base_amount as f64 * self.entry_price;
        let total_value_at_entry = base_value_at_entry + self.quote_amount as f64;
        let imbalance = (base_value_at_entry - self.quote_amount as f64).abs();
        if imbalance > total_value_at_entry * BALANCE_TOLERANCE {
            return Err(IlpError::UnbalancedDeposit);
        }

        // Value if the LP simply held the assets
        let hold_value = self.base_amount as f64 * current_price + self.quote_amount as f64;

        // Impermanent loss ratio derived from constant-product math
        let price_ratio = current_price / self.entry_price;
        let lp_multiplier = (2.0 * price_ratio.sqrt()) / (1.0 + price_ratio);
        let lp_value = hold_value * lp_multiplier;
        let impermanent_loss = (hold_value - lp_value).max(0.0);

        Ok(IlpComputation {
            hold_value,
            lp_value,
            impermanent_loss,
            price_ratio,
        })
    }

    /// Coverage progress after lockup, capped by `max_coverage_pct`
    fn coverage_progress(&self, timestamp: u64) -> f64 {
        if timestamp <= self.lockup_until {
            return 0.0;
        }

        let elapsed = timestamp - self.lockup_until;
        let progress = (elapsed as f64 / self.coverage_growth_secs as f64).min(1.0);
        progress * self.max_coverage_pct
    }
}

/// IL computation details (useful for reporting and tests)
#[derive(Debug, Clone)]
pub struct IlpComputation {
    pub hold_value: f64,
    pub lp_value: f64,
    pub impermanent_loss: f64,
    pub price_ratio: f64,
}

/// Quote for coverage and payout calculation
#[derive(Debug, Clone)]
pub struct IlpQuote {
    pub policy_id: u64,
    pub coverage_progress: f64,
    pub impermanent_loss: f64,
    pub net_impermanent_loss: f64,
    pub payout: f64,
    pub hold_value: f64,
    pub lp_value: f64,
    pub available_capacity: f64,
}

/// Impermanent Loss Protection engine managing policies and payouts
#[derive(Debug)]
pub struct ImpermanentLossProtection {
    coverage_pool: f64,
    coverage_growth_secs: u64,
    max_coverage_pct: f64,
    risk_buffer_ratio: f64,
    min_lockup_secs: u64,
    next_policy_id: u64,
    positions: HashMap<u64, InsuredPosition>,
    provider_index: HashMap<(TraderId, TradingPair), u64>,
}

impl ImpermanentLossProtection {
    /// Create a new ILP engine
    pub fn new(
        coverage_pool: f64,
        coverage_growth_secs: Option<u64>,
        max_coverage_pct: Option<f64>,
        risk_buffer_ratio: f64,
        min_lockup_secs: Option<u64>,
    ) -> Self {
        let growth_window = coverage_growth_secs.unwrap_or(DEFAULT_COVERAGE_GROWTH_SECS);
        Self {
            coverage_pool: coverage_pool.max(0.0),
            coverage_growth_secs: growth_window.max(1),
            max_coverage_pct: max_coverage_pct.unwrap_or(1.0).clamp(0.0, 1.0),
            risk_buffer_ratio: risk_buffer_ratio.clamp(0.0, 0.5),
            min_lockup_secs: min_lockup_secs.unwrap_or(DEFAULT_MIN_LOCKUP_SECS),
            next_policy_id: 1,
            positions: HashMap::new(),
            provider_index: HashMap::new(),
        }
    }

    /// Fund the coverage pool with additional reserves
    pub fn top_up_pool(&mut self, amount: f64) {
        if amount > 0.0 {
            self.coverage_pool += amount;
        }
    }

    /// Returns the amount reserved as a risk buffer (not usable for payouts)
    pub fn risk_buffer_amount(&self) -> f64 {
        self.coverage_pool * self.risk_buffer_ratio
    }

    /// Returns capacity available for new payouts after reserving the risk buffer
    pub fn available_capacity(&self) -> f64 {
        (self.coverage_pool - self.risk_buffer_amount()).max(0.0)
    }

    /// Register an insured liquidity position
    pub fn insure_position(
        &mut self,
        provider: TraderId,
        pair: TradingPair,
        base_amount: Quantity,
        quote_amount: Quantity,
        entry_price: f64,
        timestamp: u64,
        lockup_secs: u64,
    ) -> Result<u64, IlpError> {
        if base_amount == 0 || quote_amount == 0 {
            return Err(IlpError::InvalidDeposit);
        }

        if entry_price <= MIN_PRICE_THRESHOLD {
            return Err(IlpError::InvalidPrice);
        }

        let base_value_at_entry = base_amount as f64 * entry_price;
        let total_value_at_entry = base_value_at_entry + quote_amount as f64;
        let imbalance = (base_value_at_entry - quote_amount as f64).abs();
        if imbalance > total_value_at_entry * BALANCE_TOLERANCE {
            return Err(IlpError::UnbalancedDeposit);
        }

        let key = (provider.clone(), pair.clone());
        if self.provider_index.contains_key(&key) {
            return Err(IlpError::DuplicatePolicy);
        }

        let policy_id = self.next_policy_id;
        self.next_policy_id += 1;

        let position = InsuredPosition {
            policy_id,
            provider: provider.clone(),
            pair: pair.clone(),
            base_amount,
            quote_amount,
            entry_price,
            deposit_timestamp: timestamp,
            lockup_until: timestamp + lockup_secs.max(self.min_lockup_secs),
            coverage_growth_secs: self.coverage_growth_secs,
            max_coverage_pct: self.max_coverage_pct,
            fees_earned: 0.0,
            active: true,
        };

        self.positions.insert(policy_id, position);
        self.provider_index.insert(key, policy_id);
        Ok(policy_id)
    }

    /// Record additional fee earnings that offset IL for a policy
    pub fn record_fee_earnings(
        &mut self,
        policy_id: u64,
        fees_in_quote: f64,
    ) -> Result<(), IlpError> {
        let position = self
            .positions
            .get_mut(&policy_id)
            .ok_or(IlpError::PolicyNotFound(policy_id))?;

        position.fees_earned += fees_in_quote.max(0.0);
        Ok(())
    }

    /// Calculate the payout quote for a given policy without mutating state
    pub fn quote_payout(
        &self,
        policy_id: u64,
        current_price: f64,
        timestamp: u64,
    ) -> Result<IlpQuote, IlpError> {
        let position = self
            .positions
            .get(&policy_id)
            .ok_or(IlpError::PolicyNotFound(policy_id))?;

        if !position.active {
            return Err(IlpError::PolicyInactive);
        }

        if timestamp < position.lockup_until {
            return Err(IlpError::CoverageNotMatured(position.lockup_until));
        }

        let computation = position.calculate_il(current_price)?;
        if computation.impermanent_loss <= f64::EPSILON {
            return Err(IlpError::NoImpermanentLoss);
        }

        let coverage_progress = position.coverage_progress(timestamp);
        let net_il = (computation.impermanent_loss - position.fees_earned).max(0.0);
        let eligible = net_il * coverage_progress;
        let available_capacity = self.available_capacity();
        let payout = eligible.min(available_capacity);

        Ok(IlpQuote {
            policy_id,
            coverage_progress,
            impermanent_loss: computation.impermanent_loss,
            net_impermanent_loss: net_il,
            payout,
            hold_value: computation.hold_value,
            lp_value: computation.lp_value,
            available_capacity,
        })
    }

    /// Process a payout, reducing pool reserves and marking the policy inactive
    pub fn process_claim(
        &mut self,
        policy_id: u64,
        current_price: f64,
        timestamp: u64,
    ) -> Result<IlpQuote, IlpError> {
        let quote = self.quote_payout(policy_id, current_price, timestamp)?;

        if quote.payout <= f64::EPSILON {
            return Err(IlpError::NoImpermanentLoss);
        }

        if quote.payout > self.available_capacity() {
            return Err(IlpError::InsufficientCoveragePool);
        }

        let position = self
            .positions
            .get_mut(&policy_id)
            .ok_or(IlpError::PolicyNotFound(policy_id))?;

        position.active = false;
        self.coverage_pool -= quote.payout;
        self.provider_index
            .remove(&(position.provider.clone(), position.pair.clone()));

        Ok(quote)
    }

    /// Retrieve an active policy by ID
    pub fn get_policy(&self, policy_id: u64) -> Option<&InsuredPosition> {
        self.positions.get(&policy_id)
    }
}

/// Errors related to ILP operations
#[derive(Debug, Error, PartialEq)]
pub enum IlpError {
    #[error("Policy {0} not found")]
    PolicyNotFound(u64),
    #[error("Coverage not yet matured; lockup ends at {0}")]
    CoverageNotMatured(u64),
    #[error("Impermanent loss not present or already offset by fees")]
    NoImpermanentLoss,
    #[error("Policy is inactive")]
    PolicyInactive,
    #[error("Invalid price input")]
    InvalidPrice,
    #[error("Liquidity deposit must be balanced within 5%")]
    UnbalancedDeposit,
    #[error("Insufficient coverage pool after applying risk buffer")]
    InsufficientCoveragePool,
    #[error("Liquidity already insured for this provider and pair")]
    DuplicatePolicy,
    #[error("Deposit amounts must be greater than zero")]
    InvalidDeposit,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pair(base: &str, quote: &str) -> TradingPair {
        TradingPair {
            base: base.to_string(),
            quote: quote.to_string(),
        }
    }

    #[test]
    fn coverage_accumulates_after_lockup() {
        let mut ilp =
            ImpermanentLossProtection::new(1_000_000.0, Some(100), Some(1.0), 0.1, Some(10));
        let policy_id = ilp
            .insure_position(
                "lp1".to_string(),
                make_pair("ETH", "USDC"),
                10_000,
                10_000,
                1.0,
                0,
                5,
            )
            .unwrap();

        // Before lockup ends coverage is unavailable
        assert!(matches!(
            ilp.quote_payout(policy_id, 2.0, 8),
            Err(IlpError::CoverageNotMatured(_))
        ));

        // Halfway through coverage growth, payout should be roughly 50% of IL
        let quote = ilp.quote_payout(policy_id, 2.0, 60).unwrap();
        // Impermanent loss for 2x move on balanced pool is about 5.72%
        assert!(quote.coverage_progress > 0.4 && quote.coverage_progress <= 0.5);
        assert!(quote.payout > 0.0);

        // After full coverage window, payout should reach full eligible amount
        let full_quote = ilp.quote_payout(policy_id, 2.0, 150).unwrap();
        assert!(full_quote.coverage_progress <= 1.0);
        assert!(full_quote.payout >= quote.payout);
    }

    #[test]
    fn fees_offset_impermanent_loss() {
        let mut ilp = ImpermanentLossProtection::new(100_000.0, Some(50), Some(1.0), 0.05, Some(0));
        let policy_id = ilp
            .insure_position(
                "lp2".to_string(),
                make_pair("BTC", "USDT"),
                2_000,
                2_000,
                1.0,
                0,
                0,
            )
            .unwrap();

        ilp.record_fee_earnings(policy_id, 500.0).unwrap();
        let quote = ilp.quote_payout(policy_id, 1.5, 100).unwrap();
        assert!(quote.net_impermanent_loss + 1e-6 < quote.impermanent_loss);
    }

    #[test]
    fn risk_buffer_caps_payout_and_claim_marks_inactive() {
        let mut ilp = ImpermanentLossProtection::new(1_000.0, Some(10), Some(1.0), 0.1, Some(0));
        let policy_id = ilp
            .insure_position(
                "lp3".to_string(),
                make_pair("SOL", "USDC"),
                1_000,
                1_000,
                1.0,
                0,
                0,
            )
            .unwrap();

        // Large price swing should try to claim more than available (after buffer)
        let quote = ilp.quote_payout(policy_id, 3.0, 20).unwrap();
        assert!(quote.payout <= ilp.available_capacity());

        let processed = ilp.process_claim(policy_id, 3.0, 20).unwrap();
        assert_eq!(processed.payout, quote.payout);
        assert!(ilp.available_capacity() <= 1_000.0 - processed.payout);

        // Subsequent claim attempts should fail
        assert!(matches!(
            ilp.process_claim(policy_id, 3.0, 25),
            Err(IlpError::PolicyInactive)
        ));
    }

    #[test]
    fn detects_unbalanced_deposits() {
        let mut ilp = ImpermanentLossProtection::new(10_000.0, None, None, 0.1, None);
        let result = ilp.insure_position(
            "lp4".to_string(),
            make_pair("UNI", "USDC"),
            10_000,
            1_000, // Highly unbalanced
            1.0,
            0,
            0,
        );

        assert!(matches!(result, Err(IlpError::UnbalancedDeposit)));
    }
}
