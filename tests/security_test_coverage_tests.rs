//! Tests for the Test Coverage implementation using Bloom Filters
//!
//! This module tests the Priority 3 testing feature from DEX-OS-V2.csv:
//! - Testing,Testing,Testing,Bloom Filter (conceptual),Test Coverage,Medium

use dex_core::test_coverage::{TestCoverageTracker, TestCoverageStats};
use std::fs::File;
use std::io::{BufRead, BufReader};

/// Test the TestCoverageTracker with actual security test data
#[test]
fn test_security_test_coverage_with_csv_data() {
    // Create a coverage tracker for all security tests
    // Based on our count, there are 3169 lines in the CSV (including header)
    let mut tracker = TestCoverageTracker::new(3168); // -1 for header row
    
    // Read the CSV file and process the test names
    let file = File::open("d:/DEX-OS-V2/DEX-OS-V2/.reference/security_tests_full.csv")
        .expect("Failed to open security_tests_full.csv");
    let reader = BufReader::new(file);
    
    let mut test_names = Vec::new();
    
    // Skip the header line
    for (index, line) in reader.lines().enumerate().skip(1) {
        let line = line.expect("Failed to read line");
        let fields: Vec<&str> = line.split(',').collect();
        
        // Ensure we have the expected number of fields
        if fields.len() >= 5 {
            let test_name = fields[4].to_string();
            test_names.push(test_name);
        }
        
        // For testing purposes, let's only process a subset to keep the test reasonable
        if index >= 100 {  // Process first 100 tests
            break;
        }
    }
    
    // Mark some tests as executed (simulating test execution)
    for i in 0..50 {  // Mark first 50 as executed
        if i < test_names.len() {
            tracker.mark_test_executed(&test_names[i]);
        }
    }
    
    // Verify coverage statistics
    let stats = tracker.get_coverage_stats();
    assert_eq!(stats.total_tests, 3168); // Total from CSV
    assert_eq!(stats.executed_tests, 50); // We marked 50 as executed
    
    // Check that executed tests are correctly identified
    for i in 0..50 {
        if i < test_names.len() {
            assert!(tracker.is_test_executed(&test_names[i]));
            assert_eq!(tracker.get_execution_count(&test_names[i]), 1);
        }
    }
    
    // Check that non-executed tests are correctly identified (with high probability)
    let mut not_found_count = 0;
    for i in 50..100 {
        if i < test_names.len() && !tracker.is_test_executed(&test_names[i]) {
            not_found_count += 1;
        }
    }
    
    // Most non-executed tests should be correctly identified as not executed
    assert!(not_found_count >= 40); // At least 80% should be correctly identified
    
    // Verify we can get the list of executed tests
    let executed_tests = tracker.get_executed_tests();
    assert_eq!(executed_tests.len(), 50);
    
    println!("Test coverage statistics:");
    println!("  Total tests: {}", stats.total_tests);
    println!("  Executed tests: {}", stats.executed_tests);
    println!("  Coverage percentage: {:.2}%", stats.coverage_percentage);
}

/// Test the TestCoverageTracker with repeated test executions
#[test]
fn test_test_coverage_with_repeated_executions() {
    let mut tracker = TestCoverageTracker::new(100);
    
    // Mark the same test as executed multiple times
    let test_name = "test_security__governance_and_policy__policy__enforces__on_request";
    tracker.mark_test_executed(test_name);
    tracker.mark_test_executed(test_name);
    tracker.mark_test_executed(test_name);
    
    // Verify the test is marked as executed
    assert!(tracker.is_test_executed(test_name));
    
    // Verify the execution count
    assert_eq!(tracker.get_execution_count(test_name), 3);
    
    // Verify coverage statistics
    let stats = tracker.get_coverage_stats();
    assert_eq!(stats.executed_tests, 1); // Only one unique test executed
    assert_eq!(stats.coverage_percentage, 1.0); // 1/100 * 100 = 1%
}

/// Test the TestCoverageTracker with a large number of tests
#[test]
fn test_test_coverage_with_large_dataset() {
    // Create a coverage tracker for a large number of tests
    let mut tracker = TestCoverageTracker::new(10000);
    
    // Add a large number of test names
    for i in 0..7500 {
        tracker.mark_test_executed(&format!("test_security_component_{}_behavior_condition", i));
    }
    
    // Verify coverage statistics
    let stats = tracker.get_coverage_stats();
    assert_eq!(stats.total_tests, 10000);
    assert_eq!(stats.executed_tests, 7500);
    assert_eq!(stats.coverage_percentage, 75.0); // 7500/10000 * 100 = 75%
    
    // Verify that executed tests are found
    assert!(tracker.is_test_executed("test_security_component_1000_behavior_condition"));
    assert!(tracker.is_test_executed("test_security_component_5000_behavior_condition"));
    
    // Most non-executed tests should be correctly identified as not executed
    let mut not_found_count = 0;
    for i in 7500..8500 {
        if !tracker.is_test_executed(&format!("test_security_component_{}_behavior_condition", i)) {
            not_found_count += 1;
        }
    }
    
    // Should have a high percentage of non-executed tests correctly identified
    assert!(not_found_count > 900); // 90%+ should be correctly identified
}

/// Integration test showing how the TestCoverageTracker would be used in practice
#[test]
fn test_integration_with_test_results_manager() {
    use dex_core::test_results::{TestResultsManager, TestSuiteResult, IndividualTestResult, TestStatus, TestMetadata};
    use std::time::{SystemTime, UNIX_EPOCH};
    
    // Create a coverage tracker
    let mut coverage_tracker = TestCoverageTracker::new(3168); // Total security tests
    
    // Create a test results manager
    let mut results_manager = TestResultsManager::new();
    
    // Simulate running a test suite
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Create individual test results that match security test names
    let mut test_results = Vec::new();
    
    // Simulate running some security tests
    let executed_tests = vec![
        "test_security__governance_and_policy__policy__enforces__on_request",
        "test_security__governance_and_policy__policy__validates__during_ci",
        "test_security__governance_and_policy__scanner__enforces__on_request",
        "test_security__governance_and_policy__gateway__validates__after_deploy",
        "test_security__governance_and_policy__vault__blocks__quarterly",
    ];
    
    for test_name in executed_tests {
        test_results.push(IndividualTestResult {
            name: test_name.to_string(),
            status: TestStatus::Passed,
            duration_ms: 45,
            error_message: None,
            data: std::collections::HashMap::new(),
        });
        
        // Mark the test as executed in our coverage tracker
        coverage_tracker.mark_test_executed(test_name);
    }
    
    // Create a test suite result
    let suite_result = TestSuiteResult {
        id: format!("security_test_suite_{}", now),
        suite_name: "Security Test Coverage Suite".to_string(),
        started_at: now - 100,
        finished_at: now,
        status: TestStatus::Passed,
        test_results,
        metadata: TestMetadata {
            version: "1.0.0".to_string(),
            commit_hash: "abc123def456".to_string(),
            environment: "testing".to_string(),
            platform: "windows".to_string(),
            custom: std::collections::HashMap::new(),
        },
    };
    
    // Store the results
    assert!(results_manager.store_result(suite_result).is_ok());
    
    // Verify coverage statistics
    let coverage_stats = coverage_tracker.get_coverage_stats();
    assert_eq!(coverage_stats.executed_tests, 5);
    
    // Verify we can get the executed tests
    let executed = coverage_tracker.get_executed_tests();
    assert_eq!(executed.len(), 5);
    
    // Verify test results were stored correctly
    let all_results = results_manager.get_all_results();
    assert_eq!(all_results.len(), 1);
    
    let stored_result = all_results[0];
    assert_eq!(stored_result.test_results.len(), 5);
    
    println!("Integration test completed successfully:");
    println!("  - Stored {} test results", stored_result.test_results.len());
    println!("  - Tracked {} executed tests for coverage", coverage_stats.executed_tests);
    println!("  - Coverage percentage: {:.2}%", coverage_stats.coverage_percentage);
}