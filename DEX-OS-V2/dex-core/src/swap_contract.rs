//! Smart contract style swap execution over AMM liquidity pools.
//!
//! Implements Priority 4 feature:
//! - `4,Core Trading,AMM,AMM,Smart Contracts for Swaps,Swap Execution,High`
//!
//! Provides structured swap intents with deadline and minimum-output protection,
//! and executes against the AMM `LiquidityPool` while enforcing safety checks.

use crate::amm::{AMMError, LiquidityPool};
use crate::types::{Quantity, TokenId};
use std::collections::HashMap;
use thiserror::Error;

/// Represents a swap intent guarded by deadline and minimum output constraints.
#[derive(Debug, Clone, PartialEq)]
pub struct SwapIntent {
    pub from_token: TokenId,
    pub to_token: TokenId,
    pub amount_in: Quantity,
    pub min_amount_out: Quantity,
    pub recipient: String,
    pub deadline_ms: u64,
    pub executed: bool,
}

impl SwapIntent {
    pub fn new(
        from_token: TokenId,
        to_token: TokenId,
        amount_in: Quantity,
        min_amount_out: Quantity,
        recipient: String,
        deadline_ms: u64,
    ) -> Self {
        Self {
            from_token,
            to_token,
            amount_in,
            min_amount_out,
            recipient,
            deadline_ms,
            executed: false,
        }
    }
}

/// Errors returned when executing swap intents.
#[derive(Debug, Error, PartialEq)]
pub enum SwapContractError {
    #[error("intent already executed")]
    AlreadyExecuted,
    #[error("intent expired (deadline: {deadline_ms}, now: {now_ms})")]
    DeadlineExpired { deadline_ms: u64, now_ms: u64 },
    #[error("minimum output not met (required: {min_out}, actual: {actual_out})")]
    MinOutputNotMet { min_out: Quantity, actual_out: Quantity },
    #[error("recipient mismatch")]
    InvalidRecipient,
    #[error("token pair not supported by pool")]
    UnsupportedPair,
    #[error("amm error: {0}")]
    Amm(#[from] AMMError),
}

/// Receipt of a successful swap.
#[derive(Debug, Clone, PartialEq)]
pub struct SwapReceipt {
    pub recipient: String,
    pub amount_in: Quantity,
    pub amount_out: Quantity,
    pub new_base_reserve: Quantity,
    pub new_quote_reserve: Quantity,
}

/// Contract managing swap intents and execution against a liquidity pool.
#[derive(Debug, Clone)]
pub struct SwapContract {
    pool: LiquidityPool,
    intents: HashMap<String, SwapIntent>,
}

impl SwapContract {
    /// Create a contract with an underlying liquidity pool.
    pub fn new(pool: LiquidityPool) -> Self {
        Self {
            pool,
            intents: HashMap::new(),
        }
    }

    /// Register a swap intent under an ID (e.g., hash of calldata).
    pub fn register_intent(&mut self, id: String, intent: SwapIntent) {
        self.intents.insert(id, intent);
    }

    /// Inspect a previously registered intent.
    pub fn intent(&self, id: &str) -> Option<&SwapIntent> {
        self.intents.get(id)
    }

    /// Execute a registered swap intent if all checks pass.
    pub fn execute(&mut self, id: &str, caller: &str, now_ms: u64) -> Result<SwapReceipt, SwapContractError> {
        let intent = self
            .intents
            .get_mut(id)
            .ok_or(SwapContractError::UnsupportedPair)?; // treat missing as unsupported for simplicity

        if intent.executed {
            return Err(SwapContractError::AlreadyExecuted);
        }

        if intent.recipient != caller {
            return Err(SwapContractError::InvalidRecipient);
        }

        if now_ms > intent.deadline_ms {
            return Err(SwapContractError::DeadlineExpired {
                deadline_ms: intent.deadline_ms,
                now_ms,
            });
        }

        let (base_token, quote_token) = self.pool.tokens();
        if !((intent.from_token == *base_token && intent.to_token == *quote_token)
            || (intent.from_token == *quote_token && intent.to_token == *base_token))
        {
            return Err(SwapContractError::UnsupportedPair);
        }

        // Pre-quote to ensure min output without mutating reserves on failure.
        let quoted = self
            .pool
            .quote_swap(&intent.from_token, intent.amount_in)?;
        if quoted < intent.min_amount_out {
            return Err(SwapContractError::MinOutputNotMet {
                min_out: intent.min_amount_out,
                actual_out: quoted,
            });
        }

        // Execute actual swap.
        let amount_out = self
            .pool
            .swap(&intent.from_token, intent.amount_in)
            .map_err(SwapContractError::Amm)?;

        intent.executed = true;

        let (base_reserve, quote_reserve) = self.pool.reserves();
        Ok(SwapReceipt {
            recipient: intent.recipient.clone(),
            amount_in: intent.amount_in,
            amount_out,
            new_base_reserve: base_reserve,
            new_quote_reserve: quote_reserve,
        })
    }

    /// Access the underlying pool (useful for observability in tests).
    pub fn pool(&self) -> &LiquidityPool {
        &self.pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pool() -> LiquidityPool {
        let mut pool = LiquidityPool::new("ETH".into(), "USDC".into(), 30);
        pool.add_liquidity(10_000, 20_000_000).unwrap();
        pool
    }

    fn intent(min_out: Quantity, deadline_ms: u64) -> SwapIntent {
        SwapIntent::new(
            "ETH".into(),
            "USDC".into(),
            1_000,
            min_out,
            "alice".into(),
            deadline_ms,
        )
    }

    #[test]
    fn executes_swap_with_deadline_and_min_output() {
        let mut contract = SwapContract::new(pool());
        contract.register_intent("swap-1".into(), intent(1, 10_000));

        let receipt = contract.execute("swap-1", "alice", 5_000).unwrap();
        assert_eq!(receipt.recipient, "alice");
        assert!(receipt.amount_out >= 1);
        let reserves = contract.pool().reserves();
        assert_eq!(reserves.0, 11_000); // base reserve increases by amount_in
    }

    #[test]
    fn rejects_if_min_output_not_met() {
        let mut contract = SwapContract::new(pool());
        // Set an unrealistically high min_out
        contract.register_intent("swap-2".into(), intent(2_000_000_000, 10_000));
        let err = contract.execute("swap-2", "alice", 5_000).unwrap_err();
        assert!(matches!(err, SwapContractError::MinOutputNotMet { .. }));
    }

    #[test]
    fn rejects_after_deadline() {
        let mut contract = SwapContract::new(pool());
        contract.register_intent("swap-3".into(), intent(1, 1_000));
        let err = contract.execute("swap-3", "alice", 2_000).unwrap_err();
        assert!(matches!(err, SwapContractError::DeadlineExpired { .. }));
    }

    #[test]
    fn rejects_unknown_recipient_or_pair() {
        let mut contract = SwapContract::new(pool());
        contract.register_intent("swap-4".into(), intent(1, 10_000));
        let err = contract.execute("swap-4", "bob", 5_000).unwrap_err();
        assert_eq!(err, SwapContractError::InvalidRecipient);

        let mut contract = SwapContract::new(pool());
        let bad_intent = SwapIntent::new(
            "BTC".into(),
            "USDC".into(),
            100,
            1,
            "alice".into(),
            10_000,
        );
        contract.register_intent("swap-5".into(), bad_intent);
        let err = contract.execute("swap-5", "alice", 5_000).unwrap_err();
        assert!(matches!(err, SwapContractError::UnsupportedPair));
    }

    #[test]
    fn prevents_double_execution() {
        let mut contract = SwapContract::new(pool());
        contract.register_intent("swap-6".into(), intent(1, 10_000));
        contract.execute("swap-6", "alice", 5_000).unwrap();
        let err = contract.execute("swap-6", "alice", 5_000).unwrap_err();
        assert_eq!(err, SwapContractError::AlreadyExecuted);
    }
}
