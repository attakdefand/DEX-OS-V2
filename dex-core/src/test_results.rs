//! Test Results Storage implementation for the DEX-OS core engine
//!
//! This module implements the Priority 3 testing feature from DEX-OS-V2.csv:
//! - Testing,Testing,Testing,Hash Map,Test Result Storage,Medium

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use serde::{Deserialize, Serialize};

/// Test results storage manager
#[derive(Debug, Clone)]
pub struct TestResultsManager {
    /// Storage for test results using Hash Map
    results: HashMap<String, TestSuiteResult>,
    /// Index for quick lookup by test name
    name_index: HashMap<String, String>,
    /// Index for quick lookup by status
    status_index: HashMap<TestStatus, Vec<String>>,
}

/// Result of a test suite execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteResult {
    /// Unique identifier for this test suite run
    pub id: String,
    /// Name of the test suite
    pub suite_name: String,
    /// Timestamp when the test suite started
    pub started_at: u64,
    /// Timestamp when the test suite finished
    pub finished_at: u64,
    /// Overall status of the test suite
    pub status: TestStatus,
    /// Individual test results
    pub test_results: Vec<IndividualTestResult>,
    /// Metadata about the test environment
    pub metadata: TestMetadata,
}

/// Status of a test or test suite
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TestStatus {
    /// Test passed
    Passed,
    /// Test failed
    Failed,
    /// Test was skipped
    Skipped,
    /// Test is still running
    Running,
    /// Test encountered an error
    Error,
}

/// Result of an individual test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndividualTestResult {
    /// Name of the test
    pub name: String,
    /// Status of the test
    pub status: TestStatus,
    /// Duration of the test in milliseconds
    pub duration_ms: u64,
    /// Error message if the test failed
    pub error_message: Option<String>,
    /// Additional test data
    pub data: HashMap<String, String>,
}

/// Metadata about a test run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMetadata {
    /// Version of the software being tested
    pub version: String,
    /// Git commit hash
    pub commit_hash: String,
    /// Environment where tests were run
    pub environment: String,
    /// Platform information
    pub platform: String,
    /// Additional custom metadata
    pub custom: HashMap<String, String>,
}

impl TestResultsManager {
    /// Create a new test results manager
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
            name_index: HashMap::new(),
            status_index: HashMap::new(),
        }
    }

    /// Store a test suite result
    pub fn store_result(&mut self, result: TestSuiteResult) -> Result<(), TestResultsError> {
        let suite_id = result.id.clone();
        let suite_name = result.suite_name.clone();
        let status = result.status.clone();
        
        // Store the result
        self.results.insert(suite_id.clone(), result);
        
        // Update indexes
        self.name_index.insert(suite_name, suite_id.clone());
        
        // Update status index
        self.status_index
            .entry(status)
            .or_insert_with(Vec::new)
            .push(suite_id);
        
        Ok(())
    }

    /// Get a test suite result by ID
    pub fn get_result(&self, id: &str) -> Option<&TestSuiteResult> {
        self.results.get(id)
    }

    /// Get a test suite result by name
    pub fn get_result_by_name(&self, name: &str) -> Option<&TestSuiteResult> {
        if let Some(id) = self.name_index.get(name) {
            self.results.get(id)
        } else {
            None
        }
    }

    /// Get all test suite results with a specific status
    pub fn get_results_by_status(&self, status: TestStatus) -> Vec<&TestSuiteResult> {
        if let Some(ids) = self.status_index.get(&status) {
            ids.iter()
                .filter_map(|id| self.results.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all test suite results
    pub fn get_all_results(&self) -> Vec<&TestSuiteResult> {
        self.results.values().collect()
    }

    /// Get test statistics
    pub fn get_statistics(&self) -> TestStatistics {
        let mut stats = TestStatistics {
            total_suites: self.results.len(),
            passed_suites: 0,
            failed_suites: 0,
            skipped_suites: 0,
            running_suites: 0,
            errored_suites: 0,
            total_tests: 0,
            passed_tests: 0,
            failed_tests: 0,
            skipped_tests: 0,
            errored_tests: 0,
            average_duration_ms: 0,
        };
        
        let mut total_duration = 0u64;
        
        for result in self.results.values() {
            match result.status {
                TestStatus::Passed => stats.passed_suites += 1,
                TestStatus::Failed => stats.failed_suites += 1,
                TestStatus::Skipped => stats.skipped_suites += 1,
                TestStatus::Running => stats.running_suites += 1,
                TestStatus::Error => stats.errored_suites += 1,
            }
            
            for test in &result.test_results {
                stats.total_tests += 1;
                total_duration += test.duration_ms;
                
                match test.status {
                    TestStatus::Passed => stats.passed_tests += 1,
                    TestStatus::Failed => stats.failed_tests += 1,
                    TestStatus::Skipped => stats.skipped_tests += 1,
                    TestStatus::Running => stats.running_suites += 1,
                    TestStatus::Error => stats.errored_tests += 1,
                }
            }
        }
        
        if stats.total_tests > 0 {
            stats.average_duration_ms = total_duration / stats.total_tests as u64;
        }
        
        stats
    }

    /// Remove old test results (older than specified seconds)
    pub fn remove_old_results(&mut self, older_than_seconds: u64) -> usize {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let cutoff_time = now - older_than_seconds;
        
        let mut to_remove = Vec::new();
        
        // Find old results
        for (id, result) in &self.results {
            if result.finished_at < cutoff_time {
                to_remove.push(id.clone());
            }
        }
        
        let removed_count = to_remove.len();
        
        // Remove old results and update indexes
        for id in to_remove {
            if let Some(result) = self.results.remove(&id) {
                // Remove from name index
                self.name_index.retain(|_, v| *v != id);
                
                // Remove from status index
                if let Some(status_list) = self.status_index.get_mut(&result.status) {
                    status_list.retain(|x| *x != id);
                }
            }
        }
        
        removed_count
    }

    /// Clear all test results
    pub fn clear_all_results(&mut self) {
        self.results.clear();
        self.name_index.clear();
        self.status_index.clear();
    }
}

impl Default for TestResultsManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestStatistics {
    /// Total number of test suites
    pub total_suites: usize,
    /// Number of passed test suites
    pub passed_suites: usize,
    /// Number of failed test suites
    pub failed_suites: usize,
    /// Number of skipped test suites
    pub skipped_suites: usize,
    /// Number of running test suites
    pub running_suites: usize,
    /// Number of errored test suites
    pub errored_suites: usize,
    /// Total number of individual tests
    pub total_tests: usize,
    /// Number of passed individual tests
    pub passed_tests: usize,
    /// Number of failed individual tests
    pub failed_tests: usize,
    /// Number of skipped individual tests
    pub skipped_tests: usize,
    /// Number of errored individual tests
    pub errored_tests: usize,
    /// Average test duration in milliseconds
    pub average_duration_ms: u64,
}

/// Errors that can occur during test results operations
#[derive(Debug, Error)]
pub enum TestResultsError {
    #[error("Test result already exists")]
    ResultAlreadyExists,
    #[error("Test result not found")]
    ResultNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_results_manager_creation() {
        let manager = TestResultsManager::new();
        assert!(manager.results.is_empty());
        assert!(manager.name_index.is_empty());
        assert!(manager.status_index.is_empty());
    }

    #[test]
    fn test_store_and_retrieve_result() {
        let mut manager = TestResultsManager::new();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let result = TestSuiteResult {
            id: "suite_1".to_string(),
            suite_name: "Integration Tests".to_string(),
            started_at: now - 100,
            finished_at: now,
            status: TestStatus::Passed,
            test_results: vec![
                IndividualTestResult {
                    name: "test_order_matching".to_string(),
                    status: TestStatus::Passed,
                    duration_ms: 50,
                    error_message: None,
                    data: HashMap::new(),
                }
            ],
            metadata: TestMetadata {
                version: "1.0.0".to_string(),
                commit_hash: "abc123".to_string(),
                environment: "test".to_string(),
                platform: "linux".to_string(),
                custom: HashMap::new(),
            },
        };
        
        // Store result
        assert!(manager.store_result(result.clone()).is_ok());
        
        // Retrieve by ID
        let retrieved = manager.get_result("suite_1");
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, result.id);
        assert_eq!(retrieved.suite_name, result.suite_name);
        assert_eq!(retrieved.started_at, result.started_at);
        assert_eq!(retrieved.finished_at, result.finished_at);
        assert_eq!(retrieved.status, result.status);
        assert_eq!(retrieved.test_results.len(), result.test_results.len());
        assert_eq!(retrieved.metadata.version, result.metadata.version);
        
        // Retrieve by name
        let retrieved = manager.get_result_by_name("Integration Tests");
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, result.id);
        assert_eq!(retrieved.suite_name, result.suite_name);
        assert_eq!(retrieved.started_at, result.started_at);
        assert_eq!(retrieved.finished_at, result.finished_at);
        assert_eq!(retrieved.status, result.status);
        assert_eq!(retrieved.test_results.len(), result.test_results.len());
        assert_eq!(retrieved.metadata.version, result.metadata.version);
    }

    #[test]
    fn test_get_results_by_status() {
        let mut manager = TestResultsManager::new();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Add passed test suite
        let passed_result = TestSuiteResult {
            id: "suite_1".to_string(),
            suite_name: "Passed Tests".to_string(),
            started_at: now - 100,
            finished_at: now,
            status: TestStatus::Passed,
            test_results: vec![],
            metadata: TestMetadata {
                version: "1.0.0".to_string(),
                commit_hash: "abc123".to_string(),
                environment: "test".to_string(),
                platform: "linux".to_string(),
                custom: HashMap::new(),
            },
        };
        
        // Add failed test suite
        let failed_result = TestSuiteResult {
            id: "suite_2".to_string(),
            suite_name: "Failed Tests".to_string(),
            started_at: now - 100,
            finished_at: now,
            status: TestStatus::Failed,
            test_results: vec![],
            metadata: TestMetadata {
                version: "1.0.0".to_string(),
                commit_hash: "abc123".to_string(),
                environment: "test".to_string(),
                platform: "linux".to_string(),
                custom: HashMap::new(),
            },
        };
        
        manager.store_result(passed_result).unwrap();
        manager.store_result(failed_result).unwrap();
        
        // Get passed results
        let passed_results = manager.get_results_by_status(TestStatus::Passed);
        assert_eq!(passed_results.len(), 1);
        assert_eq!(passed_results[0].suite_name, "Passed Tests");
        
        // Get failed results
        let failed_results = manager.get_results_by_status(TestStatus::Failed);
        assert_eq!(failed_results.len(), 1);
        assert_eq!(failed_results[0].suite_name, "Failed Tests");
    }

    #[test]
    fn test_get_statistics() {
        let mut manager = TestResultsManager::new();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let result = TestSuiteResult {
            id: "suite_1".to_string(),
            suite_name: "Test Suite".to_string(),
            started_at: now - 100,
            finished_at: now,
            status: TestStatus::Passed,
            test_results: vec![
                IndividualTestResult {
                    name: "test_1".to_string(),
                    status: TestStatus::Passed,
                    duration_ms: 50,
                    error_message: None,
                    data: HashMap::new(),
                },
                IndividualTestResult {
                    name: "test_2".to_string(),
                    status: TestStatus::Failed,
                    duration_ms: 75,
                    error_message: Some("Assertion failed".to_string()),
                    data: HashMap::new(),
                },
                IndividualTestResult {
                    name: "test_3".to_string(),
                    status: TestStatus::Skipped,
                    duration_ms: 0,
                    error_message: None,
                    data: HashMap::new(),
                }
            ],
            metadata: TestMetadata {
                version: "1.0.0".to_string(),
                commit_hash: "abc123".to_string(),
                environment: "test".to_string(),
                platform: "linux".to_string(),
                custom: HashMap::new(),
            },
        };
        
        manager.store_result(result).unwrap();
        
        let stats = manager.get_statistics();
        assert_eq!(stats.total_suites, 1);
        assert_eq!(stats.passed_suites, 1);
        assert_eq!(stats.total_tests, 3);
        assert_eq!(stats.passed_tests, 1);
        assert_eq!(stats.failed_tests, 1);
        assert_eq!(stats.skipped_tests, 1);
        assert_eq!(stats.average_duration_ms, 41); // (50 + 75 + 0) / 3 = 41.666... -> 41
    }

    #[test]
    fn test_remove_old_results() {
        let mut manager = TestResultsManager::new();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Add old result (1 hour old)
        let old_result = TestSuiteResult {
            id: "old_suite".to_string(),
            suite_name: "Old Tests".to_string(),
            started_at: now - 3600,
            finished_at: now - 3500,
            status: TestStatus::Passed,
            test_results: vec![],
            metadata: TestMetadata {
                version: "1.0.0".to_string(),
                commit_hash: "abc123".to_string(),
                environment: "test".to_string(),
                platform: "linux".to_string(),
                custom: HashMap::new(),
            },
        };
        
        // Add recent result (1 minute old)
        let recent_result = TestSuiteResult {
            id: "recent_suite".to_string(),
            suite_name: "Recent Tests".to_string(),
            started_at: now - 60,
            finished_at: now - 30,
            status: TestStatus::Passed,
            test_results: vec![],
            metadata: TestMetadata {
                version: "1.0.0".to_string(),
                commit_hash: "abc123".to_string(),
                environment: "test".to_string(),
                platform: "linux".to_string(),
                custom: HashMap::new(),
            },
        };
        
        manager.store_result(old_result).unwrap();
        manager.store_result(recent_result).unwrap();
        
        // Remove results older than 30 minutes
        let removed_count = manager.remove_old_results(1800); // 30 minutes
        assert_eq!(removed_count, 1);
        
        // Check that only recent result remains
        assert!(manager.get_result("old_suite").is_none());
        assert!(manager.get_result("recent_suite").is_some());
    }

    #[test]
    fn test_clear_all_results() {
        let mut manager = TestResultsManager::new();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let result = TestSuiteResult {
            id: "suite_1".to_string(),
            suite_name: "Test Suite".to_string(),
            started_at: now - 100,
            finished_at: now,
            status: TestStatus::Passed,
            test_results: vec![],
            metadata: TestMetadata {
                version: "1.0.0".to_string(),
                commit_hash: "abc123".to_string(),
                environment: "test".to_string(),
                platform: "linux".to_string(),
                custom: HashMap::new(),
            },
        };
        
        manager.store_result(result).unwrap();
        assert_eq!(manager.results.len(), 1);
        
        manager.clear_all_results();
        assert!(manager.results.is_empty());
        assert!(manager.name_index.is_empty());
        assert!(manager.status_index.is_empty());
    }
}