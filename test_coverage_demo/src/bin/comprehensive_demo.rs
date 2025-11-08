// Comprehensive demonstration of TestCoverageTracker with security_tests_full.csv
use dex_core::test_coverage::{TestCoverageTracker, TestCoverageStats};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Comprehensive TestCoverageTracker Demonstration");
    println!("=============================================");
    println!("Processing security_tests_full.csv with Bloom Filter-based coverage tracking\n");
    
    // Open and process the CSV file
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
    
    println!("📊 CSV Processing Results:");
    println!("   • Total security tests found: {}", test_names.len());
    
    // Create a coverage tracker for all tests
    let mut coverage_tracker = TestCoverageTracker::new(test_names.len());
    
    // Performance test: Add all tests to the tracker
    println!("\n⚡ Performance Test - Adding tests to coverage tracker:");
    let start_time = Instant::now();
    
    // Mark first 2000 tests as executed (simulate running a large subset)
    let executed_count = 2000.min(test_names.len());
    for i in 0..executed_count {
        coverage_tracker.mark_test_executed(&test_names[i]);
    }
    
    let elapsed = start_time.elapsed();
    println!("   • Time to add {} tests: {:?}", executed_count, elapsed);
    println!("   • Average time per test: {:?} (microseconds)", elapsed.as_micros() / executed_count as u128);
    
    // Get and display coverage statistics
    let stats = coverage_tracker.get_coverage_stats();
    println!("\n📈 Coverage Statistics:");
    println!("   • Total tests in suite: {}", stats.total_tests);
    println!("   • Executed tests: {}", stats.executed_tests);
    println!("   • Coverage percentage: {:.2}%", stats.coverage_percentage);
    
    // Demonstrate specific test queries
    println!("\n🔍 Specific Test Queries:");
    println!("   • First test executed: {}", coverage_tracker.is_test_executed(&test_names[0]));
    println!("   • Test #1000 executed: {}", coverage_tracker.is_test_executed(&test_names[999]));
    
    // Show execution counts
    println!("\n🔢 Execution Count Tracking:");
    println!("   • First test execution count: {}", coverage_tracker.get_execution_count(&test_names[0]));
    println!("   • Test #500 execution count: {}", coverage_tracker.get_execution_count(&test_names[499]));
    println!("   • Non-executed test count: {}", coverage_tracker.get_execution_count("completely_new_test"));
    
    // Demonstrate list of executed tests
    let executed_tests = coverage_tracker.get_executed_tests();
    println!("\n📋 Executed Tests Summary:");
    println!("   • Number of tracked executed tests: {}", executed_tests.len());
    
    // Show a sample of executed tests
    println!("   • Sample executed tests:");
    for i in 0..5.min(executed_tests.len()) {
        println!("     - {}", executed_tests[i]);
    }
    
    // Performance test: Query performance
    println!("\n⚡ Query Performance Test:");
    let start_time = Instant::now();
    
    // Perform 1000 queries
    for i in 0..1000 {
        let _ = coverage_tracker.is_test_executed(&test_names[i % test_names.len()]);
    }
    
    let elapsed = start_time.elapsed();
    println!("   • Time for 1000 queries: {:?}", elapsed);
    println!("   • Average query time: {:?} (microseconds)", elapsed.as_micros() / 1000);
    
    // Demonstrate false positive rate testing
    println!("\n🔬 Bloom Filter Characteristics:");
    let mut false_positives = 0;
    let mut true_negatives = 0;
    
    // Test 500 non-executed tests to estimate false positive rate
    for i in 2000..2500 {
        if i < test_names.len() {
            if coverage_tracker.is_test_executed(&test_names[i]) {
                false_positives += 1;
            } else {
                true_negatives += 1;
            }
        }
    }
    
    let false_positive_rate = (false_positives as f64 / 500.0) * 100.0;
    println!("   • False positives in 500 non-executed tests: {}", false_positives);
    println!("   • True negatives: {}", true_negatives);
    println!("   • Estimated false positive rate: {:.2}%", false_positive_rate);
    
    // Demonstrate reset functionality
    println!("\n🔄 Reset Functionality:");
    let executed_before_reset = coverage_tracker.get_coverage_stats().executed_tests;
    println!("   • Executed tests before reset: {}", executed_before_reset);
    
    coverage_tracker.reset();
    
    let executed_after_reset = coverage_tracker.get_coverage_stats().executed_tests;
    println!("   • Executed tests after reset: {}", executed_after_reset);
    println!("   • Reset successful: {}", executed_after_reset == 0);
    
    // Final summary
    println!("\n✅ Comprehensive Demonstration Complete!");
    println!("   • Successfully processed {} security tests from CSV", test_names.len());
    println!("   • Demonstrated high-performance coverage tracking");
    println!("   • Showed detailed execution count tracking");
    println!("   • Verified Bloom filter characteristics");
    println!("   • Tested reset functionality");
    
    println!("\n🎯 Key Benefits of Bloom Filter-based Test Coverage:");
    println!("   • Memory efficient - no storage of actual test names in filter");
    println!("   • Fast O(k) operations where k is hash functions count");
    println!("   • Scalable to millions of tests");
    println!("   • Provides statistical coverage metrics");
    println!("   • Integrates with existing test infrastructure");
    
    Ok(())
}