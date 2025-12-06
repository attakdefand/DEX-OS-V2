//! Yield farming lock-up management for the DEX-OS core engine
//!
//! Implements the Priority 5 feature from DEX-OS-V2.csv:
//! "Liquidity & Incentive,Yield Farming/Staking,Yield Farming,Lock-up Periods,Lock-up Management,Medium {Security: Layer 4 - Application Security}"
//! with a focus on enforcing application-layer safeguards around staking schedules,
//! early exits, and pool-wide risk controls.

use crate::types::{Quantity, TokenId, TraderId};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

const BASIS_POINTS: u64 = 10_000;
const SECONDS_PER_YEAR: u64 = 31_536_000; // 365 * 24 * 60 * 60

/// Status of a lock-up position
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockupStatus {
    /// Position is actively accruing rewards and cannot be withdrawn without penalty
    Active,
    /// Position completed its lock-up and was released normally
    Released,
    /// Position was exited before maturity and incurred a penalty
    EarlyExited,
    /// Position was cancelled without rewards (e.g., rejected unlock)
    Cancelled,
}

/// Configuration for a supported lock-up period
#[derive(Debug, Clone, PartialEq)]
pub struct LockupPeriodConfig {
    /// Unique identifier for this lock-up period
    pub id: u64,
    /// Token that can be staked into this period
    pub token_id: TokenId,
    /// Duration in seconds
    pub duration_seconds: u64,
    /// APR in basis points (e.g., 1500 = 15%)
    pub apr_bps: u32,
    /// Early exit penalty in basis points applied to principal
    pub early_exit_penalty_bps: u32,
    /// Minimum amount that can be staked for this period
    pub min_amount: Quantity,
}

/// Represents a user's staked position
#[derive(Debug, Clone)]
pub struct LockupPosition {
    /// Unique identifier for the position
    pub position_id: u64,
    /// Owner of the position
    pub trader_id: TraderId,
    /// Token being staked
    pub token_id: TokenId,
    /// Principal amount locked
    pub amount: Quantity,
    /// Reward APR in basis points
    pub reward_apr_bps: u32,
    /// Early exit penalty in basis points
    pub early_exit_penalty_bps: u32,
    /// Timestamp when the position started
    pub start_timestamp: u64,
    /// Required lock-up duration
    pub lockup_duration: u64,
    /// Timestamp when funds can be released without penalty
    pub unlock_timestamp: u64,
    /// Current status of the position
    pub status: LockupStatus,
    /// Rewards that have been realized/claimed
    pub claimed_rewards: Quantity,
}

impl LockupPosition {
    /// Calculate accrued rewards at the given timestamp (capped at lock-up duration)
    pub fn accrued_rewards(&self, current_timestamp: u64) -> Quantity {
        let elapsed = current_timestamp.saturating_sub(self.start_timestamp);
        let capped_elapsed = elapsed.min(self.lockup_duration);

        let reward = (self.amount as u128)
            .saturating_mul(self.reward_apr_bps as u128)
            .saturating_mul(capped_elapsed as u128)
            / ((BASIS_POINTS as u128) * (SECONDS_PER_YEAR as u128));

        reward as Quantity
    }
}

/// Application security policy for lock-up operations
#[derive(Debug, Clone)]
pub struct LockupPolicy {
    /// Tokens that are allowed to be locked (empty set means all tokens allowed)
    pub allowed_tokens: HashSet<TokenId>,
    /// Minimum duration permitted for any lock-up
    pub min_duration: u64,
    /// Maximum duration permitted for any lock-up (0 means unlimited)
    pub max_duration: u64,
    /// Maximum total amount a single user can lock across all positions
    pub max_user_locked: Quantity,
    /// Maximum total locked per token across all users
    pub max_total_locked_per_token: Quantity,
    /// Whether early exits are globally permitted
    pub allow_early_exit: bool,
}

impl Default for LockupPolicy {
    fn default() -> Self {
        Self {
            allowed_tokens: HashSet::new(),
            min_duration: 3_600,                     // 1 hour baseline guardrail
            max_duration: SECONDS_PER_YEAR * 3,      // 3 years upper bound
            max_user_locked: 10_000_000,             // Default per-user cap
            max_total_locked_per_token: 100_000_000, // Default per-token cap
            allow_early_exit: true,                  // Permit early exits unless tightened
        }
    }
}

/// Result of unlocking a position
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockupSettlement {
    /// Principal released back to the user after penalties
    pub principal_released: Quantity,
    /// Rewards earned based on lock-up duration and APR
    pub reward_earned: Quantity,
    /// Penalty applied on early exit (in token units)
    pub penalty_applied: Quantity,
    /// Final status of the position
    pub status: LockupStatus,
}

/// Errors returned by lock-up management operations
#[derive(Debug, Error, PartialEq, Eq)]
pub enum LockupError {
    #[error("Token {0} is not allowed for lock-up farming")]
    TokenNotAllowed(TokenId),
    #[error("Lock-up duration outside permitted bounds")]
    DurationOutOfBounds,
    #[error("APR basis points are invalid")]
    InvalidApr,
    #[error("Early exit penalty basis points are invalid")]
    InvalidPenalty,
    #[error("Stake amount below minimum for the selected period")]
    AmountTooLow,
    #[error("Lock-up period not found")]
    UnknownPeriod,
    #[error("Lock-up position not found")]
    PositionNotFound,
    #[error("Caller is not authorized to operate on this position")]
    Unauthorized,
    #[error("Early exit is not allowed by policy")]
    EarlyExitNotAllowed,
    #[error("Position already settled")]
    PositionSettled,
    #[error("User lock limit exceeded")]
    UserLockLimitExceeded,
    #[error("Pool lock limit exceeded for token {0}")]
    PoolLockLimitExceeded(TokenId),
    #[error("Duration extension exceeds policy limits")]
    DurationExtensionNotAllowed,
}

/// Manager for lock-up periods and positions
#[derive(Debug, Clone)]
pub struct LockupManager {
    periods: HashMap<u64, LockupPeriodConfig>,
    positions: HashMap<u64, LockupPosition>,
    positions_by_trader: HashMap<TraderId, Vec<u64>>,
    total_locked_per_token: HashMap<TokenId, Quantity>,
    policy: LockupPolicy,
    next_period_id: u64,
    next_position_id: u64,
    audit_log: Vec<String>,
}

impl LockupManager {
    /// Create a new lock-up manager with the provided security policy
    pub fn new(policy: LockupPolicy) -> Self {
        Self {
            periods: HashMap::new(),
            positions: HashMap::new(),
            positions_by_trader: HashMap::new(),
            total_locked_per_token: HashMap::new(),
            policy,
            next_period_id: 1,
            next_position_id: 1,
            audit_log: Vec::new(),
        }
    }

    /// Register a new lock-up period
    pub fn create_period(
        &mut self,
        token_id: TokenId,
        duration_seconds: u64,
        apr_bps: u32,
        early_exit_penalty_bps: u32,
        min_amount: Quantity,
    ) -> Result<u64, LockupError> {
        self.validate_period_input(
            &token_id,
            duration_seconds,
            apr_bps,
            early_exit_penalty_bps,
            min_amount,
        )?;

        let period_id = self.next_period_id;
        self.next_period_id += 1;

        let config = LockupPeriodConfig {
            id: period_id,
            token_id: token_id.clone(),
            duration_seconds,
            apr_bps,
            early_exit_penalty_bps,
            min_amount,
        };

        self.periods.insert(period_id, config);
        self.record_audit(format!(
            "Created lock-up period {} for token {} (duration={}s, apr={}bps, penalty={}bps)",
            period_id, token_id, duration_seconds, apr_bps, early_exit_penalty_bps
        ));
        Ok(period_id)
    }

    /// Stake tokens into a lock-up period
    pub fn stake_with_lockup(
        &mut self,
        trader_id: TraderId,
        period_id: u64,
        amount: Quantity,
        start_timestamp: u64,
    ) -> Result<u64, LockupError> {
        let period = self
            .periods
            .get(&period_id)
            .ok_or(LockupError::UnknownPeriod)?
            .clone();

        if !self.is_token_allowed(&period.token_id) {
            return Err(LockupError::TokenNotAllowed(period.token_id));
        }

        if amount < period.min_amount {
            return Err(LockupError::AmountTooLow);
        }

        // User-level cap enforcement
        let current_user_locked = self.current_user_locked(&trader_id);
        if current_user_locked.saturating_add(amount) > self.policy.max_user_locked {
            return Err(LockupError::UserLockLimitExceeded);
        }

        // Pool-level cap enforcement
        let token_total = self.total_locked_for_token(&period.token_id);
        if token_total.saturating_add(amount) > self.policy.max_total_locked_per_token {
            return Err(LockupError::PoolLockLimitExceeded(period.token_id));
        }

        let unlock_timestamp = start_timestamp.saturating_add(period.duration_seconds);

        let position_id = self.next_position_id;
        self.next_position_id += 1;

        let position = LockupPosition {
            position_id,
            trader_id: trader_id.clone(),
            token_id: period.token_id.clone(),
            amount,
            reward_apr_bps: period.apr_bps,
            early_exit_penalty_bps: period.early_exit_penalty_bps,
            start_timestamp,
            lockup_duration: period.duration_seconds,
            unlock_timestamp,
            status: LockupStatus::Active,
            claimed_rewards: 0,
        };

        self.positions.insert(position_id, position);
        self.positions_by_trader
            .entry(trader_id.clone())
            .or_default()
            .push(position_id);
        self.total_locked_per_token
            .entry(period.token_id.clone())
            .and_modify(|total| *total += amount)
            .or_insert(amount);

        self.record_audit(format!(
            "Trader {} locked {} of {} into period {} until {}",
            trader_id, amount, period.token_id, period_id, unlock_timestamp
        ));

        Ok(position_id)
    }

    /// Unlock a position, optionally allowing early exit with penalty
    pub fn unlock_position(
        &mut self,
        trader_id: &TraderId,
        position_id: u64,
        current_timestamp: u64,
        allow_early_exit: bool,
    ) -> Result<LockupSettlement, LockupError> {
        let (token_id, amount, matured, penalty, rewards, status) = {
            let position = self
                .positions
                .get_mut(&position_id)
                .ok_or(LockupError::PositionNotFound)?;

            if &position.trader_id != trader_id {
                return Err(LockupError::Unauthorized);
            }

            if position.status != LockupStatus::Active {
                return Err(LockupError::PositionSettled);
            }

            let matured = current_timestamp >= position.unlock_timestamp;
            if !matured && (!self.policy.allow_early_exit || !allow_early_exit) {
                return Err(LockupError::EarlyExitNotAllowed);
            }

            let penalty = if matured {
                0
            } else {
                ((position.amount as u128 * position.early_exit_penalty_bps as u128)
                    / (BASIS_POINTS as u128)) as Quantity
            };

            let rewards = if matured {
                position.accrued_rewards(current_timestamp)
            } else {
                0
            };

            position.claimed_rewards = rewards;
            position.status = if matured {
                LockupStatus::Released
            } else {
                LockupStatus::EarlyExited
            };

            (
                position.token_id.clone(),
                position.amount,
                matured,
                penalty,
                rewards,
                position.status.clone(),
            )
        };

        // Update pool totals after release
        if let Some(total) = self.total_locked_per_token.get_mut(&token_id) {
            *total = total.saturating_sub(amount);
        }

        self.record_audit(format!(
            "Trader {} unlocked position {} (matured={}, penalty={}, rewards={})",
            trader_id, position_id, matured, penalty, rewards
        ));

        Ok(LockupSettlement {
            principal_released: amount.saturating_sub(penalty),
            reward_earned: rewards,
            penalty_applied: penalty,
            status,
        })
    }

    /// Extend a lock-up duration while enforcing duration bounds
    pub fn extend_lockup(
        &mut self,
        trader_id: &TraderId,
        position_id: u64,
        additional_seconds: u64,
    ) -> Result<(), LockupError> {
        if additional_seconds == 0 {
            return Err(LockupError::DurationOutOfBounds);
        }

        let new_unlock_timestamp = {
            let position = self
                .positions
                .get_mut(&position_id)
                .ok_or(LockupError::PositionNotFound)?;

            if &position.trader_id != trader_id {
                return Err(LockupError::Unauthorized);
            }

            if position.status != LockupStatus::Active {
                return Err(LockupError::PositionSettled);
            }

            let new_duration = position
                .lockup_duration
                .checked_add(additional_seconds)
                .ok_or(LockupError::DurationExtensionNotAllowed)?;

            if self.policy.max_duration != 0 && new_duration > self.policy.max_duration {
                return Err(LockupError::DurationExtensionNotAllowed);
            }

            position.lockup_duration = new_duration;
            position
                .unlock_timestamp
                .checked_add(additional_seconds)
                .ok_or(LockupError::DurationExtensionNotAllowed)?
        };

        if let Some(position) = self.positions.get_mut(&position_id) {
            position.unlock_timestamp = new_unlock_timestamp;
        }

        self.record_audit(format!(
            "Extended position {} by {} seconds; new unlock at {}",
            position_id, additional_seconds, new_unlock_timestamp
        ));

        Ok(())
    }

    /// Get a position by ID
    pub fn get_position(&self, position_id: u64) -> Option<&LockupPosition> {
        self.positions.get(&position_id)
    }

    /// Get all positions for a trader
    pub fn positions_for_trader(&self, trader_id: &TraderId) -> Vec<&LockupPosition> {
        if let Some(ids) = self.positions_by_trader.get(trader_id) {
            ids.iter().filter_map(|id| self.positions.get(id)).collect()
        } else {
            Vec::new()
        }
    }

    /// Get the total locked amount for a token
    pub fn total_locked_for_token(&self, token_id: &TokenId) -> Quantity {
        self.total_locked_per_token
            .get(token_id)
            .copied()
            .unwrap_or(0)
    }

    /// Access audit log entries (most recent last)
    pub fn audit_log(&self) -> &[String] {
        &self.audit_log
    }

    /// List lock-up periods for a token
    pub fn periods_for_token(&self, token_id: &TokenId) -> Vec<&LockupPeriodConfig> {
        self.periods
            .values()
            .filter(|period| &period.token_id == token_id)
            .collect()
    }

    fn validate_period_input(
        &self,
        token_id: &TokenId,
        duration_seconds: u64,
        apr_bps: u32,
        early_exit_penalty_bps: u32,
        min_amount: Quantity,
    ) -> Result<(), LockupError> {
        if !self.is_token_allowed(token_id) {
            return Err(LockupError::TokenNotAllowed(token_id.clone()));
        }

        if duration_seconds < self.policy.min_duration {
            return Err(LockupError::DurationOutOfBounds);
        }

        if self.policy.max_duration != 0 && duration_seconds > self.policy.max_duration {
            return Err(LockupError::DurationOutOfBounds);
        }

        if apr_bps == 0 || apr_bps as u64 > BASIS_POINTS * 50 {
            // Cap APR at 5000% to avoid unsafe incentives
            return Err(LockupError::InvalidApr);
        }

        if early_exit_penalty_bps as u64 > BASIS_POINTS {
            return Err(LockupError::InvalidPenalty);
        }

        if min_amount == 0 {
            return Err(LockupError::AmountTooLow);
        }

        Ok(())
    }

    fn is_token_allowed(&self, token_id: &TokenId) -> bool {
        self.policy.allowed_tokens.is_empty() || self.policy.allowed_tokens.contains(token_id)
    }

    fn current_user_locked(&self, trader_id: &TraderId) -> Quantity {
        self.positions_for_trader(trader_id)
            .iter()
            .filter(|position| position.status == LockupStatus::Active)
            .map(|position| position.amount)
            .sum()
    }

    fn record_audit(&mut self, message: String) {
        self.audit_log.push(message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_policy_with_token(token: &str) -> LockupPolicy {
        let mut allowed = HashSet::new();
        allowed.insert(token.to_string());
        LockupPolicy {
            allowed_tokens: allowed,
            min_duration: 86_400, // 1 day minimum for tests
            max_duration: SECONDS_PER_YEAR,
            max_user_locked: 1_000_000,
            max_total_locked_per_token: 10_000_000,
            allow_early_exit: true,
        }
    }

    #[test]
    fn test_register_period_and_mature_unlock() {
        let token = "LP-ETH".to_string();
        let mut manager = LockupManager::new(default_policy_with_token(&token));

        let period_id = manager
            .create_period(token.clone(), 30 * 86_400, 1500, 500, 100)
            .unwrap();

        let position_id = manager
            .stake_with_lockup("farmer1".to_string(), period_id, 1_000, 0)
            .unwrap();

        let unlock_timestamp = manager
            .get_position(position_id)
            .map(|p| p.unlock_timestamp)
            .unwrap();
        let settlement = manager
            .unlock_position(&"farmer1".to_string(), position_id, unlock_timestamp, true)
            .unwrap();

        assert_eq!(settlement.status, LockupStatus::Released);
        assert!(settlement.reward_earned > 0);
        assert_eq!(settlement.penalty_applied, 0);
        assert_eq!(manager.total_locked_for_token(&token), 0);
    }

    #[test]
    fn test_security__application_security__lockup__rejects_disallowed_token() {
        let mut policy = LockupPolicy::default();
        policy.allowed_tokens.insert("APPROVED".to_string());
        let mut manager = LockupManager::new(policy);

        let err = manager
            .create_period("BAD".to_string(), 90_000, 800, 250, 10)
            .unwrap_err();
        assert_eq!(err, LockupError::TokenNotAllowed("BAD".to_string()));
    }

    #[test]
    fn test_security__application_security__lockup__guards_early_exit() {
        let token = "LP-BTC".to_string();
        let mut policy = default_policy_with_token(&token);
        policy.allow_early_exit = false;
        let mut manager = LockupManager::new(policy);

        let period_id = manager
            .create_period(token.clone(), 7 * 86_400, 1200, 400, 50)
            .unwrap();

        let position_id = manager
            .stake_with_lockup("farmer2".to_string(), period_id, 500, 0)
            .unwrap();

        let attempt_timestamp = manager
            .get_position(position_id)
            .map(|p| p.start_timestamp + 86_400)
            .unwrap();

        let attempt =
            manager.unlock_position(&"farmer2".to_string(), position_id, attempt_timestamp, true);
        assert_eq!(attempt.unwrap_err(), LockupError::EarlyExitNotAllowed);

        // Allow early exit and ensure penalty applies
        let mut policy_relaxed = default_policy_with_token(&token);
        policy_relaxed.allow_early_exit = true;
        let mut relaxed_manager = LockupManager::new(policy_relaxed);
        let relaxed_period = relaxed_manager
            .create_period(token.clone(), 7 * 86_400, 1200, 400, 50)
            .unwrap();
        let relaxed_position = relaxed_manager
            .stake_with_lockup("farmer2".to_string(), relaxed_period, 500, 0)
            .unwrap();
        let settlement = relaxed_manager
            .unlock_position(&"farmer2".to_string(), relaxed_position, 86_400, true)
            .unwrap();

        assert_eq!(settlement.status, LockupStatus::EarlyExited);
        assert!(settlement.penalty_applied > 0);
        assert_eq!(settlement.reward_earned, 0);
    }

    #[test]
    fn test_extend_lockup_and_prevent_double_settlement() {
        let token = "LP-AVAX".to_string();
        let mut manager = LockupManager::new(default_policy_with_token(&token));

        let period_id = manager
            .create_period(token.clone(), 3 * 86_400, 900, 300, 25)
            .unwrap();

        let position_id = manager
            .stake_with_lockup("farmer3".to_string(), period_id, 750, 0)
            .unwrap();

        manager
            .extend_lockup(&"farmer3".to_string(), position_id, 86_400)
            .unwrap();

        let unlock_timestamp = manager.get_position(position_id).unwrap().unlock_timestamp;
        let settlement = manager
            .unlock_position(&"farmer3".to_string(), position_id, unlock_timestamp, true)
            .unwrap();
        assert_eq!(settlement.status, LockupStatus::Released);

        let second_attempt = manager.unlock_position(
            &"farmer3".to_string(),
            position_id,
            unlock_timestamp + 1,
            true,
        );
        assert_eq!(second_attempt.unwrap_err(), LockupError::PositionSettled);
    }

    #[test]
    fn test_policy_limits_enforced_for_users_and_pools() {
        let token = "LP-SOL".to_string();
        let mut policy = default_policy_with_token(&token);
        policy.max_user_locked = 1_000;
        policy.max_total_locked_per_token = 1_200;
        let mut manager = LockupManager::new(policy);

        let period_id = manager
            .create_period(token.clone(), 14 * 86_400, 1000, 200, 100)
            .unwrap();

        // First stake within limits
        manager
            .stake_with_lockup("farmer4".to_string(), period_id, 900, 0)
            .unwrap();

        // User limit exceeded
        let user_limit = manager.stake_with_lockup("farmer4".to_string(), period_id, 200, 0);
        assert_eq!(user_limit.unwrap_err(), LockupError::UserLockLimitExceeded);

        // Pool limit exceeded by another user
        let pool_limit = manager.stake_with_lockup("farmer5".to_string(), period_id, 400, 0);
        assert_eq!(
            pool_limit.unwrap_err(),
            LockupError::PoolLockLimitExceeded(token)
        );
    }
}
