//! Test Coverage implementation using Bloom Filters for the DEX-OS core engine
//!
//! This module implements the Priority 3 testing feature from DEX-OS-V2.csv:
//! - Testing,Testing,Testing,Bloom Filter (conceptual),Test Coverage,Medium

use crate::security::BloomFilter;
use std::collections::HashMap;

/// Test coverage tracker using Bloom Filters for efficient tracking of executed tests
#[derive(Debug, Clone)]
pub struct TestCoverageTracker {
    /// Bloom filter for tracking which tests have been executed
    executed_tests_filter: BloomFilter,
    /// Map of test names to execution counts for more detailed tracking
    execution_counts: HashMap<String, usize>,
    /// Total number of tests in the test suite
    total_tests: usize,
}

impl TestCoverageTracker {
    /// Create a new test coverage tracker
    pub fn new(total_tests: usize) -> Self {
        Self {
            executed_tests_filter: BloomFilter::new(5000, 5), // Larger filter for many tests
            execution_counts: HashMap::new(),
            total_tests,
        }
    }

    /// Mark a test as executed
    pub fn mark_test_executed(&mut self, test_name: &str) {
        self.executed_tests_filter.add(test_name);
        *self.execution_counts.entry(test_name.to_string()).or_insert(0) += 1;
    }

    /// Check if a test has been executed (may have false positives)
    pub fn is_test_executed(&self, test_name: &str) -> bool {
        self.executed_tests_filter.might_contain(test_name)
    }

    /// Get the execution count for a specific test
    pub fn get_execution_count(&self, test_name: &str) -> usize {
        *self.execution_counts.get(test_name).unwrap_or(&0)
    }

    /// Get coverage statistics
    pub fn get_coverage_stats(&self) -> TestCoverageStats {
        // For a more accurate count, we use the execution counts map
        // rather than relying on the Bloom filter which may have false positives
        let executed_count = self.execution_counts.len();
        let coverage_percentage = if self.total_tests > 0 {
            (executed_count as f64 / self.total_tests as f64) * 100.0
        } else {
            0.0
        };

        TestCoverageStats {
            total_tests: self.total_tests,
            executed_tests: executed_count,
            coverage_percentage,
        }
    }

    /// Get the list of executed tests (based on tracked execution counts)
    pub fn get_executed_tests(&self) -> Vec<String> {
        self.execution_counts.keys().cloned().collect()
    }

    /// Reset the coverage tracker
    pub fn reset(&mut self) {
        self.executed_tests_filter = BloomFilter::new(5000, 5);
        self.execution_counts.clear();
    }
}

/// Statistics about test coverage
#[derive(Debug, Clone)]
pub struct TestCoverageStats {
    /// Total number of tests in the test suite
    pub total_tests: usize,
    /// Number of tests that have been executed
    pub executed_tests: usize,
    /// Coverage percentage
    pub coverage_percentage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_coverage_tracker_creation() {
        let tracker = TestCoverageTracker::new(100);
        assert_eq!(tracker.total_tests, 100);
    }

    #[test]
    fn test_mark_test_executed() {
        let mut tracker = TestCoverageTracker::new(100);
        
        // Mark a test as executed
        tracker.mark_test_executed("test_security__governance_and_policy__policy__enforces__on_request");
        
        // Check that the test is marked as executed
        assert!(tracker.is_test_executed("test_security__governance_and_policy__policy__enforces__on_request"));
        
        // Check execution count
        assert_eq!(tracker.get_execution_count("test_security__governance_and_policy__policy__enforces__on_request"), 1);
    }

    #[test]
    fn test_get_coverage_stats() {
        let mut tracker = TestCoverageTracker::new(1000);
        
        // Mark some tests as executed
        tracker.mark_test_executed("test_security__governance_and_policy__policy__enforces__on_request");
        tracker.mark_test_executed("test_security__governance_and_policy__policy__validates__during_ci");
        tracker.mark_test_executed("test_security__governance_and_policy__scanner__enforces__on_request");
        
        let stats = tracker.get_coverage_stats();
        assert_eq!(stats.total_tests, 1000);
        assert_eq!(stats.executed_tests, 3);
        assert_eq!(stats.coverage_percentage, 0.3); // 3/1000 * 100 = 0.3%
    }

    #[test]
    fn test_get_executed_tests() {
        let mut tracker = TestCoverageTracker::new(100);
        
        // Mark some tests as executed
        tracker.mark_test_executed("test_1");
        tracker.mark_test_executed("test_2");
        tracker.mark_test_executed("test_3");
        
        let executed_tests = tracker.get_executed_tests();
        assert_eq!(executed_tests.len(), 3);
        assert!(executed_tests.contains(&"test_1".to_string()));
        assert!(executed_tests.contains(&"test_2".to_string()));
        assert!(executed_tests.contains(&"test_3".to_string()));
    }

    #[test]
    fn test_reset() {
        let mut tracker = TestCoverageTracker::new(100);
        
        // Mark some tests as executed
        tracker.mark_test_executed("test_1");
        tracker.mark_test_executed("test_2");
        
        // Verify tests are marked
        assert!(tracker.is_test_executed("test_1"));
        assert_eq!(tracker.get_execution_count("test_1"), 1);
        
        // Reset the tracker
        tracker.reset();
        
        // Verify it's reset
        assert!(!tracker.is_test_executed("test_1"));
        assert_eq!(tracker.get_execution_count("test_1"), 0);
        assert_eq!(tracker.get_executed_tests().len(), 0);
    }

    #[test]
    fn test_bloom_filter_properties() {
        let mut tracker = TestCoverageTracker::new(10000);
        
        // Add a large number of tests
        for i in 0..5000 {
            tracker.mark_test_executed(&format!("test_security__component_{}__behavior__condition", i));
        }
        
        // Check that all added tests are found
        for i in 0..5000 {
            assert!(tracker.is_test_executed(&format!("test_security__component_{}__behavior__condition", i)));
        }
        
        // Most non-added tests should not be found (Bloom filter property)
        let mut not_found_count = 0;
        for i in 5000..6000 {
            if !tracker.is_test_executed(&format!("test_security__component_{}__behavior__condition", i)) {
                not_found_count += 1;
            }
        }
        
        // Should have a high percentage of non-added tests correctly identified as not executed
        assert!(not_found_count > 900); // 90%+ should be correctly identified as not executed
    }
}