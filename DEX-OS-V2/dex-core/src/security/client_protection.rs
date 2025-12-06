//! Client Protection Module for Security Layer 10 - Front-End & User Safety
//!
//! Implements client-side security protections from DEX-OS-V2.csv line 244:
//! - Security,Security Layer,Security Layer 10,Front-End & User Safety,Client Protection,High
//!
//! Features:
//! - CSRF token generation and validation
//! - XSS prevention utilities
//! - Content Security Policy (CSP) configuration
//! - Secure cookie management
//! - Session hijacking protection
//! - Clickjacking protection
//! - Browser fingerprint detection

use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use rand::RngCore;

/// Client protection errors
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ClientProtectionError {
    #[error("Invalid CSRF token")]
    InvalidCsrfToken,
    #[error("CSRF token expired")]
    CsrfTokenExpired,
    #[error("CSRF token not found for session {0}")]
    CsrfTokenNotFound(String),
    #[error("XSS attack detected in input")]
    XssDetected,
    #[error("Session not found: {0}")]
    SessionNotFound(String),
    #[error("Session expired: {0}")]
    SessionExpired(String),
    #[error("Invalid session token")]
    InvalidSession,
    #[error("Clickjacking attempt detected")]
    ClickjackingDetected,
}

/// CSRF Token with expiration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CsrfToken {
    /// Token value
    pub token: String,
    /// Session ID this token belongs to
    pub session_id: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Expiration timestamp
    pub expires_at: u64,
}

impl CsrfToken {
    /// Create a new CSRF token
    pub fn new(session_id: String, ttl_seconds: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Generate random token
        let mut rng = rand::thread_rng();
        let mut token_bytes = [0u8; 32];
        rng.fill_bytes(&mut token_bytes);
        
        let token = base64::encode(&token_bytes);

        Self {
            token,
            session_id,
            created_at: now,
            expires_at: now + ttl_seconds,
        }
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now > self.expires_at
    }

    /// Validate token value
    pub fn validate(&self, token: &str) -> bool {
        !self.is_expired() && self.token == token
    }
}

/// Session information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    /// Session ID
    pub id: String,
    /// User ID
    pub user_id: Option<String>,
    /// Creation timestamp
    pub created_at: u64,
    /// Last activity timestamp
    pub last_activity: u64,
    /// Expiration timestamp
    pub expires_at: u64,
    /// Browser fingerprint
    pub fingerprint: Option<String>,
    /// IP address
    pub ip_address: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
}

impl Session {
    /// Create a new session
    pub fn new(ttl_seconds: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut rng = rand::thread_rng();
        let mut id_bytes = [0u8; 32];
        rng.fill_bytes(&mut id_bytes);
        
        let id = base64::encode(&id_bytes);

        Self {
            id,
            user_id: None,
            created_at: now,
            last_activity: now,
            expires_at: now + ttl_seconds,
            fingerprint: None,
            ip_address: None,
            user_agent: None,
        }
    }

    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now > self.expires_at
    }

    /// Update last activity timestamp
    pub fn update_activity(&mut self) {
        self.last_activity = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Check if session fingerprint matches
    pub fn validate_fingerprint(&self, fingerprint: &str) -> bool {
        if let Some(ref fp) = self.fingerprint {
            fp == fingerprint
        } else {
            true // No fingerprint set, allow
        }
    }
}

/// Content Security Policy configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentSecurityPolicy {
    /// default-src directive
    pub default_src: Vec<String>,
    /// script-src directive
    pub script_src: Vec<String>,
    /// style-src directive
    pub style_src: Vec<String>,
    /// img-src directive
    pub img_src: Vec<String>,
    /// connect-src directive
    pub connect_src: Vec<String>,
    /// font-src directive
    pub font_src: Vec<String>,
    /// frame-ancestors directive (clickjacking protection)
    pub frame_ancestors: Vec<String>,
}

impl ContentSecurityPolicy {
    /// Create a strict CSP
    pub fn strict() -> Self {
        Self {
            default_src: vec!["'self'".to_string()],
            script_src: vec!["'self'".to_string()],
            style_src: vec!["'self'".to_string()],
            img_src: vec!["'self'".to_string(), "data:".to_string()],
            connect_src: vec!["'self'".to_string()],
            font_src: vec!["'self'".to_string()],
            frame_ancestors: vec!["'none'".to_string()],
        }
    }

    /// Create a permissive CSP
    pub fn permissive() -> Self {
        Self {
            default_src: vec!["'self'".to_string(), "*".to_string()],
            script_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
            style_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
            img_src: vec!["*".to_string()],
            connect_src: vec!["*".to_string()],
            font_src: vec!["*".to_string()],
            frame_ancestors: vec!["'self'".to_string()],
        }
    }

    /// Convert to CSP header string
    pub fn to_header(&self) -> String {
        let mut parts = Vec::new();

        if !self.default_src.is_empty() {
            parts.push(format!("default-src {}", self.default_src.join(" ")));
        }
        if !self.script_src.is_empty() {
            parts.push(format!("script-src {}", self.script_src.join(" ")));
        }
        if !self.style_src.is_empty() {
            parts.push(format!("style-src {}", self.style_src.join(" ")));
        }
        if !self.img_src.is_empty() {
            parts.push(format!("img-src {}", self.img_src.join(" ")));
        }
        if !self.connect_src.is_empty() {
            parts.push(format!("connect-src {}", self.connect_src.join(" ")));
        }
        if !self.font_src.is_empty() {
            parts.push(format!("font-src {}", self.font_src.join(" ")));
        }
        if !self.frame_ancestors.is_empty() {
            parts.push(format!("frame-ancestors {}", self.frame_ancestors.join(" ")));
        }

        parts.join("; ")
    }
}

/// Secure cookie attributes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecureCookie {
    /// Cookie name
    pub name: String,
    /// Cookie value
    pub value: String,
    /// HttpOnly flag
    pub http_only: bool,
    /// Secure flag (HTTPS only)
    pub secure: bool,
    /// SameSite attribute
    pub same_site: SameSitePolicy,
    /// Max age in seconds
    pub max_age: Option<u64>,
    /// Domain
    pub domain: Option<String>,
    /// Path
    pub path: String,
}

/// SameSite cookie policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SameSitePolicy {
    Strict,
    Lax,
    None,
}

impl SecureCookie {
    /// Create a new secure cookie with safe defaults
    pub fn new(name: String, value: String) -> Self {
        Self {
            name,
            value,
            http_only: true,
            secure: true,
            same_site: SameSitePolicy::Strict,
            max_age: Some(3600), // 1 hour
            domain: None,
            path: "/".to_string(),
        }
    }

    /// Convert to Set-Cookie header string
    pub fn to_header(&self) -> String {
        let mut parts = vec![format!("{}={}", self.name, self.value)];

        if self.http_only {
            parts.push("HttpOnly".to_string());
        }
        if self.secure {
            parts.push("Secure".to_string());
        }

        parts.push(match self.same_site {
            SameSitePolicy::Strict => "SameSite=Strict".to_string(),
            SameSitePolicy::Lax => "SameSite=Lax".to_string(),
            SameSitePolicy::None => "SameSite=None".to_string(),
        });

        if let Some(max_age) = self.max_age {
            parts.push(format!("Max-Age={}", max_age));
        }

        if let Some(ref domain) = self.domain {
            parts.push(format!("Domain={}", domain));
        }

        parts.push(format!("Path={}", self.path));

        parts.join("; ")
    }
}

/// Client Protection Manager
#[derive(Debug, Clone)]
pub struct ClientProtectionManager {
    /// CSRF tokens by session ID
    csrf_tokens: Arc<RwLock<HashMap<String, CsrfToken>>>,
    /// Active sessions
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    /// Content Security Policy
    csp: ContentSecurityPolicy,
    /// XSS patterns to detect
    xss_patterns: Vec<regex::Regex>,
}

impl ClientProtectionManager {
    /// Create a new client protection manager
    pub fn new() -> Self {
        Self {
            csrf_tokens: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            csp: ContentSecurityPolicy::strict(),
            xss_patterns: Self::build_xss_patterns(),
        }
    }

    /// Build XSS detection patterns
    fn build_xss_patterns() -> Vec<regex::Regex> {
        vec![
            regex::Regex::new(r"(?i)<script[^>]*>").unwrap(),
            regex::Regex::new(r"(?i)</script>").unwrap(),
            regex::Regex::new(r"(?i)javascript:").unwrap(),
            regex::Regex::new(r"(?i)onerror\s*=").unwrap(),
            regex::Regex::new(r"(?i)onload\s*=").unwrap(),
            regex::Regex::new(r"(?i)onclick\s*=").unwrap(),
            regex::Regex::new(r"(?i)<iframe[^>]*>").unwrap(),
            regex::Regex::new(r"(?i)<embed[^>]*>").unwrap(),
            regex::Regex::new(r"(?i)<object[^>]*>").unwrap(),
            regex::Regex::new(r"(?i)eval\s*\(").unwrap(),
        ]
    }

    /// Set Content Security Policy
    pub fn set_csp(&mut self, csp: ContentSecurityPolicy) {
        self.csp = csp;
    }

    /// Get CSP header
    pub fn get_csp_header(&self) -> String {
        self.csp.to_header()
    }

    /// Create a new session
    pub fn create_session(&self, ttl_seconds: u64) -> Session {
        let mut session = Session::new(ttl_seconds);
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(session.id.clone(), session.clone());
        session
    }

    /// Get session by ID
    pub fn get_session(&self, session_id: &str) -> Result<Session, ClientProtectionError> {
        let sessions = self.sessions.read().unwrap();
        let session = sessions
            .get(session_id)
            .ok_or_else(|| ClientProtectionError::SessionNotFound(session_id.to_string()))?;

        if session.is_expired() {
            return Err(ClientProtectionError::SessionExpired(session_id.to_string()));
        }

        Ok(session.clone())
    }

    /// Update session activity
    pub fn update_session_activity(&self, session_id: &str) -> Result<(), ClientProtectionError> {
        let mut sessions = self.sessions.write().unwrap();
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| ClientProtectionError::SessionNotFound(session_id.to_string()))?;

        session.update_activity();
        Ok(())
    }

    /// Validate session with fingerprint
    pub fn validate_session(
        &self,
        session_id: &str,
        fingerprint: Option<&str>,
    ) -> Result<(), ClientProtectionError> {
        let session = self.get_session(session_id)?;

        if let Some(fp) = fingerprint {
            if !session.validate_fingerprint(fp) {
                return Err(ClientProtectionError::InvalidSession);
            }
        }

        Ok(())
    }

    /// Generate CSRF token for a session
    pub fn generate_csrf_token(&self, session_id: String, ttl_seconds: u64) -> CsrfToken {
        let token = CsrfToken::new(session_id.clone(), ttl_seconds);
        let mut tokens = self.csrf_tokens.write().unwrap();
        tokens.insert(session_id, token.clone());
        token
    }

    /// Validate CSRF token
    pub fn validate_csrf_token(
        &self,
        session_id: &str,
        token: &str,
    ) -> Result<(), ClientProtectionError> {
        let tokens = self.csrf_tokens.read().unwrap();
        let csrf_token = tokens
            .get(session_id)
            .ok_or_else(|| ClientProtectionError::CsrfTokenNotFound(session_id.to_string()))?;

        if csrf_token.is_expired() {
            return Err(ClientProtectionError::CsrfTokenExpired);
        }

        if !csrf_token.validate(token) {
            return Err(ClientProtectionError::InvalidCsrfToken);
        }

        Ok(())
    }

    /// Check for XSS patterns in input
    pub fn check_xss(&self, input: &str) -> Result<(), ClientProtectionError> {
        for pattern in &self.xss_patterns {
            if pattern.is_match(input) {
                return Err(ClientProtectionError::XssDetected);
            }
        }
        Ok(())
    }

    /// Sanitize HTML input (basic removal of dangerous tags)
    pub fn sanitize_html(&self, input: &str) -> String {
        let mut sanitized = input.to_string();

        // Remove script tags
        sanitized = regex::Regex::new(r"<script[^>]*>.*?</script>")
            .unwrap()
            .replace_all(&sanitized, "")
            .to_string();

        // Remove event handlers
        sanitized = regex::Regex::new(r#"on\w+\s*=\s*["'][^"']*["']"#)
            .unwrap()
            .replace_all(&sanitized, "")
            .to_string();

        // Remove javascript: protocol
        sanitized = regex::Regex::new(r"(?i)javascript:")
            .unwrap()
            .replace_all(&sanitized, "")
            .to_string();

        sanitized
    }

    /// Generate browser fingerprint from request headers
    pub fn generate_fingerprint(
        user_agent: &str,
        accept_language: &str,
        accept_encoding: &str,
    ) -> String {
        let mut hasher = Sha3_256::new();
        hasher.update(user_agent.as_bytes());
        hasher.update(accept_language.as_bytes());
        hasher.update(accept_encoding.as_bytes());
        let result = hasher.finalize();
        base64::encode(&result)
    }

    /// Clean up expired sessions and tokens
    pub fn cleanup_expired(&self) {
        let mut sessions = self.sessions.write().unwrap();
        sessions.retain(|_, session| !session.is_expired());

        let mut tokens = self.csrf_tokens.write().unwrap();
        tokens.retain(|_, token| !token.is_expired());
    }

    /// Get security headers for response
    pub fn get_security_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        headers.insert("Content-Security-Policy".to_string(), self.get_csp_header());
        headers.insert("X-Frame-Options".to_string(), "DENY".to_string());
        headers.insert("X-Content-Type-Options".to_string(), "nosniff".to_string());
        headers.insert("X-XSS-Protection".to_string(), "1; mode=block".to_string());
        headers.insert("Referrer-Policy".to_string(), "strict-origin-when-cross-origin".to_string());
        headers.insert(
            "Permissions-Policy".to_string(),
            "geolocation=(), microphone=(), camera=()".to_string(),
        );

        headers
    }
}

impl Default for ClientProtectionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csrf_token_generation() {
        let token = CsrfToken::new("session123".to_string(), 3600);
        assert_eq!(token.session_id, "session123");
        assert!(!token.is_expired());
    }

    #[test]
    fn test_csrf_token_validation() {
        let token = CsrfToken::new("session123".to_string(), 3600);
        assert!(token.validate(&token.token));
        assert!(!token.validate("invalid_token"));
    }

    #[test]
    fn test_session_creation() {
        let session = Session::new(3600);
        assert!(!session.is_expired());
    }

    #[test]
    fn test_csp_header_generation() {
        let csp = ContentSecurityPolicy::strict();
        let header = csp.to_header();
        assert!(header.contains("default-src 'self'"));
        assert!(header.contains("frame-ancestors 'none'"));
    }

    #[test]
    fn test_secure_cookie_header() {
        let cookie = SecureCookie::new("session".to_string(), "abc123".to_string());
        let header = cookie.to_header();
        assert!(header.contains("HttpOnly"));
        assert!(header.contains("Secure"));
        assert!(header.contains("SameSite=Strict"));
    }

    #[test]
    fn test_xss_detection() {
        let manager = ClientProtectionManager::new();
        
        assert!(manager.check_xss("<script>alert('xss')</script>").is_err());
        assert!(manager.check_xss("javascript:alert(1)").is_err());
        assert!(manager.check_xss("Safe text").is_ok());
    }

    #[test]
    fn test_html_sanitization() {
        let manager = ClientProtectionManager::new();
        
        let input = "Hello <script>alert('xss')</script> World";
        let sanitized = manager.sanitize_html(input);
        assert!(!sanitized.contains("<script>"));
        assert!(sanitized.contains("Hello"));
        assert!(sanitized.contains("World"));
    }

    #[test]
    fn test_fingerprint_generation() {
        let fp1 = ClientProtectionManager::generate_fingerprint(
            "Mozilla/5.0", "en-US", "gzip"
        );
        let fp2 = ClientProtectionManager::generate_fingerprint(
            "Mozilla/5.0", "en-US", "gzip"
        );
        let fp3 = ClientProtectionManager::generate_fingerprint(
            "Chrome/90.0", "en-US", "gzip"
        );
        
        assert_eq!(fp1, fp2);
        assert_ne!(fp1, fp3);
    }

    #[test]
    fn test_client_protection_manager() {
        let manager = ClientProtectionManager::new();
        
        // Create session
        let session = manager.create_session(3600);
        assert!(manager.get_session(&session.id).is_ok());
        
        // Generate CSRF token
        let csrf = manager.generate_csrf_token(session.id.clone(), 3600);
        assert!(manager.validate_csrf_token(&session.id, &csrf.token).is_ok());
        assert!(manager.validate_csrf_token(&session.id, "invalid").is_err());
    }
}
