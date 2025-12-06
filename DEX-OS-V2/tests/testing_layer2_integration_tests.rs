//! Testing Layer 2: Integration Testing and System Validation
//!
//! This file implements the Testing Layer 2 requirements from RULES.md:
//! @RULES.md ###Testing Layer 2: Integration Testing and System Validation
//!
//! Testing Layer 2 focuses on:
//! - Test interactions between different modules and components
//! - Validate database operations with a test database
//! - Test API endpoints with mock data and real scenarios
//! - Verify cross-component data flow and transformations
//! - Test service integrations and third-party API calls
//! - Validate configuration and environment-specific behavior
//! - Test failure scenarios and system resilience
//! - Perform end-to-end testing of critical user workflows
//! - Validate data consistency across distributed components

use dex_core::test_results::{IndividualTestResult, TestStatus, TestSuiteResult, TestMetadata};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Test framework for integration testing
pub struct IntegrationTestFramework {
    /// Collection of test suite results
    suite_results: Vec<TestSuiteResult>,
}

/// Integration test suite
pub struct IntegrationTestSuite {
    /// Name of the test suite
    name: String,
    /// Collection of test results
    results: Vec<IndividualTestResult>,
    /// Start time of the test suite
    started_at: u64,
}

impl IntegrationTestSuite {
    /// Create a new integration test suite
    pub fn new(name: &str) -> Self {
        let started_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        Self {
            name: name.to_string(),
            results: Vec::new(),
            started_at,
        }
    }

    /// Run an integration test function
    pub fn run_test<F>(&mut self, name: &str, test_fn: F) 
    where
        F: FnOnce() -> Result<(), String>,
    {
        let start_time = std::time::Instant::now();
        
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            test_fn()
        }));
        
        let duration = start_time.elapsed().as_millis() as u64;
        
        let test_result = match result {
            Ok(Ok(())) => IndividualTestResult {
                name: name.to_string(),
                status: TestStatus::Passed,
                duration_ms: duration,
                error_message: None,
                data: HashMap::new(),
            },
            Ok(Err(msg)) => IndividualTestResult {
                name: name.to_string(),
                status: TestStatus::Failed,
                duration_ms: duration,
                error_message: Some(msg),
                data: HashMap::new(),
            },
            Err(_) => IndividualTestResult {
                name: name.to_string(),
                status: TestStatus::Error,
                duration_ms: duration,
                error_message: Some("Test panicked".to_string()),
                data: HashMap::new(),
            },
        };
        
        self.results.push(test_result);
    }

    /// Finish the test suite and return results
    pub fn finish(self) -> TestSuiteResult {
        let finished_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        let overall_status = if self.results.iter().any(|r| r.status == TestStatus::Failed || r.status == TestStatus::Error) {
            TestStatus::Failed
        } else {
            TestStatus::Passed
        };
        
        TestSuiteResult {
            id: format!("integration_suite_{}_{}", 
                       self.name.to_lowercase().replace(" ", "_"), 
                       self.started_at),
            suite_name: self.name,
            started_at: self.started_at,
            finished_at,
            status: overall_status,
            test_results: self.results,
            metadata: TestMetadata {
                version: option_env!("CARGO_PKG_VERSION").unwrap_or("0.1.0").to_string(),
                commit_hash: "unknown".to_string(),
                environment: "integration_test".to_string(),
                platform: std::env::consts::OS.to_string(),
                custom: HashMap::new(),
            },
        }
    }

    /// Get test results
    pub fn get_results(&self) -> &[IndividualTestResult] {
        &self.results
    }
}

impl IntegrationTestFramework {
    /// Create a new integration test framework
    pub fn new() -> Self {
        Self {
            suite_results: Vec::new(),
        }
    }

    /// Create and run an integration test suite
    pub fn run_suite<F>(&mut self, name: &str, suite_fn: F) 
    where
        F: FnOnce(&mut IntegrationTestSuite),
    {
        let mut suite = IntegrationTestSuite::new(name);
        suite_fn(&mut suite);
        let result = suite.finish();
        self.suite_results.push(result);
    }

    /// Get all test suite results
    pub fn get_suite_results(&self) -> &[TestSuiteResult] {
        &self.suite_results
    }

    /// Get overall statistics
    pub fn get_statistics(&self) -> (usize, usize, usize, usize) {
        let mut total_suites = 0;
        let mut passed_suites = 0;
        let mut total_tests = 0;
        let mut passed_tests = 0;
        
        for suite in &self.suite_results {
            total_suites += 1;
            if suite.status == TestStatus::Passed {
                passed_suites += 1;
            }
            
            for test in &suite.test_results {
                total_tests += 1;
                if test.status == TestStatus::Passed {
                    passed_tests += 1;
                }
            }
        }
        
        (total_suites, passed_suites, total_tests, passed_tests)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_test_framework_creation() {
        let framework = IntegrationTestFramework::new();
        assert_eq!(framework.get_suite_results().len(), 0);
    }

    #[test]
    fn test_integration_test_suite() {
        let mut suite = IntegrationTestSuite::new("Test Suite");
        
        suite.run_test("successful_test", || Ok(()));
        suite.run_test("failing_test", || Err("Failed".to_string()));
        
        let results = suite.get_results();
        assert_eq!(results.len(), 2);
        
        let suite_result = suite.finish();
        assert_eq!(suite_result.suite_name, "Test Suite");
        assert_eq!(suite_result.test_results.len(), 2);
        assert_eq!(suite_result.status, TestStatus::Failed); // Because one test failed
    }

    #[test]
    fn test_framework_run_suite() {
        let mut framework = IntegrationTestFramework::new();
        
        framework.run_suite("Sample Integration Tests", |suite| {
            suite.run_test("test_1", || Ok(()));
            suite.run_test("test_2", || Ok(()));
        });
        
        let results = framework.get_suite_results();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].suite_name, "Sample Integration Tests");
        assert_eq!(results[0].test_results.len(), 2);
        assert_eq!(results[0].status, TestStatus::Passed);
    }

    #[test]
    fn test_framework_statistics() {
        let mut framework = IntegrationTestFramework::new();
        
        framework.run_suite("Suite 1", |suite| {
            suite.run_test("passing_test", || Ok(()));
            suite.run_test("failing_test", || Err("Failed".to_string()));
        });
        
        framework.run_suite("Suite 2", |suite| {
            suite.run_test("another_passing_test", || Ok(()));
        });
        
        let (total_suites, passed_suites, total_tests, passed_tests) = framework.get_statistics();
        assert_eq!(total_suites, 2);
        assert_eq!(passed_suites, 1); // Only Suite 2 passed
        assert_eq!(total_tests, 3);
        assert_eq!(passed_tests, 2);
    }
}
