//! Comprehensive tests for the Bloom Filter-based security test coverage implementation
//!
//! This module provides full validation of the Priority 3 testing feature from DEX-OS-V2.csv:
//! - Testing,Testing,Testing,Bloom Filter (conceptual),Test Coverage,Medium

use dex_core::test_coverage::{TestCoverageTracker, TestCoverageStats};
use dex_core::test_results::{TestResultsManager, TestSuiteResult, IndividualTestResult, TestStatus, TestMetadata};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::{SystemTime, UNIX_EPOCH};

/// Test the complete integration with the actual security_tests_full.csv file
#[test]
fn test_complete_security_test_coverage_integration() {
    // Create a coverage tracker for all security tests
    let mut coverage_tracker = TestCoverageTracker::new(3168); // Total tests from CSV
    
    // Read and process the actual CSV file
    let file = File::open("d:/DEX-OS-V2/DEX-OS-V2/.reference/security_tests_full.csv")
        .expect("Failed to open security_tests_full.csv");
    let reader = BufReader::new(file);
    
    let mut test_names = Vec::new();
    
    // Process each line in the CSV (skip header)
    for (index, line_result) in reader.lines().enumerate().skip(1) {
        let line = line_result.expect("Failed to read line");
        let fields: Vec<&str> = line.split(',').collect();
        
        // Ensure we have the expected number of fields
        if fields.len() >= 5 {
            let test_name = fields[4].to_string();
            test_names.push(test_name);
        }
        
        // For testing performance, let's process a reasonable subset
        if index >= 1000 {  // Process first 1000 tests
            break;
        }
    }
    
    // Verify we have test names
    assert!(!test_names.is_empty());
    assert!(test_names.len() <= 1000);
    
    // Mark some tests as executed
    for i in 0..500 {  // Mark first 500 as executed
        if i < test_names.len() {
            coverage_tracker.mark_test_executed(&test_names[i]);
        }
    }
    
    // Verify coverage statistics
    let stats = coverage_tracker.get_coverage_stats();
    assert_eq!(stats.total_tests, 3168);
    assert_eq!(stats.executed_tests, 500);
    
    // Verify that executed tests are correctly identified
    for i in 0..500 {
        if i < test_names.len() {
            assert!(coverage_tracker.is_test_executed(&test_names[i]));
        }
    }
    
    // Verify execution counts
    for i in 0..10 {  // Check first 10
        if i < test_names.len() {
            assert_eq!(coverage_tracker.get_execution_count(&test_names[i]), 1);
        }
    }
    
    // Verify we can get the list of executed tests
    let executed_tests = coverage_tracker.get_executed_tests();
    assert_eq!(executed_tests.len(), 500);
    
    println!("Integration test results:");
    println!("  Total tests in CSV: {}", stats.total_tests);
    println!("  Processed tests: {}", test_names.len());
    println!("  Executed tests: {}", stats.executed_tests);
    println!("  Coverage percentage: {:.2}%", stats.coverage_percentage);
}

/// Test the Bloom Filter properties with a large dataset
#[test]
fn test_bloom_filter_properties_with_large_dataset() {
    let mut coverage_tracker = TestCoverageTracker::new(10000);
    
    // Add a large number of test names
    for i in 0..7500 {
        coverage_tracker.mark_test_executed(&format!("test_security_component_{}_behavior_condition", i));
    }
    
    // Verify coverage statistics
    let stats = coverage_tracker.get_coverage_stats();
    assert_eq!(stats.total_tests, 10000);
    assert_eq!(stats.executed_tests, 7500);
    assert_eq!(stats.coverage_percentage, 75.0);
    
    // Verify that executed tests are found
    assert!(coverage_tracker.is_test_executed("test_security_component_1000_behavior_condition"));
    assert!(coverage_tracker.is_test_executed("test_security_component_5000_behavior_condition"));
    
    // Test false positive rate - most non-executed tests should be correctly identified
    let mut false_positives = 0;
    let mut true_negatives = 0;
    
    for i in 7500..8500 {
        let test_name = format!("test_security_component_{}_behavior_condition", i);
        if coverage_tracker.is_test_executed(&test_name) {
            false_positives += 1;
        } else {
            true_negatives += 1;
        }
    }
    
    // With a well-sized Bloom filter, false positives should be relatively rare
    // We expect most non-executed tests to be correctly identified as not executed
    assert!(true_negatives > 900); // 90%+ should be correctly identified
    assert!(false_positives < 100); // Less than 10% should be false positives
    
    println!("Bloom Filter properties test:");
    println!("  False positives: {}", false_positives);
    println!("  True negatives: {}", true_negatives);
    println!("  False positive rate: {:.2}%", (false_positives as f64 / 1000.0) * 100.0);
}

/// Test integration with TestResultsManager
#[test]
fn test_integration_with_test_results_manager() {
    // Create components
    let mut coverage_tracker = TestCoverageTracker::new(100);
    let mut results_manager = TestResultsManager::new();
    
    // Simulate running tests
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let mut test_results = Vec::new();
    
    // Create test results that match security test patterns
    let executed_tests = vec![
        "test_security__governance_and_policy__policy__enforces__on_request",
        "test_security__governance_and_policy__policy__validates__during_ci",
        "test_security__governance_and_policy__scanner__enforces__on_request",
        "test_security__governance_and_policy__gateway__validates__after_deploy",
        "test_security__governance_and_policy__vault__blocks__quarterly",
    ];
    
    for (i, test_name) in executed_tests.iter().enumerate() {
        let status = if i % 5 == 0 {
            TestStatus::Failed
        } else {
            TestStatus::Passed
        };
        
        let test_result = IndividualTestResult {
            name: test_name.to_string(),
            status,
            duration_ms: (i * 10 + 20) as u64,
            error_message: if status == TestStatus::Failed {
                Some("Test failed".to_string())
            } else {
                None
            },
            data: std::collections::HashMap::new(),
        };
        
        test_results.push(test_result);
        coverage_tracker.mark_test_executed(test_name);
    }
    
    // Create and store test suite result
    let suite_result = TestSuiteResult {
        id: format!("test_suite_{}", now),
        suite_name: "Security Coverage Test Suite".to_string(),
        started_at: now - 50,
        finished_at: now,
        status: TestStatus::Failed, // Because we have at least one failed test
        test_results,
        metadata: TestMetadata {
            version: "1.0.0".to_string(),
            commit_hash: "test123".to_string(),
            environment: "testing".to_string(),
            platform: "windows".to_string(),
            custom: std::collections::HashMap::new(),
        },
    };
    
    // Store results
    assert!(results_manager.store_result(suite_result).is_ok());
    
    // Verify coverage tracking
    let coverage_stats = coverage_tracker.get_coverage_stats();
    assert_eq!(coverage_stats.executed_tests, 5);
    assert_eq!(coverage_stats.coverage_percentage, 5.0); // 5/100 * 100
    
    // Verify test results storage
    let all_results = results_manager.get_all_results();
    assert_eq!(all_results.len(), 1);
    
    let stored_result = all_results[0];
    assert_eq!(stored_result.test_results.len(), 5);
    assert_eq!(stored_result.status, TestStatus::Failed);
    
    // Verify statistics
    let test_stats = results_manager.get_statistics();
    assert_eq!(test_stats.total_suites, 1);
    assert_eq!(test_stats.total_tests, 5);
    assert_eq!(test_stats.failed_tests, 1);
    assert_eq!(test_stats.passed_tests, 4);
    
    println!("Integration with TestResultsManager test passed:");
    println!("  Coverage: {} tests executed ({:.2}%)", coverage_stats.executed_tests, coverage_stats.coverage_percentage);
    println!("  Test results stored: {} suites, {} tests", test_stats.total_suites, test_stats.total_tests);
}

/// Test edge cases and error conditions
#[test]
fn test_edge_cases_and_error_conditions() {
    // Test with zero total tests
    let coverage_tracker = TestCoverageTracker::new(0);
    let stats = coverage_tracker.get_coverage_stats();
    assert_eq!(stats.total_tests, 0);
    assert_eq!(stats.coverage_percentage, 0.0);
    
    // Test with very large number of tests
    let large_tracker = TestCoverageTracker::new(1000000); // 1 million tests
    let large_stats = large_tracker.get_coverage_stats();
    assert_eq!(large_stats.total_tests, 1000000);
    assert_eq!(large_stats.coverage_percentage, 0.0); // No tests executed yet
    
    // Test marking the same test multiple times
    let mut coverage_tracker = TestCoverageTracker::new(100);
    let test_name = "edge_case_test";
    
    coverage_tracker.mark_test_executed(test_name);
    coverage_tracker.mark_test_executed(test_name);
    coverage_tracker.mark_test_executed(test_name);
    
    assert!(coverage_tracker.is_test_executed(test_name));
    assert_eq!(coverage_tracker.get_execution_count(test_name), 3);
    
    // Test getting execution count for non-executed test
    assert_eq!(coverage_tracker.get_execution_count("non_executed_test"), 0);
    
    // Test reset functionality
    coverage_tracker.reset();
    assert!(!coverage_tracker.is_test_executed(test_name));
    assert_eq!(coverage_tracker.get_execution_count(test_name), 0);
    
    let reset_stats = coverage_tracker.get_coverage_stats();
    assert_eq!(reset_stats.executed_tests, 0);
    
    println!("Edge cases and error conditions test passed");
}

/// Performance test for the Bloom Filter implementation
#[test]
fn test_bloom_filter_performance() {
    use std::time::Instant;
    
    let mut coverage_tracker = TestCoverageTracker::new(50000); // 50k tests
    
    // Measure time to add tests
    let start = Instant::now();
    for i in 0..30000 {
        coverage_tracker.mark_test_executed(&format!("performance_test_{}", i));
    }
    let add_duration = start.elapsed();
    
    // Measure time to check tests
    let start = Instant::now();
    for i in 0..1000 {
        coverage_tracker.is_test_executed(&format!("performance_test_{}", i));
    }
    let check_duration = start.elapsed();
    
    // Verify functionality still works after performance test
    assert!(coverage_tracker.is_test_executed("performance_test_1000"));
    
    let stats = coverage_tracker.get_coverage_stats();
    assert_eq!(stats.executed_tests, 30000);
    
    println!("Performance test results:");
    println!("  Time to add 30,000 tests: {:?}", add_duration);
    println!("  Time to check 1,000 tests: {:?}", check_duration);
    println!("  Memory usage: Bloom filter with 50000 tests capacity");
    
    // These should complete in reasonable time (less than 1 second each)
    assert!(add_duration.as_secs() < 1);
    assert!(check_duration.as_secs() < 1);
}