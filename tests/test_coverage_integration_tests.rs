// Integration tests for TestCoverageTracker with TestResultsManager
use dex_core::test_coverage::TestCoverageTracker;
use dex_core::test_results::{TestResultsManager, TestSuiteResult};
use dex_core::types::TestResult;

#[test]
fn test_coverage_tracker_with_test_results_manager() {
    // Create both components
    let mut coverage_tracker = TestCoverageTracker::new(100);
    let mut results_manager = TestResultsManager::new();
    
    // Create some test results
    let mut test_results = Vec::new();
    for i in 0..10 {
        test_results.push(TestResult {
            test_name: format!("test_security_feature_{}", i),
            passed: i % 2 == 0, // Alternate pass/fail
            execution_time_ms: 100 + i * 10,
            timestamp: 1000000 + i as u64,
        });
    }
    
    let test_suite_result = TestSuiteResult {
        suite_name: "security_tests".to_string(),
        results: test_results,
        timestamp: 1000000,
        duration_ms: 1000,
    };
    
    // Track coverage while storing results
    for result in &test_suite_result.results {
        coverage_tracker.mark_test_executed(&result.test_name);
    }
    
    results_manager.store_result(test_suite_result.clone());
    
    // Verify both systems work correctly
    let coverage_stats = coverage_tracker.get_coverage_stats();
    assert_eq!(coverage_stats.total_tests, 100);
    assert_eq!(coverage_stats.executed_tests, 10);
    assert_eq!(coverage_stats.coverage_percentage, 10.0);
    
    // Check that we can retrieve the stored results
    let retrieved_results = results_manager.get_results("security_tests");
    assert!(retrieved_results.is_some());
    assert_eq!(retrieved_results.unwrap().results.len(), 10);
    
    // Verify specific test tracking
    assert!(coverage_tracker.is_test_executed("test_security_feature_0"));
    assert!(coverage_tracker.is_test_executed("test_security_feature_5"));
    assert_eq!(coverage_tracker.get_execution_count("test_security_feature_0"), 1);
    
    // Test that a non-executed test is correctly identified
    assert_eq!(coverage_tracker.get_execution_count("non_executed_test"), 0);
}

#[test]
fn test_coverage_tracker_with_large_dataset() {
    // Test with a larger dataset to ensure performance
    let mut coverage_tracker = TestCoverageTracker::new(10000);
    
    // Add 5000 test executions
    for i in 0..5000 {
        coverage_tracker.mark_test_executed(&format!("large_dataset_test_{}", i));
    }
    
    let stats = coverage_tracker.get_coverage_stats();
    assert_eq!(stats.total_tests, 10000);
    assert_eq!(stats.executed_tests, 5000);
    assert_eq!(stats.coverage_percentage, 50.0);
    
    // Verify some specific tests
    assert!(coverage_tracker.is_test_executed("large_dataset_test_1000"));
    assert!(coverage_tracker.is_test_executed("large_dataset_test_4999"));
    assert_eq!(coverage_tracker.get_execution_count("large_dataset_test_1000"), 1);
    
    // Get executed tests list
    let executed_tests = coverage_tracker.get_executed_tests();
    assert_eq!(executed_tests.len(), 5000);
}

#[test]
fn test_coverage_tracker_reset_functionality() {
    let mut coverage_tracker = TestCoverageTracker::new(100);
    
    // Add some test executions
    coverage_tracker.mark_test_executed("test_1");
    coverage_tracker.mark_test_executed("test_2");
    coverage_tracker.mark_test_executed("test_3");
    
    // Verify tests are tracked
    assert!(coverage_tracker.is_test_executed("test_1"));
    assert_eq!(coverage_tracker.get_execution_count("test_1"), 1);
    
    let stats = coverage_tracker.get_coverage_stats();
    assert_eq!(stats.executed_tests, 3);
    
    // Reset the tracker
    coverage_tracker.reset();
    
    // Verify reset worked
    assert!(!coverage_tracker.is_test_executed("test_1"));
    assert_eq!(coverage_tracker.get_execution_count("test_1"), 0);
    
    let reset_stats = coverage_tracker.get_coverage_stats();
    assert_eq!(reset_stats.executed_tests, 0);
    assert_eq!(reset_stats.coverage_percentage, 0.0);
}