//! Testing Layer 1: Unit Testing and Component Validation
//!
//! This file implements the Testing Layer 1 requirements from RULES.md:
//! @RULES.md ###Testing Layer 1: Unit Testing and Component Validation
//!
//! Testing Layer 1 focuses on:
//! - Write unit tests for all public functions and methods
//! - Test both happy path and error conditions
//! - Maintain high code coverage (target 80% or higher)
//! - Use property-based testing for mathematical functions and algorithms
//! - Implement test-driven development (TDD) where appropriate
//! - Mock external dependencies to isolate units under test
//! - Validate input parameter boundaries and edge cases
//! - Test error handling and recovery mechanisms
//! - Document test cases with clear descriptions of expected behavior

use dex_core::test_results::{IndividualTestResult, TestStatus};
use std::collections::HashMap;

/// Test framework for unit testing
pub struct UnitTestFramework {
    /// Collection of test results
    results: Vec<IndividualTestResult>,
}

impl UnitTestFramework {
    /// Create a new unit test framework
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    /// Run a unit test function
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

    /// Assert that two values are equal
    pub fn assert_eq<T: PartialEq + std::fmt::Debug>(left: T, right: T) -> Result<(), String> {
        if left == right {
            Ok(())
        } else {
            Err(format!("Assertion failed: {:?} != {:?}", left, right))
        }
    }

    /// Assert that a condition is true
    pub fn assert_true(condition: bool, message: &str) -> Result<(), String> {
        if condition {
            Ok(())
        } else {
            Err(message.to_string())
        }
    }

    /// Assert that a result is OK
    pub fn assert_ok<T, E: std::fmt::Debug>(result: Result<T, E>) -> Result<(), String> {
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Expected Ok, got Err: {:?}", e)),
        }
    }

    /// Assert that a result is Err
    pub fn assert_err<T: std::fmt::Debug, E>(result: Result<T, E>) -> Result<(), String> {
        match result {
            Ok(t) => Err(format!("Expected Err, got Ok: {:?}", t)),
            Err(_) => Ok(()),
        }
    }

    /// Get test results
    pub fn get_results(&self) -> &[IndividualTestResult] {
        &self.results
    }

    /// Get test statistics
    pub fn get_statistics(&self) -> (usize, usize, usize) {
        let passed = self.results.iter().filter(|r| r.status == TestStatus::Passed).count();
        let failed = self.results.iter().filter(|r| r.status == TestStatus::Failed).count();
        let errored = self.results.iter().filter(|r| r.status == TestStatus::Error).count();
        (passed, failed, errored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_test_framework_creation() {
        let framework = UnitTestFramework::new();
        assert_eq!(framework.get_results().len(), 0);
    }

    #[test]
    fn test_successful_test() {
        let mut framework = UnitTestFramework::new();
        
        framework.run_test("successful_test", || {
            UnitTestFramework::assert_true(true, "Should be true")
        });
        
        let results = framework.get_results();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, TestStatus::Passed);
    }

    #[test]
    fn test_failed_test() {
        let mut framework = UnitTestFramework::new();
        
        framework.run_test("failed_test", || {
            UnitTestFramework::assert_true(false, "Should fail")
        });
        
        let results = framework.get_results();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, TestStatus::Failed);
    }

    #[test]
    fn test_assertion_helpers() {
        // Test assert_eq
        assert!(UnitTestFramework::assert_eq(1, 1).is_ok());
        assert!(UnitTestFramework::assert_eq(1, 2).is_err());
        
        // Test assert_true
        assert!(UnitTestFramework::assert_true(true, "msg").is_ok());
        assert!(UnitTestFramework::assert_true(false, "msg").is_err());
        
        // Test assert_ok
        assert!(UnitTestFramework::assert_ok(Ok::<i32, &str>(1)).is_ok());
        assert!(UnitTestFramework::assert_ok(Err::<i32, &str>("error")).is_err());
        
        // Test assert_err
        assert!(UnitTestFramework::assert_err(Err::<i32, &str>("error")).is_ok());
        assert!(UnitTestFramework::assert_err(Ok::<i32, &str>(1)).is_err());
    }

    #[test]
    fn test_statistics() {
        let mut framework = UnitTestFramework::new();
        
        framework.run_test("passing_test", || Ok(()));
        framework.run_test("failing_test", || Err("Failed".to_string()));
        
        let (passed, failed, errored) = framework.get_statistics();
        assert_eq!(passed, 1);
        assert_eq!(failed, 1);
        assert_eq!(errored, 0);
    }
}