//! Mobile session manager for the WASM module
//!
//! This module provides enhanced session management for mobile devices
//! including security policies, geofencing, and permission management.

use dex_core::wallet::{SessionManager, WalletError};
use crate::{MobileDeviceInfo, SecurityPolicies, MobileSessionInfo, SessionSecurityContext};
use wasm_bindgen::prelude::*;

/// Mobile-specific session manager with enhanced security features
#[wasm_bindgen]
pub struct WasmMobileSessionManager {
    inner: SessionManager,
    active_mobile_sessions: std::collections::HashMap<String, MobileSessionInfo>,
    session_timeouts: std::collections::HashMap<String, f64>, // token -> timeout timestamp
    security_policies: SecurityPolicies,
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

        let session = self
            .inner
            .issue_session(&address, ttl_seconds)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

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
            Err(e) => Err(JsValue::from_str(&e.to_string())),
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
        let is_valid = match self.validate_mobile_session(address.clone(), token.clone(), security_context) {
            Ok(valid) => valid,
            Err(e) => return Err(e),
        };
        
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
            Err(e) => return Err(JsValue::from_str(&e.to_string())),
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