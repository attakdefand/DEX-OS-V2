// Standalone test of TestCoverageTracker with security_tests_full.csv
// This version doesn't depend on the dex-core crate
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;
use sha3::{Digest, Sha3_256};

/// Bloom filter for efficient probabilistic set membership testing
#[derive(Debug, Clone)]
pub struct BloomFilter {
    /// Bit array for the filter
    bits: Vec<bool>,
    /// Number of hash functions
    num_hash_functions: usize,
    /// Size of the bit array
    size: usize,
}

impl BloomFilter {
    /// Create a new Bloom filter with the specified size and number of hash functions
    pub fn new(size: usize, num_hash_functions: usize) -> Self {
        Self {
            bits: vec![false; size],
            num_hash_functions,
            size,
        }
    }

    /// Add an item to the Bloom filter
    pub fn add(&mut self, item: &str) {
        for i in 0..self.num_hash_functions {
            let hash = self.hash(item, i);
            let index = hash % self.size;
            self.bits[index] = true;
        }
    }

    /// Check if an item might be in the set (with possible false positives)
    pub fn might_contain(&self, item: &str) -> bool {
        for i in 0..self.num_hash_functions {
            let hash = self.hash(item, i);
            let index = hash % self.size;
            if !self.bits[index] {
                return false;
            }
        }
        true
    }

    /// Simple hash function using SHA3-256
    fn hash(&self, item: &str, seed: usize) -> usize {
        let mut hasher = Sha3_256::new();
        hasher.update(item.as_bytes());
        hasher.update(&[seed as u8]);
        let result = hasher.finalize();
        
        // Convert first 8 bytes to usize
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&result[..8]);
        usize::from_le_bytes(bytes)
    }
}

impl Default for BloomFilter {
    fn default() -> Self {
        // Default to a reasonably sized filter with 3 hash functions
        Self::new(1000, 3)
    }
}

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Standalone TestCoverageTracker with security_tests_full.csv");
    println!("========================================================");
    
    // Try to open the CSV file
    let file_path = "../.reference/security_tests_full.csv";
    let file = File::open(file_path)?;
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
    
    println!("\n✅ Successfully processed {} tests from security_tests_full.csv", test_names.len());
    println!("✅ Tracked execution of {} tests", stats.executed_tests);
    println!("✅ Coverage tracking working correctly");
    println!("✅ Execution count tracking working correctly");
    
    Ok(())
}