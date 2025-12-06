//! RAMP Client Module
//!
//! This module provides integration with the RAMP system for fiat/crypto conversions
//! and alternative payment methods including Bank Transfers, E-Wallets, and Cash-Based Methods.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Payment method types for RAMP integration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RampPaymentMethod {
    Card,
    BankTransfer,
    EWallet,
    Cash,
}

impl ToString for RampPaymentMethod {
    fn to_string(&self) -> String {
        match self {
            RampPaymentMethod::Card => "card".to_string(),
            RampPaymentMethod::BankTransfer => "bank_transfer".to_string(),
            RampPaymentMethod::EWallet => "ewallet".to_string(),
            RampPaymentMethod::Cash => "cash".to_string(),
        }
    }
}

/// RAMP transaction status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RampTransactionStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

/// RAMP on-ramp request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnRampRequest {
    pub user_id: String,
    pub fiat_amount: f64,
    pub fiat_currency: String,
    pub crypto_currency: String,
    pub payment_method: String,
    pub payment_details: Option<HashMap<String, serde_json::Value>>,
}

/// RAMP on-ramp response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnRampResponse {
    pub transaction_id: String,
    pub status: RampTransactionStatus,
    pub crypto_amount: Option<f64>,
    pub exchange_rate: Option<f64>,
    pub fees: Option<f64>,
    pub estimated_completion_time: Option<String>,
}

/// RAMP off-ramp request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffRampRequest {
    pub user_id: String,
    pub crypto_amount: f64,
    pub crypto_currency: String,
    pub fiat_currency: String,
    pub payout_method: String,
    pub payout_details: Option<HashMap<String, serde_json::Value>>,
}

/// RAMP off-ramp response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffRampResponse {
    pub transaction_id: String,
    pub status: RampTransactionStatus,
    pub fiat_amount: Option<f64>,
    pub exchange_rate: Option<f64>,
    pub fees: Option<f64>,
    pub estimated_completion_time: Option<String>,
}

/// RAMP cross-ramp request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossRampRequest {
    pub user_id: String,
    pub source_chain: String,
    pub source_token: String,
    pub destination_chain: String,
    pub destination_token: String,
    pub amount: f64,
}

/// RAMP cross-ramp response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossRampResponse {
    pub transaction_id: String,
    pub status: RampTransactionStatus,
    pub bridge_fee: Option<f64>,
    pub estimated_gas_fee: Option<f64>,
    pub estimated_completion_time: Option<String>,
    pub source_tx_hash: Option<String>,
    pub destination_tx_hash: Option<String>,
}

/// RAMP client configuration
#[derive(Debug, Clone)]
pub struct RampConfig {
    pub api_endpoint: String,
    pub api_key: String,
    pub timeout_seconds: u64,
}

/// RAMP client errors
#[derive(Debug, Error)]
pub enum RampError {
    #[error("HTTP request failed: {0}")]
    HttpError(String),
    #[error("JSON serialization/deserialization failed: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Invalid response from RAMP API: {0}")]
    InvalidResponse(String),
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
}

/// RAMP Client for interacting with the RAMP system
pub struct RampClient {
    config: RampConfig,
    client: reqwest::Client,
}

impl RampClient {
    /// Create a new RAMP client
    pub fn new(config: RampConfig) -> Result<Self, RampError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| RampError::HttpError(e.to_string()))?;

        Ok(Self { config, client })
    }

    /// Initiate an on-ramp transaction (fiat to crypto)
    pub async fn initiate_on_ramp(&self, request: OnRampRequest) -> Result<OnRampResponse, RampError> {
        // In a real implementation, this would make an HTTP request to the RAMP API
        // For now, we'll simulate a successful response
        
        // Log the request for debugging
        println!("Initiating on-ramp transaction: {:?}", request);
        
        // Simulate API call delay
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        // Return a simulated successful response
        Ok(OnRampResponse {
            transaction_id: format!("onramp_{}", uuid::Uuid::new_v4()),
            status: RampTransactionStatus::Pending,
            crypto_amount: Some(request.fiat_amount / 20000.0), // Simulated exchange rate
            exchange_rate: Some(20000.0),
            fees: Some(10.0),
            estimated_completion_time: Some("2023-12-01T10:00:00Z".to_string()),
        })
    }

    /// Initiate an off-ramp transaction (crypto to fiat)
    pub async fn initiate_off_ramp(&self, request: OffRampRequest) -> Result<OffRampResponse, RampError> {
        // In a real implementation, this would make an HTTP request to the RAMP API
        // For now, we'll simulate a successful response
        
        // Log the request for debugging
        println!("Initiating off-ramp transaction: {:?}", request);
        
        // Simulate API call delay
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        // Return a simulated successful response
        Ok(OffRampResponse {
            transaction_id: format!("offramp_{}", uuid::Uuid::new_v4()),
            status: RampTransactionStatus::Pending,
            fiat_amount: Some(request.crypto_amount * 20000.0), // Simulated exchange rate
            exchange_rate: Some(20000.0),
            fees: Some(5.0),
            estimated_completion_time: Some("2023-12-01T10:00:00Z".to_string()),
        })
    }

    /// Initiate a cross-ramp transaction (cross-chain)
    pub async fn initiate_cross_ramp(&self, request: CrossRampRequest) -> Result<CrossRampResponse, RampError> {
        // In a real implementation, this would make an HTTP request to the RAMP API
        // For now, we'll simulate a successful response
        
        // Log the request for debugging
        println!("Initiating cross-ramp transaction: {:?}", request);
        
        // Simulate API call delay
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        // Return a simulated successful response
        Ok(CrossRampResponse {
            transaction_id: format!("crossramp_{}", uuid::Uuid::new_v4()),
            status: RampTransactionStatus::Pending,
            bridge_fee: Some(2.0),
            estimated_gas_fee: Some(50.0),
            estimated_completion_time: Some("2023-12-01T10:00:00Z".to_string()),
            source_tx_hash: Some(format!("0x{}", uuid::Uuid::new_v4().to_string()[..10].to_string())),
            destination_tx_hash: Some(format!("0x{}", uuid::Uuid::new_v4().to_string()[..10].to_string())),
        })
    }

    /// Get the status of a RAMP transaction
    pub async fn get_transaction_status(&self, transaction_id: &str) -> Result<RampTransactionStatus, RampError> {
        // In a real implementation, this would make an HTTP request to the RAMP API
        // For now, we'll simulate a successful response
        
        // Log the request for debugging
        println!("Getting status for transaction: {}", transaction_id);
        
        // Simulate API call delay
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        
        // Return a simulated status (in a real implementation, this would depend on the actual transaction)
        Ok(RampTransactionStatus::Completed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_method_to_string() {
        assert_eq!(RampPaymentMethod::Card.to_string(), "card");
        assert_eq!(RampPaymentMethod::BankTransfer.to_string(), "bank_transfer");
        assert_eq!(RampPaymentMethod::EWallet.to_string(), "ewallet");
        assert_eq!(RampPaymentMethod::Cash.to_string(), "cash");
    }

    #[tokio::test]
    async fn test_ramp_client_creation() {
        let config = RampConfig {
            api_endpoint: "https://api.ramp.network".to_string(),
            api_key: "test_key".to_string(),
            timeout_seconds: 30,
        };

        let client = RampClient::new(config);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_initiate_on_ramp() {
        let config = RampConfig {
            api_endpoint: "https://api.ramp.network".to_string(),
            api_key: "test_key".to_string(),
            timeout_seconds: 30,
        };

        let client = RampClient::new(config).unwrap();

        let request = OnRampRequest {
            user_id: "user123".to_string(),
            fiat_amount: 1000.0,
            fiat_currency: "USD".to_string(),
            crypto_currency: "BTC".to_string(),
            payment_method: "card".to_string(),
            payment_details: None,
        };

        let response = client.initiate_on_ramp(request).await;
        assert!(response.is_ok());

        let response = response.unwrap();
        assert_eq!(response.status, RampTransactionStatus::Pending);
        assert!(response.crypto_amount.is_some());
        assert!(response.exchange_rate.is_some());
        assert!(response.fees.is_some());
    }

    #[tokio::test]
    async fn test_initiate_off_ramp() {
        let config = RampConfig {
            api_endpoint: "https://api.ramp.network".to_string(),
            api_key: "test_key".to_string(),
            timeout_seconds: 30,
        };

        let client = RampClient::new(config).unwrap();

        let request = OffRampRequest {
            user_id: "user123".to_string(),
            crypto_amount: 0.5,
            crypto_currency: "BTC".to_string(),
            fiat_currency: "USD".to_string(),
            payout_method: "bank".to_string(),
            payout_details: None,
        };

        let response = client.initiate_off_ramp(request).await;
        assert!(response.is_ok());

        let response = response.unwrap();
        assert_eq!(response.status, RampTransactionStatus::Pending);
        assert!(response.fiat_amount.is_some());
        assert!(response.exchange_rate.is_some());
        assert!(response.fees.is_some());
    }

    #[tokio::test]
    async fn test_initiate_cross_ramp() {
        let config = RampConfig {
            api_endpoint: "https://api.ramp.network".to_string(),
            api_key: "test_key".to_string(),
            timeout_seconds: 30,
        };

        let client = RampClient::new(config).unwrap();

        let request = CrossRampRequest {
            user_id: "user123".to_string(),
            source_chain: "ethereum".to_string(),
            source_token: "ETH".to_string(),
            destination_chain: "polygon".to_string(),
            destination_token: "MATIC".to_string(),
            amount: 1.0,
        };

        let response = client.initiate_cross_ramp(request).await;
        assert!(response.is_ok());

        let response = response.unwrap();
        assert_eq!(response.status, RampTransactionStatus::Pending);
        assert!(response.bridge_fee.is_some());
        assert!(response.estimated_gas_fee.is_some());
        assert!(response.source_tx_hash.is_some());
        assert!(response.destination_tx_hash.is_some());
    }

    #[tokio::test]
    async fn test_get_transaction_status() {
        let config = RampConfig {
            api_endpoint: "https://api.ramp.network".to_string(),
            api_key: "test_key".to_string(),
            timeout_seconds: 30,
        };

        let client = RampClient::new(config).unwrap();

        let status = client.get_transaction_status("test_transaction_id").await;
        assert!(status.is_ok());

        let status = status.unwrap();
        assert_eq!(status, RampTransactionStatus::Completed);
    }
}