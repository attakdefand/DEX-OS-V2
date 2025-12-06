//! Service Mesh implementation for DEX-OS Network Security
//!
//! Implements Security Layer 6 - Network & Infrastructure Security
//! From DEX-OS-V2.csv line 227:
//! - Infrastructure,Network,Network,Hash Map,Service Mesh,High {Security: Layer 2 - Network Security}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use thiserror::Error;

/// Service mesh error types
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ServiceMeshError {
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
    #[error("Service already exists: {0}")]
    ServiceAlreadyExists(String),
    #[error("Endpoint not found: {0}")]
    EndpointNotFound(String),
    #[error("Invalid service configuration: {0}")]
    InvalidConfiguration(String),
    #[error("Circuit breaker open for service: {0}")]
    CircuitBreakerOpen(String),
}

/// Service health status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Load balancing strategy
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastConnections,
    Random,
    WeightedRoundRobin,
    IpHash,
}

/// Circuit breaker state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

/// Service endpoint definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    /// Endpoint ID
    pub id: String,
    /// IP address
    pub ip: IpAddr,
    /// Port
    pub port: u16,
    /// Health status
    pub health: HealthStatus,
    /// Weight for load balancing
    pub weight: u32,
    /// Active connections
    pub active_connections: u32,
    /// Total requests served
    pub total_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Last health check timestamp
    pub last_health_check: u64,
    /// Response time (milliseconds)
    pub avg_response_time_ms: u64,
}

impl ServiceEndpoint {
    pub fn new(id: String, ip: IpAddr, port: u16, weight: u32) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id,
            ip,
            port,
            health: HealthStatus::Unknown,
            weight,
            active_connections: 0,
            total_requests: 0,
            failed_requests: 0,
            last_health_check: now,
            avg_response_time_ms: 0,
        }
    }

    pub fn is_healthy(&self) -> bool {
        matches!(self.health, HealthStatus::Healthy | HealthStatus::Degraded)
    }
}

/// Circuit breaker configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CircuitBreaker {
    /// Current state
    pub state: CircuitBreakerState,
    /// Failure threshold to open circuit
    pub failure_threshold: u32,
    /// Current failure count
    pub failure_count: u32,
    /// Success threshold to close circuit
    pub success_threshold: u32,
    /// Current success count (in half-open state)
    pub success_count: u32,
    /// Timeout before attempting to close (seconds)
    pub timeout_seconds: u64,
    /// Last state change timestamp
    pub last_state_change: u64,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, success_threshold: u32, timeout_seconds: u64) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            state: CircuitBreakerState::Closed,
            failure_threshold,
            failure_count: 0,
            success_threshold,
            success_count: 0,
            timeout_seconds,
            last_state_change: now,
        }
    }

    pub fn record_success(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        match self.state {
            CircuitBreakerState::Closed => {
                self.failure_count = 0;
            }
            CircuitBreakerState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.success_threshold {
                    self.state = CircuitBreakerState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                    self.last_state_change = now;
                }
            }
            CircuitBreakerState::Open => {}
        }
    }

    pub fn record_failure(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        match self.state {
            CircuitBreakerState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitBreakerState::Open;
                    self.last_state_change = now;
                }
            }
            CircuitBreakerState::HalfOpen => {
                self.state = CircuitBreakerState::Open;
                self.success_count = 0;
                self.last_state_change = now;
            }
            CircuitBreakerState::Open => {}
        }
    }

    pub fn can_attempt(&mut self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::HalfOpen => true,
            CircuitBreakerState::Open => {
                if now - self.last_state_change >= self.timeout_seconds {
                    self.state = CircuitBreakerState::HalfOpen;
                    self.success_count = 0;
                    self.last_state_change = now;
                    true
                } else {
                    false
                }
            }
        }
    }
}

/// Service definition
#[derive(Debug, Clone)]
pub struct Service {
    /// Service name
    pub name: String,
    /// Service description
    pub description: String,
    /// Service endpoints
    pub endpoints: Vec<ServiceEndpoint>,
    /// Load balancing strategy
    pub load_balancing: LoadBalancingStrategy,
    /// Circuit breaker
    pub circuit_breaker: CircuitBreaker,
    /// Current round-robin index
    round_robin_index: usize,
    /// Service metadata
    pub metadata: HashMap<String, String>,
    /// Service creation timestamp
    pub created_at: u64,
}

impl Service {
    pub fn new(
        name: String,
        description: String,
        load_balancing: LoadBalancingStrategy,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            name,
            description,
            endpoints: Vec::new(),
            load_balancing,
            circuit_breaker: CircuitBreaker::new(5, 3, 60),
            round_robin_index: 0,
            metadata: HashMap::new(),
            created_at: now,
        }
    }

    pub fn add_endpoint(&mut self, endpoint: ServiceEndpoint) {
        self.endpoints.push(endpoint);
    }

    pub fn remove_endpoint(&mut self, endpoint_id: &str) -> Result<(), ServiceMeshError> {
        let initial_len = self.endpoints.len();
        self.endpoints.retain(|e| e.id != endpoint_id);
        
        if self.endpoints.len() == initial_len {
            Err(ServiceMeshError::EndpointNotFound(endpoint_id.to_string()))
        } else {
            Ok(())
        }
    }

    pub fn get_healthy_endpoints(&self) -> Vec<&ServiceEndpoint> {
        self.endpoints.iter().filter(|e| e.is_healthy()).collect()
    }

    pub fn select_endpoint(&mut self) -> Option<&mut ServiceEndpoint> {
        let healthy_endpoints: Vec<usize> = self
            .endpoints
            .iter()
            .enumerate()
            .filter(|(_, e)| e.is_healthy())
            .map(|(i, _)| i)
            .collect();

        if healthy_endpoints.is_empty() {
            return None;
        }

        let selected_index = match self.load_balancing {
            LoadBalancingStrategy::RoundRobin => {
                let index = self.round_robin_index % healthy_endpoints.len();
                self.round_robin_index += 1;
                healthy_endpoints[index]
            }
            LoadBalancingStrategy::LeastConnections => {
                *healthy_endpoints
                    .iter()
                    .min_by_key(|&&i| self.endpoints[i].active_connections)
                    .unwrap()
            }
            LoadBalancingStrategy::Random => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                healthy_endpoints[rng.gen_range(0..healthy_endpoints.len())]
            }
            LoadBalancingStrategy::WeightedRoundRobin => {
                // Simplified weighted round-robin
                let total_weight: u32 = healthy_endpoints
                    .iter()
                    .map(|&i| self.endpoints[i].weight)
                    .sum();
                let mut target = (self.round_robin_index as u32) % total_weight;
                self.round_robin_index += 1;

                let mut selected = healthy_endpoints[0];
                for &i in &healthy_endpoints {
                    if target < self.endpoints[i].weight {
                        selected = i;
                        break;
                    }
                    target -= self.endpoints[i].weight;
                }
                selected
            }
            LoadBalancingStrategy::IpHash => {
                // For IP hash, we'd need the client IP, so we'll use round-robin as fallback
                let index = self.round_robin_index % healthy_endpoints.len();
                self.round_robin_index += 1;
                healthy_endpoints[index]
            }
        };

        Some(&mut self.endpoints[selected_index])
    }
}

/// Service Mesh Manager using Hash Map for efficient service registry
#[derive(Debug, Clone)]
pub struct ServiceMeshManager {
    /// Services registry (Hash Map for O(1) lookup)
    services: HashMap<String, Service>,
    /// Service discovery cache
    discovery_cache: HashMap<String, Vec<String>>,
    /// Statistics
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
}

impl ServiceMeshManager {
    /// Create a new service mesh manager
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
            discovery_cache: HashMap::new(),
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
        }
    }

    /// Register a new service
    pub fn register_service(&mut self, service: Service) -> Result<(), ServiceMeshError> {
        if self.services.contains_key(&service.name) {
            return Err(ServiceMeshError::ServiceAlreadyExists(service.name.clone()));
        }

        self.services.insert(service.name.clone(), service);
        Ok(())
    }

    /// Unregister a service
    pub fn unregister_service(&mut self, service_name: &str) -> Result<(), ServiceMeshError> {
        self.services
            .remove(service_name)
            .ok_or_else(|| ServiceMeshError::ServiceNotFound(service_name.to_string()))?;
        
        self.discovery_cache.remove(service_name);
        Ok(())
    }

    /// Get a service by name
    pub fn get_service(&self, service_name: &str) -> Option<&Service> {
        self.services.get(service_name)
    }

    /// Get a mutable service by name
    pub fn get_service_mut(&mut self, service_name: &str) -> Option<&mut Service> {
        self.services.get_mut(service_name)
    }

    /// Discover services by tag
    pub fn discover_services(&self, tag: &str) -> Vec<&Service> {
        self.services
            .values()
            .filter(|s| s.metadata.get("tags").map_or(false, |tags| tags.contains(tag)))
            .collect()
    }

    /// Select an endpoint for a service
    pub fn select_endpoint(
        &mut self,
        service_name: &str,
    ) -> Result<&ServiceEndpoint, ServiceMeshError> {
        let service = self
            .services
            .get_mut(service_name)
            .ok_or_else(|| ServiceMeshError::ServiceNotFound(service_name.to_string()))?;

        if !service.circuit_breaker.can_attempt() {
            return Err(ServiceMeshError::CircuitBreakerOpen(service_name.to_string()));
        }

        let endpoint = service
            .select_endpoint()
            .ok_or_else(|| ServiceMeshError::EndpointNotFound("No healthy endpoints".to_string()))?;

        // Return immutable reference (we need to work around the mutable borrow)
        let endpoint_id = endpoint.id.clone();
        let service = self.services.get(service_name).unwrap();
        Ok(service.endpoints.iter().find(|e| e.id == endpoint_id).unwrap())
    }

    /// Record a successful request
    pub fn record_success(&mut self, service_name: &str, endpoint_id: &str) {
        self.total_requests += 1;
        self.successful_requests += 1;

        if let Some(service) = self.services.get_mut(service_name) {
            service.circuit_breaker.record_success();
            if let Some(endpoint) = service.endpoints.iter_mut().find(|e| e.id == endpoint_id) {
                endpoint.total_requests += 1;
            }
        }
    }

    /// Record a failed request
    pub fn record_failure(&mut self, service_name: &str, endpoint_id: &str) {
        self.total_requests += 1;
        self.failed_requests += 1;

        if let Some(service) = self.services.get_mut(service_name) {
            service.circuit_breaker.record_failure();
            if let Some(endpoint) = service.endpoints.iter_mut().find(|e| e.id == endpoint_id) {
                endpoint.failed_requests += 1;
            }
        }
    }

    /// Get all services
    pub fn get_all_services(&self) -> Vec<&Service> {
        self.services.values().collect()
    }

    /// Get service mesh statistics
    pub fn get_statistics(&self) -> ServiceMeshStatistics {
        ServiceMeshStatistics {
            total_services: self.services.len(),
            total_endpoints: self.services.values().map(|s| s.endpoints.len()).sum(),
            healthy_endpoints: self
                .services
                .values()
                .flat_map(|s| &s.endpoints)
                .filter(|e| e.is_healthy())
                .count(),
            total_requests: self.total_requests,
            successful_requests: self.successful_requests,
            failed_requests: self.failed_requests,
            success_rate: if self.total_requests > 0 {
                (self.successful_requests as f64 / self.total_requests as f64) * 100.0
            } else {
                0.0
            },
        }
    }
}

/// Service mesh statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceMeshStatistics {
    pub total_services: usize,
    pub total_endpoints: usize,
    pub healthy_endpoints: usize,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub success_rate: f64,
}

impl Default for ServiceMeshManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_service_creation() {
        let service = Service::new(
            "api-service".to_string(),
            "API Gateway".to_string(),
            LoadBalancingStrategy::RoundRobin,
        );

        assert_eq!(service.name, "api-service");
        assert_eq!(service.endpoints.len(), 0);
    }

    #[test]
    fn test_circuit_breaker() {
        let mut cb = CircuitBreaker::new(3, 2, 60);
        assert_eq!(cb.state, CircuitBreakerState::Closed);

        // Record failures to open circuit
        cb.record_failure();
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state, CircuitBreakerState::Open);
    }

    #[test]
    fn test_service_mesh_registration() {
        let mut mesh = ServiceMeshManager::new();
        let service = Service::new(
            "test-service".to_string(),
            "Test Service".to_string(),
            LoadBalancingStrategy::RoundRobin,
        );

        assert!(mesh.register_service(service).is_ok());
        assert_eq!(mesh.get_statistics().total_services, 1);
    }
}
