//! CORS Policy for Security Layer 4 - API & Gateway Security
//!
//! Implements Cross-Origin Resource Sharing (CORS) policy enforcement.
//! From DEX-OS-V2.csv line 238:
//! - Security,Security Layer,Security Layer 4,API & Gateway Security,API Protection,High

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// HTTP methods
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    OPTIONS,
    HEAD,
}

impl HttpMethod {
    pub fn as_str(&self) -> &str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::OPTIONS => "OPTIONS",
            HttpMethod::HEAD => "HEAD",
        }
    }
}

/// CORS Policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CORSPolicy {
    /// Allowed origins (use "*" for all origins)
    pub allowed_origins: HashSet<String>,
    /// Allowed HTTP methods
    pub allowed_methods: HashSet<HttpMethod>,
    /// Allowed request headers
    pub allowed_headers: HashSet<String>,
    /// Headers exposed to the client
    pub exposed_headers: HashSet<String>,
    /// Whether to allow credentials (cookies, authorization headers)
    pub allow_credentials: bool,
    /// Max age for preflight cache (in seconds)
    pub max_age: u64,
}

impl CORSPolicy {
    /// Create a new CORS policy
    pub fn new() -> Self {
        Self {
            allowed_origins: HashSet::new(),
            allowed_methods: HashSet::new(),
            allowed_headers: HashSet::new(),
            exposed_headers: HashSet::new(),
            allow_credentials: false,
            max_age: 3600, // 1 hour
        }
    }

    /// Create a permissive CORS policy (allows all origins)
    pub fn permissive() -> Self {
        let mut policy = Self::new();
        policy.allowed_origins.insert("*".to_string());
        policy.allowed_methods.insert(HttpMethod::GET);
        policy.allowed_methods.insert(HttpMethod::POST);
        policy.allowed_methods.insert(HttpMethod::PUT);
        policy.allowed_methods.insert(HttpMethod::DELETE);
        policy.allowed_methods.insert(HttpMethod::PATCH);
        policy.allowed_methods.insert(HttpMethod::OPTIONS);
        policy.allowed_headers.insert("*".to_string());
        policy.exposed_headers.insert("*".to_string());
        policy.allow_credentials = true;
        policy
    }

    /// Create a strict CORS policy (specific origins only)
    pub fn strict(allowed_origins: Vec<String>) -> Self {
        let mut policy = Self::new();
        policy.allowed_origins = allowed_origins.into_iter().collect();
        policy.allowed_methods.insert(HttpMethod::GET);
        policy.allowed_methods.insert(HttpMethod::POST);
        policy.allowed_headers.insert("Content-Type".to_string());
        policy.allowed_headers.insert("Authorization".to_string());
        policy.exposed_headers.insert("Content-Type".to_string());
        policy.allow_credentials = false;
        policy
    }

    /// Add an allowed origin
    pub fn add_origin(&mut self, origin: String) {
        self.allowed_origins.insert(origin);
    }

    /// Add an allowed method
    pub fn add_method(&mut self, method: HttpMethod) {
        self.allowed_methods.insert(method);
    }

    /// Add an allowed header
    pub fn add_header(&mut self, header: String) {
        self.allowed_headers.insert(header);
    }

    /// Add an exposed header
    pub fn add_exposed_header(&mut self, header: String) {
        self.exposed_headers.insert(header);
    }

    /// Check if an origin is allowed
    pub fn is_origin_allowed(&self, origin: &str) -> bool {
        self.allowed_origins.contains("*") || self.allowed_origins.contains(origin)
    }

    /// Check if a method is allowed
    pub fn is_method_allowed(&self, method: &HttpMethod) -> bool {
        self.allowed_methods.contains(method)
    }

    /// Check if a header is allowed
    pub fn is_header_allowed(&self, header: &str) -> bool {
        self.allowed_headers.contains("*") || self.allowed_headers.contains(header)
    }

    /// Get CORS headers for a response
    pub fn get_cors_headers(&self, origin: Option<&str>) -> Vec<(String, String)> {
        let mut headers = Vec::new();

        // Access-Control-Allow-Origin
        if let Some(origin) = origin {
            if self.is_origin_allowed(origin) {
                if self.allowed_origins.contains("*") {
                    headers.push(("Access-Control-Allow-Origin".to_string(), "*".to_string()));
                } else {
                    headers.push(("Access-Control-Allow-Origin".to_string(), origin.to_string()));
                }
            }
        }

        // Access-Control-Allow-Methods
        let methods: Vec<String> = self
            .allowed_methods
            .iter()
            .map(|m| m.as_str().to_string())
            .collect();
        if !methods.is_empty() {
            headers.push(("Access-Control-Allow-Methods".to_string(), methods.join(", ")));
        }

        // Access-Control-Allow-Headers
        if self.allowed_headers.contains("*") {
            headers.push(("Access-Control-Allow-Headers".to_string(), "*".to_string()));
        } else {
            let allowed_headers: Vec<String> = self.allowed_headers.iter().cloned().collect();
            if !allowed_headers.is_empty() {
                headers.push((
                    "Access-Control-Allow-Headers".to_string(),
                    allowed_headers.join(", "),
                ));
            }
        }

        // Access-Control-Expose-Headers
        if self.exposed_headers.contains("*") {
            headers.push(("Access-Control-Expose-Headers".to_string(), "*".to_string()));
        } else {
            let exposed: Vec<String> = self.exposed_headers.iter().cloned().collect();
            if !exposed.is_empty() {
                headers.push(("Access-Control-Expose-Headers".to_string(), exposed.join(", ")));
            }
        }

        // Access-Control-Allow-Credentials
        if self.allow_credentials {
            headers.push(("Access-Control-Allow-Credentials".to_string(), "true".to_string()));
        }

        // Access-Control-Max-Age
        headers.push(("Access-Control-Max-Age".to_string(), self.max_age.to_string()));

        headers
    }

    /// Handle preflight request
    pub fn handle_preflight(
        &self,
        origin: &str,
        method: &HttpMethod,
        headers: &[String],
    ) -> Result<Vec<(String, String)>, String> {
        // Check origin
        if !self.is_origin_allowed(origin) {
            return Err(format!("Origin not allowed: {}", origin));
        }

        // Check method
        if !self.is_method_allowed(method) {
            return Err(format!("Method not allowed: {:?}", method));
        }

        // Check headers
        for header in headers {
            if !self.is_header_allowed(header) {
                return Err(format!("Header not allowed: {}", header));
            }
        }

        Ok(self.get_cors_headers(Some(origin)))
    }
}

impl Default for CORSPolicy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cors_policy_creation() {
        let policy = CORSPolicy::new();
        assert_eq!(policy.max_age, 3600);
        assert!(!policy.allow_credentials);
    }

    #[test]
    fn test_permissive_cors_policy() {
        let policy = CORSPolicy::permissive();
        assert!(policy.is_origin_allowed("https://example.com"));
        assert!(policy.is_origin_allowed("https://any-origin.com"));
        assert!(policy.is_method_allowed(&HttpMethod::GET));
        assert!(policy.is_method_allowed(&HttpMethod::POST));
    }

    #[test]
    fn test_strict_cors_policy() {
        let policy = CORSPolicy::strict(vec!["https://example.com".to_string()]);
        assert!(policy.is_origin_allowed("https://example.com"));
        assert!(!policy.is_origin_allowed("https://other.com"));
    }

    #[test]
    fn test_cors_headers() {
        let mut policy = CORSPolicy::new();
        policy.add_origin("https://example.com".to_string());
        policy.add_method(HttpMethod::GET);
        policy.add_header("Content-Type".to_string());

        let headers = policy.get_cors_headers(Some("https://example.com"));
        
        // Should have Access-Control-Allow-Origin
        assert!(headers.iter().any(|(k, v)| k == "Access-Control-Allow-Origin" && v == "https://example.com"));
        
        // Should have Access-Control-Allow-Methods
        assert!(headers.iter().any(|(k, _)| k == "Access-Control-Allow-Methods"));
    }

    #[test]
    fn test_preflight_request() {
        let mut policy = CORSPolicy::new();
        policy.add_origin("https://example.com".to_string());
        policy.add_method(HttpMethod::POST);
        policy.add_header("Content-Type".to_string());

        let result = policy.handle_preflight(
            "https://example.com",
            &HttpMethod::POST,
            &vec!["Content-Type".to_string()],
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_preflight_request_blocked() {
        let policy = CORSPolicy::strict(vec!["https://example.com".to_string()]);

        // Wrong origin
        let result = policy.handle_preflight(
            "https://malicious.com",
            &HttpMethod::GET,
            &vec![],
        );
        assert!(result.is_err());
    }
}
