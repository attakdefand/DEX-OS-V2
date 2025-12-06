// Integration tests for TestCoverageTracker with TestResultsManager
use dex_core::test_coverage::TestCoverageTracker;
use dex_core::test_results::{
    IndividualTestResult, TestMetadata, TestResultsManager, TestStatus, TestSuiteResult,
};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn test_coverage_tracker_with_test_results_manager() {
    // Create both components
    let mut coverage_tracker = TestCoverageTracker::new(100);
    let mut results_manager = TestResultsManager::new();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_secs();

    // Create some test results and track coverage
    let mut test_results = Vec::new();
    for i in 0..10 {
        let test_name = format!("test_security_feature_{}", i);
        coverage_tracker.mark_test_executed(&test_name);

        test_results.push(IndividualTestResult {
            name: test_name,
            status: if i % 2 == 0 {
                TestStatus::Passed
            } else {
                TestStatus::Failed
            },
            duration_ms: 100 + i as u64 * 10,
            error_message: None,
            data: HashMap::new(),
        });
    }

    let test_suite_result = TestSuiteResult {
        id: format!("security_tests_{}", now),
        suite_name: "security_tests".to_string(),
        started_at: now - 120,
        finished_at: now,
        status: TestStatus::Failed, // mix of passed/failed above
        test_results,
        metadata: TestMetadata {
            version: "1.0.0".to_string(),
            commit_hash: "abc123".to_string(),
            environment: "ci".to_string(),
            platform: "windows".to_string(),
            custom: HashMap::new(),
        },
    };

    results_manager
        .store_result(test_suite_result.clone())
        .expect("failed to store suite");

    // Verify both systems work correctly
    let coverage_stats = coverage_tracker.get_coverage_stats();
    assert_eq!(coverage_stats.total_tests, 100);
    assert_eq!(coverage_stats.executed_tests, 10);
    assert_eq!(coverage_stats.coverage_percentage, 10.0);

    // Check that we can retrieve the stored results
    let retrieved_results = results_manager
        .get_result_by_name("security_tests")
        .expect("suite not found");
    assert_eq!(retrieved_results.test_results.len(), 10);
    assert_eq!(retrieved_results.status, TestStatus::Failed);

    // Verify specific test tracking
    assert!(coverage_tracker.is_test_executed("test_security_feature_0"));
    assert!(coverage_tracker.is_test_executed("test_security_feature_5"));
    assert_eq!(
        coverage_tracker.get_execution_count("test_security_feature_0"),
        1
    );

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
    assert_eq!(
        coverage_tracker.get_execution_count("large_dataset_test_1000"),
        1
    );

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
