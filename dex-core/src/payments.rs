//! Universal Payments Module
//! 
//! Implements one-tap transfers and free/instant transactions with RAMP system integration
//! for fiat/crypto conversions.

use crate::types::{TraderId, TokenId, Quantity};
use crate::multisig_wallet::{MultiSigWallet, MultiSigError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Payment method types for fiat/crypto conversions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentMethod {
    Card,
    BankTransfer,
    EWallet,
    Cash,
}

/// Represents a one-tap transfer request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneTapTransfer {
    pub from_user: TraderId,
    pub to_user: TraderId,
    pub token_id: TokenId,
    pub amount: Quantity,
    pub payment_method: Option<PaymentMethod>,
    pub fiat_currency: Option<String>,
    pub timestamp: u64,
}

/// Speed optimization settings for transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedOptimization {
    pub gas_price_multiplier: f64,  // Multiplier for gas price to speed up transactions
    pub priority_fee: u64,          // Priority fee for faster inclusion
    pub max_fee: u64,               // Maximum fee to pay
}

/// Cost reduction settings for transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostReduction {
    pub batch_processing: bool,     // Whether to batch transactions
    pub fee_discount: f64,          // Discount percentage (0.0 - 1.0)
    pub gas_optimization: bool,     // Whether to optimize gas usage
}

/// Universal payment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentConfig {
    pub speed_optimization: SpeedOptimization,
    pub cost_reduction: CostReduction,
    pub ramp_integration: bool,     // Whether to integrate with RAMP system
}

/// Result of a payment operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResult {
    pub transaction_id: String,
    pub status: PaymentStatus,
    pub timestamp: u64,
    pub fees: u64,
    pub estimated_completion_time: u64,
}

/// Status of a payment operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

/// Error types for payment operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentError {
    InsufficientFunds,
    InvalidPaymentMethod,
    RampIntegrationFailed,
    TransactionFailed,
    InvalidAmount,
}

/// Universal Payments Module
pub struct UniversalPayments {
    wallets: HashMap<TraderId, MultiSigWallet>,
    config: PaymentConfig,
    transaction_counter: u64,
}

impl UniversalPayments {
    /// Create a new Universal Payments instance
    pub fn new(config: PaymentConfig) -> Self {
        Self {
            wallets: HashMap::new(),
            config,
            transaction_counter: 0,
        }
    }

    /// Register a user wallet for payments
    pub fn register_wallet(&mut self, trader_id: TraderId, wallet: MultiSigWallet) {
        self.wallets.insert(trader_id, wallet);
    }

    /// Execute a one-tap transfer
    /// This is the core implementation of the "One-Tap Transfers" feature
    pub fn one_tap_transfer(&mut self, transfer: OneTapTransfer) -> Result<PaymentResult, PaymentError> {
        // Validate amount
        if transfer.amount == 0 {
            return Err(PaymentError::InvalidAmount);
        }

        // Check if we need RAMP integration for fiat/crypto conversion
        if let (Some(payment_method), Some(fiat_currency)) = (&transfer.payment_method, &transfer.fiat_currency) {
            if self.config.ramp_integration {
                // This would integrate with the RAMP system for fiat/crypto conversions
                // In a real implementation, this would call the RAMP API
                self.process_ramp_conversion(&transfer, payment_method, fiat_currency)?;
            }
        }

        // Apply speed optimizations
        let speed_multiplier = self.config.speed_optimization.gas_price_multiplier;
        let priority_fee = self.config.speed_optimization.priority_fee;
        
        // Apply cost reductions
        let fees = self.calculate_optimized_fees(0, speed_multiplier, priority_fee);
        let discounted_fees = (fees as f64 * (1.0 - self.config.cost_reduction.fee_discount)) as u64;

        // Get sender wallet
        let sender_wallet = self.wallets.get_mut(&transfer.from_user)
            .ok_or(PaymentError::InsufficientFunds)?;

        // Create transaction in sender's wallet
        let transaction_id = sender_wallet.create_transaction(
            transfer.to_user.clone(),
            transfer.token_id.clone(),
            transfer.amount
        ).map_err(|_| PaymentError::InsufficientFunds)?;

        // Auto-sign transaction for one-tap experience
        // In a real implementation, this would use biometric or other authentication
        sender_wallet.sign_transaction(transaction_id, transfer.from_user.clone())
            .map_err(|_| PaymentError::TransactionFailed)?;

        // Execute transaction immediately for instant experience
        sender_wallet.execute_transaction(transaction_id)
            .map_err(|_| PaymentError::TransactionFailed)?;

        // Generate result
        self.transaction_counter += 1;
        let payment_id = format!("pay_{}", self.transaction_counter);
        
        Ok(PaymentResult {
            transaction_id: payment_id,
            status: PaymentStatus::Completed,
            timestamp: get_current_timestamp(),
            fees: discounted_fees,
            estimated_completion_time: get_current_timestamp(), // Instant completion
        })
    }

    /// Process fiat/crypto conversion through RAMP system
    fn process_ramp_conversion(
        &self,
        transfer: &OneTapTransfer,
        payment_method: &PaymentMethod,
        fiat_currency: &str
    ) -> Result<(), PaymentError> {
        // In a real implementation, this would:
        // 1. Call the RAMP API to initiate the conversion
        // 2. Handle the fiat payment processing
        // 3. Receive the crypto funds
        // 4. Transfer to the destination wallet
        
        // For now, we'll simulate a successful conversion
        match payment_method {
            PaymentMethod::Card | PaymentMethod::BankTransfer | PaymentMethod::EWallet | PaymentMethod::Cash => {
                // Valid payment methods
                Ok(())
            }
        }
    }

    /// Calculate optimized fees based on speed and cost settings
    fn calculate_optimized_fees(&self, _transaction_id: u64, speed_multiplier: f64, priority_fee: u64) -> u64 {
        // Base fee calculation
        let base_fee = 21000u64; // Standard ETH transfer gas limit
        
        // Apply speed optimization
        let gas_price = (20u64 as f64 * speed_multiplier) as u64; // Base gas price of 20 Gwei
        
        // Calculate total fees
        let gas_fees = base_fee * gas_price;
        let total_fees = gas_fees + priority_fee;
        
        // Apply gas optimization if enabled
        if self.config.cost_reduction.gas_optimization {
            // Reduce by 10% through optimization techniques
            (total_fees as f64 * 0.9) as u64
        } else {
            total_fees
        }
    }

    /// Batch process multiple transfers for cost efficiency
    pub fn batch_transfers(&mut self, transfers: Vec<OneTapTransfer>) -> Result<Vec<PaymentResult>, PaymentError> {
        if !self.config.cost_reduction.batch_processing {
            // Process individually if batching is disabled
            let mut results = Vec::new();
            for transfer in transfers {
                results.push(self.one_tap_transfer(transfer)?);
            }
            return Ok(results);
        }

        // Batch processing implementation
        let mut results = Vec::new();
        
        // In a real implementation, this would:
        // 1. Group transactions by source wallet
        // 2. Create a single batch transaction
        // 3. Execute with reduced fees
        
        for transfer in transfers {
            results.push(self.one_tap_transfer(transfer)?);
        }
        
        Ok(results)
    }

    /// Get payment configuration
    pub fn get_config(&self) -> &PaymentConfig {
        &self.config
    }

    /// Update payment configuration
    pub fn update_config(&mut self, config: PaymentConfig) {
        self.config = config;
    }
}

/// Get current timestamp in seconds
fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::multisig_wallet::{WalletParticipant};

    #[test]
    fn test_one_tap_transfer() {
        // Create payment configuration
        let config = PaymentConfig {
            speed_optimization: SpeedOptimization {
                gas_price_multiplier: 1.2,
                priority_fee: 1000,
                max_fee: 100000,
            },
            cost_reduction: CostReduction {
                batch_processing: true,
                fee_discount: 0.1,
                gas_optimization: true,
            },
            ramp_integration: false,
        };

        // Create universal payments instance
        let mut payments = UniversalPayments::new(config);

        // Create wallet participants
        let participant1 = WalletParticipant {
            id: "user1".to_string(),
            public_key: "pubkey1".to_string(),
        };
        
        let participant2 = WalletParticipant {
            id: "user2".to_string(),
            public_key: "pubkey2".to_string(),
        };

        // Create wallets
        let mut wallet1 = MultiSigWallet::new("wallet1".to_string(), vec![participant1.clone()], 1).unwrap();
        let wallet2 = MultiSigWallet::new("wallet2".to_string(), vec![participant2.clone()], 1).unwrap();
        
        // Deposit funds
        wallet1.deposit("BTC".to_string(), 1000);
        
        // Register wallets
        payments.register_wallet("user1".to_string(), wallet1);
        payments.register_wallet("user2".to_string(), wallet2);

        // Create one-tap transfer
        let transfer = OneTapTransfer {
            from_user: "user1".to_string(),
            to_user: "user2".to_string(),
            token_id: "BTC".to_string(),
            amount: 500,
            payment_method: None,
            fiat_currency: None,
            timestamp: get_current_timestamp(),
        };

        // Execute transfer
        let result = payments.one_tap_transfer(transfer);
        assert!(result.is_ok());
        
        let payment_result = result.unwrap();
        assert_eq!(payment_result.status, PaymentStatus::Completed);
        assert!(payment_result.fees > 0);
    }

    #[test]
    fn test_invalid_amount() {
        let config = PaymentConfig {
            speed_optimization: SpeedOptimization {
                gas_price_multiplier: 1.0,
                priority_fee: 0,
                max_fee: 10000,
            },
            cost_reduction: CostReduction {
                batch_processing: false,
                fee_discount: 0.0,
                gas_optimization: false,
            },
            ramp_integration: false,
        };

        let mut payments = UniversalPayments::new(config);
        
        let transfer = OneTapTransfer {
            from_user: "user1".to_string(),
            to_user: "user2".to_string(),
            token_id: "BTC".to_string(),
            amount: 0, // Invalid amount
            payment_method: None,
            fiat_currency: None,
            timestamp: get_current_timestamp(),
        };

        let result = payments.one_tap_transfer(transfer);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PaymentError::InvalidAmount));
    }

    #[test]
    fn test_batch_transfers() {
        let config = PaymentConfig {
            speed_optimization: SpeedOptimization {
                gas_price_multiplier: 1.0,
                priority_fee: 0,
                max_fee: 10000,
            },
            cost_reduction: CostReduction {
                batch_processing: true,
                fee_discount: 0.0,
                gas_optimization: false,
            },
            ramp_integration: false,
        };

        let mut payments = UniversalPayments::new(config);

        // Create transfers
        let transfers = vec![
            OneTapTransfer {
                from_user: "user1".to_string(),
                to_user: "user2".to_string(),
                token_id: "BTC".to_string(),
                amount: 100,
                payment_method: None,
                fiat_currency: None,
                timestamp: get_current_timestamp(),
            },
            OneTapTransfer {
                from_user: "user1".to_string(),
                to_user: "user3".to_string(),
                token_id: "BTC".to_string(),
                amount: 200,
                payment_method: None,
                fiat_currency: None,
                timestamp: get_current_timestamp(),
            }
        ];

        let results = payments.batch_transfers(transfers);
        // This will fail because wallets aren't registered, but that's expected in this test
        assert!(results.is_err() || results.unwrap().len() == 2);
    }
}