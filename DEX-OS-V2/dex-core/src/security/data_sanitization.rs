//! Data Sanitization Module for Protection Layer 2 - Input Validation
//!
//! Implements data sanitization from DEX-OS-V2.csv line 246:
//! - Security,Protection Layer,Protection Layer 2,Input Validation,Data Sanitization,High

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;
use regex::Regex;

/// Data sanitization errors
#[derive(Debug, Error, Clone, PartialEq)]
pub enum SanitizationError {
    #[error("SQL injection detected in input")]
    SqlInjectionDetected,
    #[error("NoSQL injection detected in input")]
    NoSqlInjectionDetected,
    #[error("Path traversal detected in input")]
    PathTraversalDetected,
    #[error("Command injection detected in input")]
    CommandInjectionDetected,
    #[error("LDAP injection detected in input")]
    LdapInjectionDetected,
    #[error("XXE attack detected in input")]
    XxeDetected,
    #[error("Dangerous character detected: {0}")]
    DangerousCharacter(String),
    #[error("Invalid encoding detected")]
    InvalidEncoding,
}

/// Sanitization level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SanitizationLevel {
    Basic,
    Moderate,
    Strict,
}

/// Sanitization result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SanitizationResult {
    pub original: String,
    pub sanitized: String,
    pub was_modified: bool,
    pub threats: Vec<String>,
}

/// Data sanitization manager
#[derive(Debug, Clone)]
pub struct DataSanitizer {
    level: SanitizationLevel,
    sql_patterns: Vec<Regex>,
    nosql_patterns: Vec<Regex>,
    command_patterns: Vec<Regex>,
    ldap_patterns: Vec<Regex>,
    path_patterns: Vec<Regex>,
    xxe_patterns: Vec<Regex>,
    allowed_html_tags: HashSet<String>,
}

impl DataSanitizer {
    pub fn new(level: SanitizationLevel) -> Self {
        Self {
            level,
            sql_patterns: Self::build_sql_patterns(),
            nosql_patterns: Self::build_nosql_patterns(),
            command_patterns: Self::build_command_patterns(),
            ldap_patterns: Self::build_ldap_patterns(),
            path_patterns: Self::build_path_patterns(),
            xxe_patterns: Self::build_xxe_patterns(),
            allowed_html_tags: Self::build_allowed_html_tags(level),
        }
    }

    fn build_sql_patterns() -> Vec<Regex> {
        vec![
            Regex::new(r"(?i)(\bUNION\b.+\bSELECT\b)").unwrap(),
            Regex::new(r"(?i)(\bSELECT\b.+\bFROM\b)").unwrap(),
            Regex::new(r"[';]").unwrap(),
        ]
    }

    fn build_nosql_patterns() -> Vec<Regex> {
        vec![
            Regex::new(r"\$where").unwrap(),
            Regex::new(r"\$ne").unwrap(),
        ]
    }

    fn build_command_patterns() -> Vec<Regex> {
        vec![
            Regex::new(r"[;&|`]").unwrap(),
            Regex::new(r"\$\(").unwrap(),
        ]
    }

    fn build_ldap_patterns() -> Vec<Regex> {
        vec![
            Regex::new(r"[*()\\]").unwrap(),
        ]
    }

    fn build_path_patterns() -> Vec<Regex> {
        vec![
            Regex::new(r"\.\.[/\\]").unwrap(),
        ]
    }

    fn build_xxe_patterns() -> Vec<Regex> {
        vec![
            Regex::new(r"<!ENTITY").unwrap(),
        ]
    }

    fn build_allowed_html_tags(_level: SanitizationLevel) -> HashSet<String> {
        let mut tags = HashSet::new();
        tags.insert("p".to_string());
        tags.insert("br".to_string());
        tags
    }

    pub fn check_sql_injection(&self, input: &str) -> Result<(), SanitizationError> {
        for pattern in &self.sql_patterns {
            if pattern.is_match(input) {
                return Err(SanitizationError::SqlInjectionDetected);
            }
        }
        Ok(())
    }

    pub fn check_nosql_injection(&self, input: &str) -> Result<(), SanitizationError> {
        for pattern in &self.nosql_patterns {
            if pattern.is_match(input) {
                return Err(SanitizationError::NoSqlInjectionDetected);
            }
        }
        Ok(())
    }

    pub fn check_command_injection(&self, input: &str) -> Result<(), SanitizationError> {
        for pattern in &self.command_patterns {
            if pattern.is_match(input) {
                return Err(SanitizationError::CommandInjectionDetected);
            }
        }
        Ok(())
    }

    pub fn check_ldap_injection(&self, input: &str) -> Result<(), SanitizationError> {
        for pattern in &self.ldap_patterns {
            if pattern.is_match(input) {
                return Err(SanitizationError::LdapInjectionDetected);
            }
        }
        Ok(())
    }

    pub fn check_path_traversal(&self, input: &str) -> Result<(), SanitizationError> {
        for pattern in &self.path_patterns {
            if pattern.is_match(input) {
                return Err(SanitizationError::PathTraversalDetected);
            }
        }
        Ok(())
    }

    pub fn check_xxe(&self, input: &str) -> Result<(), SanitizationError> {
        for pattern in &self.xxe_patterns {
            if pattern.is_match(input) {
                return Err(SanitizationError::XxeDetected);
            }
        }
        Ok(())
    }

    pub fn sanitize_sql(&self, input: &str) -> String {
        input.replace("'", "''")
    }

    pub fn sanitize_filename(&self, input: &str) -> String {
        input.replace("..", "").replace("/", "").replace("\\", "")
    }

    pub fn sanitize_html(&self, input: &str) -> String {
        Regex::new(r"(?i)<script[^>]*>.*?</script>")
            .unwrap()
            .replace_all(input, "")
            .to_string()
    }

    pub fn sanitize(&self, input: &str) -> SanitizationResult {
        let original = input.to_string();
        let mut sanitized = input.to_string();
        let mut threats = Vec::new();

        if self.check_sql_injection(&sanitized).is_err() {
            threats.push("SQL Injection".to_string());
            sanitized = self.sanitize_sql(&sanitized);
        }

        if self.check_path_traversal(&sanitized).is_err() {
            threats.push("Path Traversal".to_string());
            sanitized = self.sanitize_filename(&sanitized);
        }

        sanitized = self.sanitize_html(&sanitized);

        SanitizationResult {
            original: original.clone(),
            sanitized: sanitized.clone(),
            was_modified: original != sanitized,
            threats,
        }
    }

    pub fn sanitize_email(&self, email: &str) -> Result<String, SanitizationError> {
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        let trimmed = email.trim();
        
        if !email_regex.is_match(trimmed) {
            return Err(SanitizationError::InvalidEncoding);
        }

        Ok(trimmed.to_lowercase())
    }

    pub fn sanitize_url(&self, url: &str) -> Result<String, SanitizationError> {
        let url = url.trim();
        
        if url.starts_with("javascript:") || url.starts_with("data:") || url.starts_with("file:") {
            return Err(SanitizationError::DangerousCharacter("dangerous protocol".to_string()));
        }

        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(SanitizationError::InvalidEncoding);
        }

        Ok(url.to_string())
    }
}

impl Default for DataSanitizer {
    fn default() -> Self {
        Self::new(SanitizationLevel::Moderate)
    }
}

/// Data Validator for strict validation checks (without modification)
#[derive(Debug, Clone)]
pub struct DataValidator;

impl DataValidator {
    pub fn new() -> Self {
        Self
    }

    /// Validate email format
    pub fn validate_email(&self, email: &str) -> bool {
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        email_regex.is_match(email)
    }

    /// Validate URL format
    pub fn validate_url(&self, url: &str) -> bool {
        let url_regex = Regex::new(r"^(https?://)?([\da-z\.-]+)\.([a-z\.]{2,6})([/\w \.-]*)*/?$").unwrap();
        url_regex.is_match(url)
    }

    /// Validate username (alphanumeric, 3-20 chars)
    pub fn validate_username(&self, username: &str) -> bool {
        let regex = Regex::new(r"^[a-zA-Z0-9_]{3,20}$").unwrap();
        regex.is_match(username)
    }

    /// Validate strong password
    /// - At least 8 chars
    /// - At least one uppercase
    /// - At least one lowercase
    /// - At least one number
    /// - At least one special char
    pub fn validate_password_strength(&self, password: &str) -> bool {
        if password.len() < 8 { return false; }
        
        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_numeric());
        let has_special = password.chars().any(|c| !c.is_alphanumeric());

        has_upper && has_lower && has_digit && has_special
    }

    /// Validate UUID format
    pub fn validate_uuid(&self, uuid: &str) -> bool {
        let regex = Regex::new(r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$").unwrap();
        regex.is_match(uuid)
    }

    /// Validate JSON string
    pub fn validate_json(&self, json: &str) -> bool {
        serde_json::from_str::<serde_json::Value>(json).is_ok()
    }
}

impl Default for DataValidator {
    fn default() -> Self {
        Self::new()
    }
}
