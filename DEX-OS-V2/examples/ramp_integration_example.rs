//! Example demonstrating RAMP system integration for alternative payment methods

use dex_core::payments::{
    UniversalPayments, PaymentConfig, SpeedOptimization, CostReduction, 
    OneTapTransfer, PaymentMethod
};
use dex_core::ramp_client::{RampClient, RampConfig};
use dex_core::multisig_wallet::{MultiSigWallet, WalletParticipant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create payment configuration with RAMP integration enabled
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
        ramp_integration: true, // Enable RAMP integration
    };

    // Create universal payments instance
    let mut payments = UniversalPayments::new(config);

    // Configure RAMP client for alternative payment methods
    let ramp_config = RampConfig {
        api_endpoint: "https://api.ramp.network".to_string(),
        api_key: "your_api_key_here".to_string(),
        timeout_seconds: 30,
    };
    
    let ramp_client = RampClient::new(ramp_config)?;
    payments.set_ramp_client(ramp_client);

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
    let mut wallet1 = MultiSigWallet::new("wallet1".to_string(), vec![participant1.clone()], 1)?;
    let wallet2 = MultiSigWallet::new("wallet2".to_string(), vec![participant2.clone()], 1)?;

    // Deposit funds
    wallet1.deposit("BTC".to_string(), 1000);

    // Register wallets
    payments.register_wallet("user1".to_string(), wallet1);
    payments.register_wallet("user2".to_string(), wallet2);

    // Example 1: Bank Transfer
    let bank_transfer = OneTapTransfer {
        from_user: "user1".to_string(),
        to_user: "user2".to_string(),
        token_id: "BTC".to_string(),
        amount: 100,
        payment_method: Some(PaymentMethod::BankTransfer),
        fiat_currency: Some("USD".to_string()),
        timestamp: get_current_timestamp(),
        is_nation_state_payment: false,
        compliance_metadata: None,
        biometric_verified: true, // Simulate biometric verification
    };

    match payments.one_tap_transfer(bank_transfer).await {
        Ok(result) => println!("Bank transfer result: {:?}", result),
        Err(e) => println!("Bank transfer failed: {:?}", e),
    }

    // Example 2: E-Wallet Transfer
    let ewallet_transfer = OneTapTransfer {
        from_user: "user1".to_string(),
        to_user: "user2".to_string(),
        token_id: "BTC".to_string(),
        amount: 50,
        payment_method: Some(PaymentMethod::EWallet),
        fiat_currency: Some("EUR".to_string()),
        timestamp: get_current_timestamp(),
        is_nation_state_payment: false,
        compliance_metadata: None,
        biometric_verified: true, // Simulate biometric verification
    };

    match payments.one_tap_transfer(ewallet_transfer).await {
        Ok(result) => println!("E-Wallet transfer result: {:?}", result),
        Err(e) => println!("E-Wallet transfer failed: {:?}", e),
    }

    // Example 3: Cash-Based Transfer
    let cash_transfer = OneTapTransfer {
        from_user: "user1".to_string(),
        to_user: "user2".to_string(),
        token_id: "BTC".to_string(),
        amount: 25,
        payment_method: Some(PaymentMethod::Cash),
        fiat_currency: Some("GBP".to_string()),
        timestamp: get_current_timestamp(),
        is_nation_state_payment: false,
        compliance_metadata: None,
        biometric_verified: true, // Simulate biometric verification
    };

    match payments.one_tap_transfer(cash_transfer).await {
        Ok(result) => println!("Cash transfer result: {:?}", result),
        Err(e) => println!("Cash transfer failed: {:?}", e),
    }

    println!("All RAMP integration examples completed!");
    Ok(())
}

/// Get current timestamp in seconds
fn get_current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}