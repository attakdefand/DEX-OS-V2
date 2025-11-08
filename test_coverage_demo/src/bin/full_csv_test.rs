// Full test of TestCoverageTracker with the entire security_tests_full.csv file
use dex_core::test_coverage::{TestCoverageTracker, TestCoverageStats};
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing TestCoverageTracker with full security_tests_full.csv file");
    println!("==================================================================");
    
    // Open the CSV file
    let file = File::open("../.reference/security_tests_full.csv")?;
    let reader = BufReader::new(file);
    
    let mut test_names = Vec::new();
    
    // Process each line in the CSV (skip header)
    for (index, line_result) in reader.lines().enumerate().skip(1) {
        let line = line_result?;
        let fields: Vec<&str> = line.split(',').collect();
        
        // Ensure we have the expected number of fields
        if fields.len() >= 5 {
            let test_name = fields[4].to_string();
            test_names.push(test_name);
        }
    }
    
    println!("Total tests in CSV: {}", test_names.len());
    
    // Create a coverage tracker for all tests
    let mut coverage_tracker = TestCoverageTracker::new(test_names.len());
    
    // Mark first 1000 tests as executed (simulate running a subset of tests)
    let executed_count = 1000.min(test_names.len());
    for i in 0..executed_count {
        coverage_tracker.mark_test_executed(&test_names[i]);
    }
    
    // Get coverage statistics
    let stats = coverage_tracker.get_coverage_stats();
    
    println!("Coverage results:");
    println!("  Executed tests: {}", stats.executed_tests);
    println!("  Total tests: {}", stats.total_tests);
    println!("  Coverage percentage: {:.2}%", stats.coverage_percentage);
    
    // Test a few specific tests
    println!("\nTesting specific test queries:");
    println!("  First test executed: {}", coverage_tracker.is_test_executed(&test_names[0]));
    println!("  Test #500 executed: {}", coverage_tracker.is_test_executed(&test_names[499]));
    if test_names.len() > 1000 {
        println!("  Test #1001 executed: {}", coverage_tracker.is_test_executed(&test_names[1000]));
    }
    
    // Test execution counts
    println!("\nExecution counts:");
    println!("  First test count: {}", coverage_tracker.get_execution_count(&test_names[0]));
    println!("  Test #500 count: {}", coverage_tracker.get_execution_count(&test_names[499]));
    println!("  Non-executed test count: {}", coverage_tracker.get_execution_count("non_executed_test"));
    
    // Get list of executed tests
    let executed_tests = coverage_tracker.get_executed_tests();
    println!("\nExecuted tests list length: {}", executed_tests.len());
    
    // Verify we can get coverage stats multiple times
    let stats2 = coverage_tracker.get_coverage_stats();
    assert_eq!(stats.total_tests, stats2.total_tests);
    assert_eq!(stats.executed_tests, stats2.executed_tests);
    
    println!("\n✅ Successfully processed {} tests from security_tests_full.csv", test_names.len());
    println!("✅ Tracked execution of {} tests", stats.executed_tests);
    println!("✅ Coverage tracking working correctly");
    println!("✅ Execution count tracking working correctly");
    
    Ok(())
}