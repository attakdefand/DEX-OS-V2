//! API Gateway for Security Layer 4 - API & Gateway Security
//!
//! Central entry point for all API requests with routing, validation, and security.
//! From DEX-OS-V2.csv line 238:
//! - Security,Security Layer,Security Layer 4,API & Gateway Security,API Protection,High

use super::{APIKeyManager, APIRateLimiter, CORSPolicy, HttpMethod};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

/// API Gateway error types
#[derive(Debug, Error, Clone)]
pub enum APIGatewayError {
    #[error("Route not found: {0}")]
    RouteNotFound(String),
    #[error("Method not allowed: {0}")]
    MethodNotAllowed(String),
    #[error("Authentication required")]
    AuthenticationRequired,
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    #[error("CORS policy violation: {0}")]
    CORSViolation(String),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

/// Route configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    /// Route path (e.g., "/api/users")
    pub path: String,
    /// Allowed HTTP methods
    pub methods: Vec<HttpMethod>,
    /// Whether authentication is required
    pub auth_required: bool,
    /// Required scopes for this route
    pub required_scopes: Vec<String>,
    /// Custom rate limit for this route
    pub rate_limit: Option<super::RateLimit>,
}

/// API Gateway
#[derive(Debug, Clone)]
pub struct APIGateway {
    /// API key manager
    key_manager: Arc<APIKeyManager>,
    /// Rate limiter
    rate_limiter: Arc<APIRateLimiter>,
    /// CORS policy
    cors_policy: CORSPolicy,
    /// Route configurations
    routes: HashMap<String, RouteConfig>,
}

impl APIGateway {
    /// Create a new API gateway
    pub fn new(
        key_manager: Arc<APIKeyManager>,
        rate_limiter: Arc<APIRateLimiter>,
        cors_policy: CORSPolicy,
    ) -> Self {
        Self {
            key_manager,
            rate_limiter,
            cors_policy,
            routes: HashMap::new(),
        }
    }

    /// Register a route
    pub fn register_route(&mut self, config: RouteConfig) {
        self.routes.insert(config.path.clone(), config);
    }

    /// Process an API request
    pub fn process_request(
        &self,
        path: &str,
        method: &HttpMethod,
        api_key: Option<&str>,
        origin: Option<&str>,
    ) -> Result<(), APIGatewayError> {
        // Find route
        let route = self
            .routes
            .get(path)
            .ok_or_else(|| APIGatewayError::RouteNotFound(path.to_string()))?;

        // Check method
        if !route.methods.contains(method) {
            return Err(APIGatewayError::MethodNotAllowed(format!("{:?}", method)));
        }

        // Check authentication
        let validated_key = if route.auth_required {
            let key = api_key.ok_or(APIGatewayError::AuthenticationRequired)?;
            Some(
                self.key_manager
                    .validate_key(key)
                    .map_err(|e| APIGatewayError::AuthorizationFailed(e.to_string()))?,
            )
        } else {
            None
        };

        // Check scopes
        if let Some(key) = &validated_key {
            for required_scope in &route.required_scopes {
                if !key.has_scope(required_scope) {
                    return Err(APIGatewayError::AuthorizationFailed(format!(
                        "Missing scope: {}",
                        required_scope
                    )));
                }
            }
        }

        // Check rate limit
        let client_id = validated_key
            .as_ref()
            .map(|k| k.client_id.as_str())
            .unwrap_or("anonymous");

        self.rate_limiter
            .check_request(client_id, path)
            .map_err(|e| APIGatewayError::RateLimitExceeded(e.to_string()))?;

        // Check CORS
        if let Some(origin) = origin {
            if !self.cors_policy.is_origin_allowed(origin) {
                return Err(APIGatewayError::CORSViolation(format!(
                    "Origin not allowed: {}",
                    origin
                )));
            }
        }

        Ok(())
    }

    /// Get CORS headers for a response
    pub fn get_cors_headers(&self, origin: Option<&str>) -> Vec<(String, String)> {
        self.cors_policy.get_cors_headers(origin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::RateLimit;

    #[test]
    fn test_api_gateway_creation() {
        let key_manager = Arc::new(APIKeyManager::default());
        let rate_limiter = Arc::new(APIRateLimiter::new(RateLimit::permissive()));
        let cors_policy = CORSPolicy::permissive();

        let gateway = APIGateway::new(key_manager, rate_limiter, cors_policy);
        assert_eq!(gateway.routes.len(), 0);
    }

    #[test]
    fn test_route_registration() {
        let key_manager = Arc::new(APIKeyManager::default());
        let rate_limiter = Arc::new(APIRateLimiter::new(RateLimit::permissive()));
        let cors_policy = CORSPolicy::permissive();

        let mut gateway = APIGateway::new(key_manager, rate_limiter, cors_policy);

        let route = RouteConfig {
            path: "/api/users".to_string(),
            methods: vec![HttpMethod::GET, HttpMethod::POST],
            auth_required: false,
            required_scopes: vec![],
            rate_limit: None,
        };

        gateway.register_route(route);
        assert_eq!(gateway.routes.len(), 1);
    }

    #[test]
    fn test_process_request_public_route() {
        let key_manager = Arc::new(APIKeyManager::default());
        let rate_limiter = Arc::new(APIRateLimiter::new(RateLimit::permissive()));
        let cors_policy = CORSPolicy::permissive();

        let mut gateway = APIGateway::new(key_manager, rate_limiter, cors_policy);

        let route = RouteConfig {
            path: "/api/public".to_string(),
            methods: vec![HttpMethod::GET],
            auth_required: false,
            required_scopes: vec![],
            rate_limit: None,
        };

        gateway.register_route(route);

        let result = gateway.process_request("/api/public", &HttpMethod::GET, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_request_requires_auth() {
        let key_manager = Arc::new(APIKeyManager::default());
        let rate_limiter = Arc::new(APIRateLimiter::new(RateLimit::permissive()));
        let cors_policy = CORSPolicy::permissive();

        let mut gateway = APIGateway::new(key_manager, rate_limiter, cors_policy);

        let route = RouteConfig {
            path: "/api/private".to_string(),
            methods: vec![HttpMethod::GET],
            auth_required: true,
            required_scopes: vec![],
            rate_limit: None,
        };

        gateway.register_route(route);

        let result = gateway.process_request("/api/private", &HttpMethod::GET, None, None);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), APIGatewayError::AuthenticationRequired));
    }
}
