//! Mobile wallet functionality for the WASM module
//!
//! This module provides mobile-specific wallet functionality including
//! device binding, enhanced security, and platform-specific integrations.

use dex_core::wallet::{WalletSigner, SessionManager, WalletError};
use crate::{MobileDeviceInfo, MobileSecurityContext, SecurityOptions, SecurityChallenge};
use wasm_bindgen::prelude::*;
#[cfg(test)]
use wasm_bindgen_test::*;
#[cfg(test)]
use js_sys;

/// WASM wrapper for mobile wallet functionality with enhanced security
#[wasm_bindgen]
pub struct WasmMobileWallet {
    inner: WalletSigner,
    session_manager: SessionManager,
    device_info: Option<String>, // Serialized MobileDeviceInfo
    security_context: MobileSecurityContext,
}

#[wasm_bindgen]
impl WasmMobileWallet {
    /// Create a new mobile wallet signer from a private key
    #[wasm_bindgen(constructor)]
    pub fn new(private_key: String) -> Result<WasmMobileWallet, JsValue> {
        let inner = WalletSigner::from_private_key_hex(&private_key).map_err(|e| JsValue::from_str(&e.to_string()))?;
        
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
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Issue a mobile session with TTL and security context
    #[wasm_bindgen]
    pub fn issue_mobile_session(&mut self, ttl_seconds: u64) -> Result<JsValue, JsValue> {
        let session = self
            .session_manager
            .issue_session(self.inner.address(), ttl_seconds)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

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
            Err(e) => Err(JsValue::from_str(&e.to_string())),
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
            .map_err(|e| JsValue::from_str(&e.to_string()))
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