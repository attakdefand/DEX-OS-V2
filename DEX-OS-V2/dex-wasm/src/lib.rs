//! WebAssembly interface for the DEX-OS core engine
//!
//! This module provides the WASM bindings to allow the DEX-OS core engine
//! to be used in web browsers and other WASM environments.

use dex_core::{
    amm::ConstantProductAMM,
    iot_wallet::{IoTDeviceProfile, IoTWalletRuntime},
    orderbook::OrderBook,
    types::Order,
    wallet::{SessionManager, WalletError, WalletSigner},
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

pub mod mobile_wallet;
pub mod mobile_session_manager;

/// WASM wrapper for the OrderBook
#[wasm_bindgen]
pub struct WasmOrderBook {
    inner: OrderBook,
}

#[wasm_bindgen]
impl WasmOrderBook {
    /// Create a new orderbook
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmOrderBook {
        WasmOrderBook {
            inner: OrderBook::new(),
        }
    }

    /// Add an order to the orderbook
    #[wasm_bindgen]
    pub fn add_order(&mut self, order: JsValue) -> Result<JsValue, JsValue> {
        let order: Order = serde_wasm_bindgen::from_value(order)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize order: {}", e)))?;

        match self.inner.add_order(order) {
            Ok(trades) => {
                // Convert trades to JsValue
                let js_trades = serde_wasm_bindgen::to_value(&trades).map_err(|e| {
                    JsValue::from_str(&format!("Failed to serialize trades: {}", e))
                })?;
                Ok(js_trades)
            }
            Err(e) => Err(JsValue::from_str(&format!("Failed to add order: {}", e))),
        }
    }

    /// Get the best bid price
    #[wasm_bindgen]
    pub fn best_bid(&self) -> Option<u64> {
        self.inner.best_bid()
    }

    /// Get the best ask price
    #[wasm_bindgen]
    pub fn best_ask(&self) -> Option<u64> {
        self.inner.best_ask()
    }

    /// Remove an order from the orderbook
    #[wasm_bindgen]
    pub fn remove_order(&mut self, order_id: u64) -> Result<JsValue, JsValue> {
        match self.inner.remove_order(order_id) {
            Ok(order) => {
                let js_order = serde_wasm_bindgen::to_value(&order)
                    .map_err(|e| JsValue::from_str(&format!("Failed to serialize order: {}", e)))?;
                Ok(js_order)
            }
            Err(e) => Err(JsValue::from_str(&format!("Failed to remove order: {}", e))),
        }
    }

    /// Lookup an order by its ID
    #[wasm_bindgen]
    pub fn get_order(&self, order_id: u64) -> Result<JsValue, JsValue> {
        match self.inner.get_order(order_id) {
            Some(order) => {
                let js_order = serde_wasm_bindgen::to_value(order)
                    .map_err(|e| JsValue::from_str(&format!("Failed to serialize order: {}", e)))?;
                Ok(js_order)
            }
            None => Err(JsValue::from_str("Order not found")),
        }
    }

    /// Generate a Merkle proof for a batch of orders
    #[wasm_bindgen]
    pub fn generate_batch_proof(&self, order_ids: JsValue) -> Result<JsValue, JsValue> {
        let order_ids: Vec<u64> = serde_wasm_bindgen::from_value(order_ids)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize order IDs: {}", e)))?;

        match self.inner.generate_batch_proof(&order_ids) {
            Some(proof) => {
                let js_proof = serde_wasm_bindgen::to_value(&proof)
                    .map_err(|e| JsValue::from_str(&format!("Failed to serialize proof: {}", e)))?;
                Ok(js_proof)
            }
            None => Err(JsValue::from_str("Failed to generate batch proof")),
        }
    }
}

/// WASM wrapper for the ConstantProductAMM
#[wasm_bindgen]
pub struct WasmAMM {
    inner: ConstantProductAMM,
}

#[wasm_bindgen]
impl WasmAMM {
    /// Create a new AMM with the specified fee (in basis points)
    #[wasm_bindgen(constructor)]
    pub fn new(fee: u32) -> WasmAMM {
        WasmAMM {
            inner: ConstantProductAMM::new(fee),
        }
    }

    /// Add liquidity to the pool
    #[wasm_bindgen]
    pub fn add_liquidity(
        &mut self,
        token_a: String,
        amount_a: u64,
        token_b: String,
        amount_b: u64,
    ) -> Result<u64, JsValue> {
        self.inner
            .add_liquidity(token_a, amount_a, token_b, amount_b)
            .map_err(|e| JsValue::from_str(&format!("Failed to add liquidity: {}", e)))
    }

    /// Remove liquidity from the pool
    #[wasm_bindgen]
    pub fn remove_liquidity(
        &mut self,
        token_a: String,
        token_b: String,
        liquidity_tokens: u64,
    ) -> Result<JsValue, JsValue> {
        match self
            .inner
            .remove_liquidity(token_a, token_b, liquidity_tokens)
        {
            Ok((amount_a, amount_b)) => {
                let result = serde_json::json!({
                    "amount_a": amount_a,
                    "amount_b": amount_b
                });
                let js_result = serde_wasm_bindgen::to_value(&result).map_err(|e| {
                    JsValue::from_str(&format!("Failed to serialize result: {}", e))
                })?;
                Ok(js_result)
            }
            Err(e) => Err(JsValue::from_str(&format!(
                "Failed to remove liquidity: {}",
                e
            ))),
        }
    }

    /// Swap tokens in the pool
    #[wasm_bindgen]
    pub fn swap(
        &mut self,
        from_token: String,
        to_token: String,
        amount_in: u64,
    ) -> Result<u64, JsValue> {
        self.inner
            .swap(from_token, to_token, amount_in)
            .map_err(|e| JsValue::from_str(&format!("Failed to swap: {}", e)))
    }

    /// Get the price of one token in terms of another
    #[wasm_bindgen]
    pub fn get_price(&self, from_token: String, to_token: String) -> Result<f64, JsValue> {
        self.inner
            .get_price(&from_token, &to_token)
            .map_err(|e| JsValue::from_str(&format!("Failed to get price: {}", e)))
    }

    /// Find the optimal price within a given range using binary search
    #[wasm_bindgen]
    pub fn find_price_in_range(
        &self,
        from_token: String,
        to_token: String,
        min_price: f64,
        max_price: f64,
        tolerance: f64,
    ) -> Result<f64, JsValue> {
        self.inner
            .find_price_in_range(&from_token, &to_token, min_price, max_price, tolerance)
            .map_err(|e| JsValue::from_str(&format!("Failed to find price in range: {}", e)))
    }

    /// Check if a given price is within acceptable slippage range
    #[wasm_bindgen]
    pub fn is_price_within_slippage(
        &self,
        from_token: String,
        to_token: String,
        proposed_price: f64,
        max_slippage: f64,
    ) -> Result<bool, JsValue> {
        self.inner
            .is_price_within_slippage(&from_token, &to_token, proposed_price, max_slippage)
            .map_err(|e| {
                JsValue::from_str(&format!("Failed to check price within slippage: {}", e))
            })
    }
}

/// Wallet signer wrapper for Ethereum-compatible signing and session helpers.
#[wasm_bindgen]
pub struct WasmWalletSigner {
    inner: WalletSigner,
}

#[wasm_bindgen]
impl WasmWalletSigner {
    /// Create a signer from a 0x-prefixed private key.
    #[wasm_bindgen(constructor)]
    pub fn new(private_key: String) -> Result<WasmWalletSigner, JsValue> {
        let inner = WalletSigner::from_private_key_hex(&private_key).map_err(wallet_error_to_js)?;
        Ok(WasmWalletSigner { inner })
    }

    /// Return the normalized 0x-prefixed address.
    #[wasm_bindgen]
    pub fn address(&self) -> String {
        self.inner.address().to_string()
    }

    /// Sign an arbitrary message using the Ethereum personal_sign format.
    #[wasm_bindgen]
    pub fn sign_message(&self, message: String) -> Result<String, JsValue> {
        self.inner
            .sign_personal_message(&message)
            .map_err(wallet_error_to_js)
    }
}

/// Verify a personal_sign signature against an address.
#[wasm_bindgen]
pub fn verify_wallet_signature(
    address: String,
    message: String,
    signature: String,
) -> Result<(), JsValue> {
    WalletSigner::verify_personal_message(&address, &message, &signature)
        .map_err(wallet_error_to_js)
}

/// Session token manager exposed to WASM.
#[wasm_bindgen]
pub struct WasmSessionManager {
    inner: SessionManager,
}

#[wasm_bindgen]
impl WasmSessionManager {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmSessionManager {
        WasmSessionManager {
            inner: SessionManager::new(),
        }
    }

    /// Issue a short-lived session token for an address.
    #[wasm_bindgen]
    pub fn issue_session(&mut self, address: String, ttl_seconds: u64) -> Result<JsValue, JsValue> {
        let session = self
            .inner
            .issue_session(&address, ttl_seconds)
            .map_err(wallet_error_to_js)?;

        serde_wasm_bindgen::to_value(&session)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize session: {}", e)))
    }

    /// Validate that a session token is present and unexpired.
    #[wasm_bindgen]
    pub fn validate_session(&self, address: String, token: String) -> Result<(), JsValue> {
        self.inner
            .validate_session(&address, &token)
            .map_err(wallet_error_to_js)
    }
}

/// WASM wrapper for the IoT wallet runtime (Layer 20 security)
#[wasm_bindgen]
pub struct WasmIoTWalletRuntime {
    inner: IoTWalletRuntime,
}

#[wasm_bindgen]
impl WasmIoTWalletRuntime {
    /// Create a new IoT wallet runtime with default security windows
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmIoTWalletRuntime {
        WasmIoTWalletRuntime {
            inner: IoTWalletRuntime::new(),
        }
    }

    /// Register an IoT device profile (public key, firmware, segments, operations)
    #[wasm_bindgen]
    pub fn register_device(&mut self, profile: JsValue) -> Result<(), JsValue> {
        let profile: IoTDeviceProfile = serde_wasm_bindgen::from_value(profile).map_err(|e| {
            JsValue::from_str(&format!("Failed to deserialize IoT device profile: {}", e))
        })?;

        self.inner
            .register_device(profile)
            .map_err(|e| JsValue::from_str(&format!("Failed to register device: {}", e)))
    }

    /// Issue an authentication challenge for a device
    #[wasm_bindgen]
    pub fn issue_challenge(&mut self, device_id: String) -> Result<JsValue, JsValue> {
        match self.inner.issue_challenge(&device_id) {
            Ok(challenge) => {
                serde_wasm_bindgen::to_value(&challenge).map_err(|e| {
                    JsValue::from_str(&format!("Failed to serialize challenge: {}", e))
                })
            },
            Err(e) => Err(JsValue::from_str(&format!("Failed to issue challenge: {}", e)))
        }
    }

    /// Verify the device's signed response and mint a short-lived session
    #[wasm_bindgen]
    pub fn verify_response(
        &mut self,
        device_id: String,
        signature: Vec<u8>,
    ) -> Result<JsValue, JsValue> {
        match self.inner.verify_challenge_response(&device_id, signature) {
            Ok(session) => {
                serde_wasm_bindgen::to_value(&session)
                    .map_err(|e| JsValue::from_str(&format!("Failed to serialize session: {}", e)))
            },
            Err(e) => Err(JsValue::from_str(&format!("Failed to verify response: {}", e)))
        }
    }

    /// Record a heartbeat from a device
    #[wasm_bindgen]
    pub fn record_heartbeat(
        &mut self,
        device_id: String,
        timestamp: u64,
    ) -> Result<JsValue, JsValue> {
        match self.inner.record_heartbeat(&device_id, timestamp) {
            Ok(status) => {
                serde_wasm_bindgen::to_value(&status)
                    .map_err(|e| JsValue::from_str(&format!("Failed to serialize status: {}", e)))
            },
            Err(e) => Err(JsValue::from_str(&format!("Failed to record heartbeat: {}", e)))
        }
    }

    /// Produce a risk assessment for the device (stale heartbeat, stale attestation, failures)
    #[wasm_bindgen]
    pub fn risk_report(&self, device_id: String) -> Result<JsValue, JsValue> {
        match self.inner.assess_risk(&device_id) {
            Ok(assessment) => {
                serde_wasm_bindgen::to_value(&assessment).map_err(|e| {
                    JsValue::from_str(&format!("Failed to serialize risk assessment: {}", e))
                })
            },
            Err(e) => Err(JsValue::from_str(&format!("Failed to assess risk: {}", e)))
        }
    }
}

/// WASM wrapper for mobile wallet functionality with enhanced security
#[wasm_bindgen]
pub struct WasmMobileWallet {
    inner: WalletSigner,
    session_manager: SessionManager,
    device_info: Option<String>, // Serialized MobileDeviceInfo
    security_context: MobileSecurityContext,
}

/// Security context for mobile wallet operations
#[derive(Serialize, Deserialize, Clone)]
pub struct MobileSecurityContext {
    pub encryption_key: String, // In a real implementation, this would be a proper key
    pub biometric_required: bool,
    pub pin_required: bool,
    pub failed_attempts: u32,
    pub last_failed_attempt: Option<f64>, // JavaScript timestamp
    pub lockout_until: Option<f64>, // JavaScript timestamp
}

/// Device information for mobile integration
#[derive(Serialize, Deserialize, Clone)]
pub struct MobileDeviceInfo {
    pub device_id: String,
    pub platform: String,  // "iOS" or "Android"
    pub os_version: String,
    pub app_version: String,
    pub model: String,
    pub security_features: Vec<String>, // Security features of the device
}

#[wasm_bindgen]
impl WasmMobileWallet {
    /// Create a new mobile wallet signer from a private key
    #[wasm_bindgen(constructor)]
    pub fn new(private_key: String) -> Result<WasmMobileWallet, JsValue> {
        let inner = WalletSigner::from_private_key_hex(&private_key).map_err(wallet_error_to_js)?;
        
        // Initialize security context
        let security_context = MobileSecurityContext {
            encryption_key: "default_key".to_string(), // In a real implementation, this would be a proper key
            biometric_required: true,
            pin_required: true,
            failed_attempts: 0,
            last_failed_attempt: None,
            lockout_until: None,
        };
        
        Ok(WasmMobileWallet {
            inner,
            session_manager: SessionManager::new(),
            device_info: None,
            security_context,
        })
    }

    /// Set device information for mobile tracking
    #[wasm_bindgen]
    pub fn set_device_info(&mut self, device_info: JsValue) -> Result<(), JsValue> {
        let info: MobileDeviceInfo = serde_wasm_bindgen::from_value(device_info)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize device info: {}", e)))?;
        let serialized = serde_json::to_string(&info)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize device info: {}", e)))?;
        self.device_info = Some(serialized);
        Ok(())
    }

    /// Get the wallet address
    #[wasm_bindgen]
    pub fn address(&self) -> String {
        self.inner.address().to_string()
    }

    /// Sign a message with mobile-specific metadata and security checks
    #[wasm_bindgen]
    pub fn sign_message(&self, message: String, security_options: JsValue) -> Result<String, JsValue> {
        // Check if the wallet is locked out
        if let Some(lockout_until) = self.security_context.lockout_until {
            if js_sys::Date::now() < lockout_until {
                return Err(JsValue::from_str("Wallet is temporarily locked due to failed attempts"));
            }
        }
        
        // Parse security options
        let options: SecurityOptions = serde_wasm_bindgen::from_value(security_options)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse security options: {}", e)))?;
            
        // Check if biometric verification is required and provided
        if self.security_context.biometric_required && !options.biometric_verified {
            return Err(JsValue::from_str("Biometric verification required"));
        }
        
        // Check if PIN is required and provided
        if self.security_context.pin_required && options.pin.is_none() {
            return Err(JsValue::from_str("PIN required"));
        }
        
        // In a real implementation, we would verify the PIN here
        // For now, we'll just check if it's provided when required
        if self.security_context.pin_required {
            if let Some(ref pin) = options.pin {
                // In a real implementation, we would verify the PIN against a stored hash
                if pin.len() < 4 {
                    return Err(JsValue::from_str("PIN must be at least 4 digits"));
                }
            }
        }
        
        // Add security context to the message
        let secured_message = format!("{}|security_level:high|timestamp:{}", 
            message, 
            js_sys::Date::now());
            
        self.inner
            .sign_personal_message(&secured_message)
            .map_err(wallet_error_to_js)
    }

    /// Issue a mobile session with TTL and security context
    #[wasm_bindgen]
    pub fn issue_mobile_session(&mut self, ttl_seconds: u64) -> Result<JsValue, JsValue> {
        let session = self
            .session_manager
            .issue_session(self.inner.address(), ttl_seconds)
            .map_err(wallet_error_to_js)?;

        // Create enhanced session with device info and security context
        let device_info_obj: Option<MobileDeviceInfo> = self.device_info.as_ref().and_then(|s| {
            serde_json::from_str(s).ok()
        });
        
        let enhanced_session = serde_json::json!({
            "session": session,
            "device_info": device_info_obj,
            "security_context": self.security_context,
            "issued_at": js_sys::Date::now()
        });

        serde_wasm_bindgen::to_value(&enhanced_session)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize session: {}", e)))
    }

    /// Validate a mobile session with security checks
    #[wasm_bindgen]
    pub fn validate_mobile_session(&mut self, token: String, security_challenge: JsValue) -> Result<bool, JsValue> {
        // Check if the wallet is locked out
        if let Some(lockout_until) = self.security_context.lockout_until {
            if js_sys::Date::now() < lockout_until {
                return Err(JsValue::from_str("Wallet is temporarily locked due to failed attempts"));
            }
        }
        
        // Parse security challenge
        let challenge: SecurityChallenge = serde_wasm_bindgen::from_value(security_challenge)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse security challenge: {}", e)))?;
            
        // Verify the security challenge
        if !self.verify_security_challenge(&challenge)? {
            // Increment failed attempts
            self.security_context.failed_attempts += 1;
            self.security_context.last_failed_attempt = Some(js_sys::Date::now());
            
            // Implement lockout logic after 3 failed attempts
            if self.security_context.failed_attempts >= 3 {
                let lockout_duration = 300000.0; // 5 minutes in milliseconds
                self.security_context.lockout_until = Some(js_sys::Date::now() + lockout_duration);
                return Err(JsValue::from_str("Too many failed attempts. Wallet locked for 5 minutes."));
            }
            
            return Ok(false);
        }
        
        // Reset failed attempts on successful verification
        self.security_context.failed_attempts = 0;
        self.security_context.last_failed_attempt = None;
        self.security_context.lockout_until = None;

        match self.session_manager.validate_session(self.inner.address(), &token) {
            Ok(_) => Ok(true),
            Err(WalletError::SessionExpired) => Ok(false),
            Err(WalletError::SessionNotFound) => Ok(false),
            Err(e) => Err(wallet_error_to_js(e)),
        }
    }
    
    /// Verify a security challenge
    fn verify_security_challenge(&self, challenge: &SecurityChallenge) -> Result<bool, JsValue> {
        // In a real implementation, this would verify biometric data, PIN, etc.
        // For now, we'll just check if the required fields are present
        
        if self.security_context.biometric_required && !challenge.biometric_verified {
            return Ok(false);
        }
        
        if self.security_context.pin_required && challenge.pin.is_none() {
            return Ok(false);
        }
        
        // In a real implementation, we would verify the PIN and biometric data properly
        Ok(true)
    }

    /// Perform a secure mobile transaction with comprehensive security checks
    #[wasm_bindgen]
    pub fn secure_mobile_transaction(&mut self, transaction_data: String, security_options: JsValue) -> Result<String, JsValue> {
        // Check if the wallet is locked out
        if let Some(lockout_until) = self.security_context.lockout_until {
            if js_sys::Date::now() < lockout_until {
                return Err(JsValue::from_str("Wallet is temporarily locked due to failed attempts"));
            }
        }
        
        // Parse security options
        let options: SecurityOptions = serde_wasm_bindgen::from_value(security_options)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse security options: {}", e)))?;
            
        // Check if biometric verification is required and provided
        if self.security_context.biometric_required && !options.biometric_verified {
            // Increment failed attempts
            self.security_context.failed_attempts += 1;
            self.security_context.last_failed_attempt = Some(js_sys::Date::now());
            
            // Implement lockout logic after 3 failed attempts
            if self.security_context.failed_attempts >= 3 {
                let lockout_duration = 300000.0; // 5 minutes in milliseconds
                self.security_context.lockout_until = Some(js_sys::Date::now() + lockout_duration);
                return Err(JsValue::from_str("Biometric verification required. Too many failed attempts. Wallet locked for 5 minutes."));
            }
            
            return Err(JsValue::from_str("Biometric verification required for mobile transactions"));
        }
        
        // Check if PIN is required and provided
        if self.security_context.pin_required && options.pin.is_none() {
            return Err(JsValue::from_str("PIN required for mobile transactions"));
        }
        
        // In a real implementation, we would verify the PIN here
        // For now, we'll just check if it's provided when required
        if self.security_context.pin_required {
            if let Some(ref pin) = options.pin {
                // In a real implementation, we would verify the PIN against a stored hash
                if pin.len() < 4 {
                    return Err(JsValue::from_str("PIN must be at least 4 digits"));
                }
            }
        }
        
        // Reset failed attempts on successful verification
        self.security_context.failed_attempts = 0;
        self.security_context.last_failed_attempt = None;
        self.security_context.lockout_until = None;

        // Add device context to transaction
        let transaction_with_context = if let Some(ref serialized_device_info) = self.device_info {
            // Deserialize the device info
            match serde_json::from_str::<MobileDeviceInfo>(serialized_device_info) {
                Ok(device_info) => {
                    format!("{}|device:{}|platform:{}|app:{}|security_level:high|timestamp:{}", 
                        transaction_data, 
                        device_info.device_id, 
                        device_info.platform, 
                        device_info.app_version,
                        js_sys::Date::now())
                },
                Err(_) => format!("{}|security_level:high|timestamp:{}", transaction_data, js_sys::Date::now()) // If deserialization fails, use transaction data as is
            }
        } else {
            format!("{}|security_level:high|timestamp:{}", transaction_data, js_sys::Date::now())
        };

        // Sign the transaction
        self.inner
            .sign_personal_message(&transaction_with_context)
            .map_err(wallet_error_to_js)
    }
    
    /// iOS-specific method for integrating with Apple Wallet with enhanced security
    #[wasm_bindgen]
    pub fn ios_apple_wallet_integration(&self, payment_data: JsValue, security_options: JsValue) -> Result<JsValue, JsValue> {
        // Parse payment data
        let payment_info: serde_json::Value = serde_wasm_bindgen::from_value(payment_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse payment data: {}", e)))?;
            
        // Parse security options
        let options: SecurityOptions = serde_wasm_bindgen::from_value(security_options)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse security options: {}", e)))?;
            
        // Check if biometric verification is required and provided
        if self.security_context.biometric_required && !options.biometric_verified {
            return Err(JsValue::from_str("Biometric verification required for Apple Pay integration"));
        }
        
        // Add iOS-specific metadata
        let device_info_obj: Option<MobileDeviceInfo> = self.device_info.as_ref().and_then(|s| {
            serde_json::from_str(s).ok()
        });
        
        let ios_payment_data = serde_json::json!({
            "payment": payment_info,
            "platform": "iOS",
            "device_info": device_info_obj,
            "security_context": self.security_context,
            "timestamp": js_sys::Date::now(),
            "wallet_type": "Apple Wallet"
        });
        
        // In a real implementation, this would integrate with Apple Pay APIs
        // For now, we'll just return the enhanced data
        serde_wasm_bindgen::to_value(&ios_payment_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize iOS payment data: {}", e)))
    }
    
    /// Android-specific method for integrating with Google Pay with enhanced security
    #[wasm_bindgen]
    pub fn android_google_pay_integration(&self, payment_data: JsValue, security_options: JsValue) -> Result<JsValue, JsValue> {
        // Parse payment data
        let payment_info: serde_json::Value = serde_wasm_bindgen::from_value(payment_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse payment data: {}", e)))?;
            
        // Parse security options
        let options: SecurityOptions = serde_wasm_bindgen::from_value(security_options)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse security options: {}", e)))?;
            
        // Check if biometric verification is required and provided
        if self.security_context.biometric_required && !options.biometric_verified {
            return Err(JsValue::from_str("Biometric verification required for Google Pay integration"));
        }
            
        // Add Android-specific metadata
        let device_info_obj: Option<MobileDeviceInfo> = self.device_info.as_ref().and_then(|s| {
            serde_json::from_str(s).ok()
        });
        
        let android_payment_data = serde_json::json!({
            "payment": payment_info,
            "platform": "Android",
            "device_info": device_info_obj,
            "security_context": self.security_context,
            "timestamp": js_sys::Date::now(),
            "wallet_type": "Google Pay"
        });
        
        // In a real implementation, this would integrate with Google Pay APIs
        // For now, we'll just return the enhanced data
        serde_wasm_bindgen::to_value(&android_payment_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize Android payment data: {}", e)))
    }
    
    /// Update security settings
    #[wasm_bindgen]
    pub fn update_security_settings(&mut self, settings: JsValue) -> Result<(), JsValue> {
        let new_settings: MobileSecurityContext = serde_wasm_bindgen::from_value(settings)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse security settings: {}", e)))?;
            
        self.security_context = new_settings;
        Ok(())
    }
    
    /// Get current security status
    #[wasm_bindgen]
    pub fn get_security_status(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.security_context)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize security context: {}", e)))
    }
}

/// Security options for mobile operations
#[derive(Serialize, Deserialize)]
pub struct SecurityOptions {
    pub biometric_verified: bool,
    pub pin: Option<String>,
    pub location_verified: bool,
}

/// Security challenge for session validation
#[derive(Serialize, Deserialize)]
pub struct SecurityChallenge {
    pub biometric_verified: bool,
    pub pin: Option<String>,
    pub timestamp: f64,
}

/// Mobile-specific session manager with enhanced security features
#[wasm_bindgen]
pub struct WasmMobileSessionManager {
    inner: SessionManager,
    active_mobile_sessions: std::collections::HashMap<String, MobileSessionInfo>,
    session_timeouts: std::collections::HashMap<String, f64>, // token -> timeout timestamp
    security_policies: SecurityPolicies,
}

/// Security policies for mobile sessions
#[derive(Serialize, Deserialize, Clone)]
pub struct SecurityPolicies {
    pub max_sessions_per_device: usize,
    pub session_timeout_minutes: u64,
    pub require_biometric_on_critical_ops: bool,
    pub enable_geofencing: bool,
    pub allowed_countries: Vec<String>,
}

/// Enhanced session information for mobile devices
#[derive(Serialize, Deserialize, Clone)]
pub struct MobileSessionInfo {
    pub session: String, // Serialized WalletSession
    pub device_info: Option<String>, // Serialized MobileDeviceInfo
    pub last_activity: f64,  // JavaScript timestamp
    pub location_coordinates: Option<(f64, f64)>,  // lat, lon
    pub security_flags: Vec<String>, // Security-related flags
    pub encryption_key: String, // Session-specific encryption key
    pub permissions: Vec<String>, // Permissions granted to this session
}

#[wasm_bindgen]
impl WasmMobileSessionManager {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmMobileSessionManager {
        let security_policies = SecurityPolicies {
            max_sessions_per_device: 3,
            session_timeout_minutes: 15,
            require_biometric_on_critical_ops: true,
            enable_geofencing: false,
            allowed_countries: vec!["US".to_string(), "CA".to_string(), "GB".to_string()],
        };
        
        WasmMobileSessionManager {
            inner: SessionManager::new(),
            active_mobile_sessions: std::collections::HashMap::new(),
            session_timeouts: std::collections::HashMap::new(),
            security_policies,
        }
    }

    /// Issue a mobile session with device context and enhanced security
    #[wasm_bindgen]
    pub fn issue_mobile_session(&mut self, address: String, ttl_seconds: u64, device_info: JsValue) -> Result<JsValue, JsValue> {
        let session = self
            .inner
            .issue_session(&address, ttl_seconds)
            .map_err(wallet_error_to_js)?;

        let device: Option<MobileDeviceInfo> = match serde_wasm_bindgen::from_value(device_info) {
            Ok(d) => Some(d),
            Err(_) => None, // If device info is not provided or invalid, continue without it
        };
        
        // Check if we're exceeding the maximum sessions per device
        if let Some(ref device) = device {
            let device_sessions = self.count_device_sessions(&device.device_id);
            if device_sessions >= self.security_policies.max_sessions_per_device {
                return Err(JsValue::from_str("Maximum sessions per device exceeded"));
            }
        }

        // Serialize the session for storage
        let session_json = serde_json::to_string(&session)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize session: {}", e)))?;

        // Serialize the device info
        let serialized_device_info: Option<String> = device.as_ref().map(|d| {
            serde_json::to_string(d).unwrap_or_else(|_| "{}".to_string())
        });

        // Generate session-specific encryption key
        let encryption_key = self.generate_session_key();
        
        let security_flags = vec![
            "device_bound".to_string(),
            "time_limited".to_string(),
            "activity_monitored".to_string(),
            "encrypted".to_string()
        ];
        
        let permissions = vec![
            "read_balance".to_string(),
            "view_transactions".to_string()
        ];

        let session_info = MobileSessionInfo {
            session: session_json,
            device_info: serialized_device_info,
            last_activity: js_sys::Date::now(),
            location_coordinates: None,
            security_flags,
            encryption_key,
            permissions,
        };

        // Set timeout
        let timeout = js_sys::Date::now() + (ttl_seconds as f64) * 1000.0;
        self.session_timeouts.insert(session.token.clone(), timeout);
        self.active_mobile_sessions.insert(session.token.clone(), session_info);

        serde_wasm_bindgen::to_value(&session)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize session: {}", e)))
    }
    
    /// Generate a session-specific encryption key
    fn generate_session_key(&self) -> String {
        // In a real implementation, this would generate a proper cryptographic key
        // For now, we'll use a placeholder
        format!("session_key_{}", js_sys::Date::now())
    }
    
    /// Count active sessions for a device
    fn count_device_sessions(&self, device_id: &str) -> usize {
        self.active_mobile_sessions.values().filter(|session_info| {
            if let Some(ref serialized_device_info) = session_info.device_info {
                if let Ok(device_info) = serde_json::from_str::<MobileDeviceInfo>(serialized_device_info) {
                    return device_info.device_id == device_id;
                }
            }
            false
        }).count()
    }

    /// Validate mobile session with activity tracking and timeout checking
    #[wasm_bindgen]
    pub fn validate_mobile_session(&mut self, address: String, token: String, security_context: JsValue) -> Result<bool, JsValue> {
        // Parse security context
        let context: SessionSecurityContext = serde_wasm_bindgen::from_value(security_context)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse security context: {}", e)))?;
            
        // Check if session has timed out
        if let Some(timeout) = self.session_timeouts.get(&token) {
            if &js_sys::Date::now() > timeout {
                self.active_mobile_sessions.remove(&token);
                self.session_timeouts.remove(&token);
                return Ok(false);
            }
        }
        
        // Check geofencing if enabled
        if self.security_policies.enable_geofencing {
            if let Some(session_info) = self.active_mobile_sessions.get(&token) {
                if let Some((lat, lon)) = session_info.location_coordinates {
                    if !self.is_location_allowed(lat, lon, &context.country_code.unwrap_or_default()) {
                        return Ok(false);
                    }
                }
            }
        }

        match self.inner.validate_session(&address, &token) {
            Ok(_) => {
                // Update last activity
                if let Some(session_info) = self.active_mobile_sessions.get_mut(&token) {
                    session_info.last_activity = js_sys::Date::now();
                }
                Ok(true)
            },
            Err(WalletError::SessionExpired) => {
                self.active_mobile_sessions.remove(&token);
                self.session_timeouts.remove(&token);
                Ok(false)
            },
            Err(WalletError::SessionNotFound) => Ok(false),
            Err(e) => Err(wallet_error_to_js(e)),
        }
    }
    
    /// Check if a location is allowed based on security policies
    fn is_location_allowed(&self, _latitude: f64, _longitude: f64, country_code: &str) -> bool {
        // In a real implementation, this would check the location against allowed countries
        // and potentially other geofencing rules
        if !country_code.is_empty() {
            self.security_policies.allowed_countries.contains(&country_code.to_string())
        } else {
            // If no country code is provided, we'll assume it's allowed for now
            true
        }
    }

    /// Update session location for geofencing
    #[wasm_bindgen]
    pub fn update_session_location(&mut self, token: String, latitude: f64, longitude: f64, country_code: String) -> Result<(), JsValue> {
        if let Some(session_info) = self.active_mobile_sessions.get_mut(&token) {
            session_info.location_coordinates = Some((latitude, longitude));
            
            // Check if the location is allowed
            if self.security_policies.enable_geofencing && 
               !self.is_location_allowed(latitude, longitude, &country_code) {
                // Revoke the session if location is not allowed
                self.active_mobile_sessions.remove(&token);
                self.session_timeouts.remove(&token);
                return Err(JsValue::from_str("Session revoked due to location restriction"));
            }
            
            Ok(())
        } else {
            Err(JsValue::from_str("Session not found"))
        }
    }

    /// Get session information
    #[wasm_bindgen]
    pub fn get_session_info(&self, token: String) -> Result<JsValue, JsValue> {
        if let Some(session_info) = self.active_mobile_sessions.get(&token) {
            serde_wasm_bindgen::to_value(session_info)
                .map_err(|e| JsValue::from_str(&format!("Failed to serialize session info: {}", e)))
        } else {
            Err(JsValue::from_str("Session not found"))
        }
    }
    
    /// iOS-specific session validation with additional security checks
    #[wasm_bindgen]
    pub fn ios_secure_validate(&mut self, address: String, token: String, biometric_data: JsValue, security_context: JsValue) -> Result<bool, JsValue> {
        // First, perform standard validation
        let is_valid = self.validate_mobile_session(address.clone(), token.clone(), security_context)
            .map_err(|_| JsValue::from_str("Failed to validate session"))?;
        
        if !is_valid {
            return Ok(false);
        }
        
        // In a real implementation, we would verify biometric data here
        // For now, we'll just simulate the process
        let biometric_verified: bool = serde_wasm_bindgen::from_value(biometric_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse biometric data: {}", e)))?;
            
        if !biometric_verified {
            return Err(JsValue::from_str("Biometric verification failed"));
        }
        
        // Additional iOS-specific security checks could be implemented here
        Ok(true)
    }
    
    /// Refresh a mobile session with updated TTL
    #[wasm_bindgen]
    pub fn refresh_session(&mut self, address: String, token: String, new_ttl_seconds: u64) -> Result<bool, JsValue> {
        // First validate the existing session
        let is_valid = match self.inner.validate_session(&address, &token) {
            Ok(_) => true,
            Err(WalletError::SessionExpired) => false,
            Err(WalletError::SessionNotFound) => false,
            Err(e) => return Err(wallet_error_to_js(e)),
        };
        
        if !is_valid {
            return Ok(false);
        }
        
        // Update the session timeout
        let new_timeout = js_sys::Date::now() + (new_ttl_seconds as f64) * 1000.0;
        self.session_timeouts.insert(token.clone(), new_timeout);
        
        // Update last activity
        if let Some(session_info) = self.active_mobile_sessions.get_mut(&token) {
            session_info.last_activity = js_sys::Date::now();
        }
        
        Ok(true)
    }
    
    /// Get all active sessions for a device
    #[wasm_bindgen]
    pub fn get_device_sessions(&self, device_id: String) -> Result<JsValue, JsValue> {
        let mut device_sessions = Vec::new();
        
        for (token, session_info) in &self.active_mobile_sessions {
            if let Some(ref serialized_device_info) = session_info.device_info {
                // Deserialize the device info to check the device_id
                if let Ok(device_info) = serde_json::from_str::<MobileDeviceInfo>(serialized_device_info) {
                    if device_info.device_id == device_id {
                        device_sessions.push(token.clone());
                    }
                }
            }
        }
        
        serde_wasm_bindgen::to_value(&device_sessions)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize device sessions: {}", e)))
    }
    
    /// Revoke all sessions for a specific device (e.g., when device is lost)
    #[wasm_bindgen]
    pub fn revoke_device_sessions(&mut self, device_id: String) -> Result<JsValue, JsValue> {
        let mut revoked_tokens = Vec::new();
        
        // Collect tokens to revoke
        for (token, session_info) in &self.active_mobile_sessions {
            if let Some(ref serialized_device_info) = session_info.device_info {
                // Deserialize the device info to check the device_id
                if let Ok(device_info) = serde_json::from_str::<MobileDeviceInfo>(serialized_device_info) {
                    if device_info.device_id == device_id {
                        revoked_tokens.push(token.clone());
                    }
                }
            }
        }
        
        // Remove the sessions
        for token in &revoked_tokens {
            self.active_mobile_sessions.remove(token);
            self.session_timeouts.remove(token);
        }
        
        serde_wasm_bindgen::to_value(&revoked_tokens)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize revoked tokens: {}", e)))
    }
    
    /// Get session statistics
    #[wasm_bindgen]
    pub fn get_session_statistics(&self) -> Result<JsValue, JsValue> {
        let stats = serde_json::json!({
            "total_active_sessions": self.active_mobile_sessions.len(),
            "total_tracked_timeouts": self.session_timeouts.len(),
            "security_policies": self.security_policies,
        });
        
        serde_wasm_bindgen::to_value(&stats)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize statistics: {}", e)))
    }
    
    /// Update security policies
    #[wasm_bindgen]
    pub fn update_security_policies(&mut self, policies: JsValue) -> Result<(), JsValue> {
        let new_policies: SecurityPolicies = serde_wasm_bindgen::from_value(policies)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse security policies: {}", e)))?;
            
        self.security_policies = new_policies;
        Ok(())
    }
    
    /// Grant additional permissions to a session
    #[wasm_bindgen]
    pub fn grant_session_permissions(&mut self, token: String, permissions: JsValue) -> Result<(), JsValue> {
        let new_permissions: Vec<String> = serde_wasm_bindgen::from_value(permissions)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse permissions: {}", e)))?;
            
        if let Some(session_info) = self.active_mobile_sessions.get_mut(&token) {
            // Add new permissions, avoiding duplicates
            for permission in new_permissions {
                if !session_info.permissions.contains(&permission) {
                    session_info.permissions.push(permission);
                }
            }
            Ok(())
        } else {
            Err(JsValue::from_str("Session not found"))
        }
    }
    
    /// Check if a session has specific permissions
    #[wasm_bindgen]
    pub fn check_session_permissions(&self, token: String, permissions: JsValue) -> Result<bool, JsValue> {
        let required_permissions: Vec<String> = serde_wasm_bindgen::from_value(permissions)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse permissions: {}", e)))?;
            
        if let Some(session_info) = self.active_mobile_sessions.get(&token) {
            // Check if all required permissions are granted
            for permission in required_permissions {
                if !session_info.permissions.contains(&permission) {
                    return Ok(false);
                }
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// Security context for session validation
#[derive(Serialize, Deserialize)]
pub struct SessionSecurityContext {
    pub country_code: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

// The default allocator is used for WASM builds to avoid unmaintained dependencies.

fn wallet_error_to_js(err: WalletError) -> JsValue {
    JsValue::from_str(&err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use serde_wasm_bindgen;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn wasm_sign_and_verify_roundtrip() {
        let signer = WasmWalletSigner::new(
            "0x4c0883a69102937d6231471b5dbb6204fe5129617082796f9b8f0b62f5d7c6c0".into(),
        )
        .expect("signer created");
        let address = signer.address();
        assert_eq!(address, "0x90f8bf6a479f320ead074411a4b0e7944ea8c9c1");
        let signature = signer
            .sign_message("wasm-verify".into())
            .expect("signature");
        verify_wallet_signature(address.clone(), "wasm-verify".into(), signature)
            .expect("verification succeeds");
    }

    #[wasm_bindgen_test]
    fn wasm_session_issue_and_validate() {
        let mut manager = WasmSessionManager::new();
        let address = "0x90f8bf6a479f320ead074411a4b0e7944ea8c9c1".to_string();
        let session = manager
            .issue_session(address.clone(), 180)
            .expect("session issued");
        let parsed: Value = serde_wasm_bindgen::from_value(session).expect("deserialized");
        let token = parsed
            .get("token")
            .and_then(|v| v.as_str())
            .expect("token present");
        manager
            .validate_session(address, token.to_string())
            .expect("session valid");
    }

    // Mobile integration tests
    #[wasm_bindgen_test]
    fn mobile_wallet_creation_test() {
        use crate::mobile_wallet::WasmMobileWallet;
        
        let private_key = "0x4c0883a69102937d6231471b5dbb6204fe5129617082796f9b8f0b62f5d7c6c0";
        let wallet = WasmMobileWallet::new(private_key.to_string());
        assert!(wallet.is_ok());
        
        let wallet = wallet.unwrap();
        let address = wallet.address();
        println!("Generated address: {}", address);
        assert_eq!(address, "0x90f8bf6a479f320ead074411a4b0e7944ea8c9c1");
    }

    #[wasm_bindgen_test]
    fn test_android_google_pay_integration() {
        use crate::mobile_wallet::WasmMobileWallet;
        use crate::{MobileDeviceInfo, SecurityOptions};
        
        let private_key = "0x4c0883a69102937d6231471b5dbb6204fe5129617082796f9b8f0b62f5d7c6c0";
        let mut wallet = WasmMobileWallet::new(private_key.to_string()).unwrap();
        
        // Set device info
        let device_info = MobileDeviceInfo {
            device_id: "test-android-device-123".to_string(),
            platform: "Android".to_string(),
            os_version: "12".to_string(),
            app_version: "1.0.0".to_string(),
            model: "Pixel 6".to_string(),
            security_features: vec!["biometric".to_string(), "secure_element".to_string()],
        };
        
        let device_info_js = serde_wasm_bindgen::to_value(&device_info).unwrap();
        assert!(wallet.set_device_info(device_info_js).is_ok());
        
        // Prepare payment data
        let payment_data = serde_json::json!({
            "amount": "100.00",
            "currency": "USD",
            "merchant": "Test Merchant",
            "timestamp": js_sys::Date::now()
        });
        
        let payment_data_js = serde_wasm_bindgen::to_value(&payment_data).unwrap();
        
        // Prepare security options without biometric verification (should fail)
        let security_options_fail = SecurityOptions {
            biometric_verified: false,
            pin: Some("1234".to_string()),
            location_verified: true,
        };
        
        let security_options_fail_js = serde_wasm_bindgen::to_value(&security_options_fail).unwrap();
        
        let result = wallet.android_google_pay_integration(payment_data_js.clone(), security_options_fail_js);
        assert!(result.is_err());
        assert!(result.unwrap_err().as_string().unwrap().contains("Biometric verification required"));
        
        // Prepare security options with biometric verification (should succeed)
        let security_options_success = SecurityOptions {
            biometric_verified: true,
            pin: Some("1234".to_string()),
            location_verified: true,
        };
        
        let security_options_success_js = serde_wasm_bindgen::to_value(&security_options_success).unwrap();
        
        let result = wallet.android_google_pay_integration(payment_data_js, security_options_success_js);
        assert!(result.is_ok());
        
        let response: serde_json::Value = serde_wasm_bindgen::from_value(result.unwrap()).unwrap();
        assert_eq!(response["platform"], "Android");
        assert_eq!(response["wallet_type"], "Google Pay");
        assert_eq!(response["device_info"]["device_id"], "test-android-device-123");
    }

    #[wasm_bindgen_test]
    fn test_android_security_features() {
        use crate::mobile_wallet::WasmMobileWallet;
        use crate::MobileSecurityContext;
        
        let private_key = "0x4c0883a69102937d6231471b5dbb6204fe5129617082796f9b8f0b62f5d7c6c0";
        let mut wallet = WasmMobileWallet::new(private_key.to_string()).unwrap();
        
        // Test security status
        let security_status = wallet.get_security_status().unwrap();
        let security_context: MobileSecurityContext = serde_wasm_bindgen::from_value(security_status).unwrap();
        assert!(security_context.biometric_required);
        assert!(security_context.pin_required);
        
        // Test updating security settings
        let new_security_context = MobileSecurityContext {
            encryption_key: "new_key".to_string(),
            biometric_required: false,
            pin_required: false,
            failed_attempts: 0,
            last_failed_attempt: None,
            lockout_until: None,
        };
        
        let new_security_context_js = serde_wasm_bindgen::to_value(&new_security_context).unwrap();
        assert!(wallet.update_security_settings(new_security_context_js).is_ok());
        
        // Verify settings were updated
        let security_status = wallet.get_security_status().unwrap();
        let updated_context: MobileSecurityContext = serde_wasm_bindgen::from_value(security_status).unwrap();
        assert!(!updated_context.biometric_required);
        assert!(!updated_context.pin_required);
        assert_eq!(updated_context.encryption_key, "new_key");
    }
}
