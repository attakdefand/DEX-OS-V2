use std::sync::{Arc, RwLock};
use thiserror::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Errors related to load validation
#[derive(Debug, Error)]
pub enum LoadValidationError {
    #[error("Test failed: {0}")]
    TestFailed(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    #[error("Timeout waiting for test completion")]
    Timeout,
}

/// Configuration for a load test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadTestConfig {
    pub name: String,
    pub target_component: String,
    pub users: u32,
    pub duration_seconds: u64,
    pub requests_per_second: u32,
    pub ramp_up_seconds: u64,
}

/// Result of a load test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadTestResult {
    pub test_id: String,
    pub config: LoadTestConfig,
    pub timestamp: u64,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub max_latency_ms: f64,
    pub throughput_rps: f64,
    pub errors: Vec<String>,
    pub passed: bool,
}

/// Manager for Load Validation and Performance Testing (Security Layer 11 - DevSecOps)
#[derive(Debug, Clone)]
pub struct LoadValidationManager {
    /// History of test results
    results: Arc<RwLock<Vec<LoadTestResult>>>,
    /// Active tests (simulated)
    active_tests: Arc<RwLock<HashMap<String, LoadTestConfig>>>,
}

impl LoadValidationManager {
    pub fn new() -> Self {
        Self {
            results: Arc::new(RwLock::new(Vec::new())),
            active_tests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Run a synchronous load test (simulated)
    pub fn run_load_test(&self, config: LoadTestConfig) -> Result<LoadTestResult, LoadValidationError> {
        // In a real system, this would spawn threads/tasks to generate load.
        // Here we simulate the result based on the config.
        
        let test_id = format!("test_{}_{}", config.name, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
        
        // Simulate processing time
        // std::thread::sleep(std::time::Duration::from_millis(100)); 

        // Simulate results
        let success_rate = if config.requests_per_second > 10000 { 0.95 } else { 0.9999 };
        let avg_latency = if config.users > 1000 { 50.0 } else { 10.0 };
        
        let result = LoadTestResult {
            test_id: test_id.clone(),
            config: config.clone(),
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            success_rate,
            avg_latency_ms: avg_latency,
            p95_latency_ms: avg_latency * 1.5,
            p99_latency_ms: avg_latency * 2.0,
            max_latency_ms: avg_latency * 5.0,
            throughput_rps: config.requests_per_second as f64 * success_rate,
            errors: if success_rate < 1.0 { vec!["Connection timeout".to_string()] } else { vec![] },
            passed: success_rate > 0.99 && avg_latency < 100.0,
        };

        let mut results = self.results.write().map_err(|_| LoadValidationError::TestFailed("Lock error".into()))?;
        results.push(result.clone());

        Ok(result)
    }

    /// Get all test results
    pub fn get_results(&self) -> Vec<LoadTestResult> {
        self.results.read().unwrap().clone()
    }

    /// Get results for a specific component
    pub fn get_component_results(&self, component: &str) -> Vec<LoadTestResult> {
        self.results.read().unwrap().iter()
            .filter(|r| r.config.target_component == component)
            .cloned()
            .collect()
    }
}

impl Default for LoadValidationManager {
    fn default() -> Self {
        Self::new()
    }
}
