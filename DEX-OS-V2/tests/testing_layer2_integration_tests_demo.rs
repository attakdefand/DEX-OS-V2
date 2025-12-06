//! Demo of Testing Layer 2: Integration Testing and System Validation
//!
//! This file demonstrates integration tests for core DEX-OS components following
//! Testing Layer 2 requirements from RULES.md:
//! @RULES.md ###Testing Layer 2: Integration Testing and System Validation

/// Mock struct to demonstrate integration testing concepts
struct MockServiceA {
    data: String,
}

impl MockServiceA {
    pub fn new() -> Self {
        Self {
            data: "Service A Data".to_string(),
        }
    }
    
    pub fn get_data(&self) -> &str {
        &self.data
    }
    
    pub fn process_data(&self, input: &str) -> String {
        format!("Processed by A: {}", input)
    }
}

/// Another mock struct to demonstrate integration
struct MockServiceB {
    enabled: bool,
}

impl MockServiceB {
    pub fn new() -> Self {
        Self {
            enabled: true,
        }
    }
    
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    pub fn transform_data(&self, input: String) -> String {
        if self.enabled {
            format!("Transformed by B: {}", input)
        } else {
            "Service B disabled".to_string()
        }
    }
}

/// System that integrates multiple services
struct IntegratedSystem {
    service_a: MockServiceA,
    service_b: MockServiceB,
}

impl IntegratedSystem {
    pub fn new() -> Self {
        Self {
            service_a: MockServiceA::new(),
            service_b: MockServiceB::new(),
        }
    }
    
    /// End-to-end workflow demonstrating integration
    pub fn process_workflow(&self, input: &str) -> String {
        // Step 1: Get data from Service A
        let data = self.service_a.get_data();
        
        // Step 2: Process with Service A
        let processed = self.service_a.process_data(input);
        
        // Step 3: Transform with Service B
        let transformed = self.service_b.transform_data(processed);
        
        // Step 4: Combine results
        format!("{} | {} | Final: {}", data, transformed, input)
    }
    
    /// Check if all services are available
    pub fn health_check(&self) -> bool {
        self.service_b.is_enabled()
    }
}

/// Integration test framework demonstration
pub struct IntegrationTestFramework {
    suite_name: String,
    tests_run: usize,
    tests_passed: usize,
    tests_failed: usize,
}

impl IntegrationTestFramework {
    /// Create a new integration test framework
    pub fn new(suite_name: &str) -> Self {
        Self {
            suite_name: suite_name.to_string(),
            tests_run: 0,
            tests_passed: 0,
            tests_failed: 0,
        }
    }
    
    /// Run an integration test
    pub fn run_integration_test<F>(&mut self, name: &str, test_fn: F) 
    where
        F: FnOnce() -> Result<(), String>,
    {
        self.tests_run += 1;
        print!("Running integration test '{}::{}' ... ", self.suite_name, name);
        
        match test_fn() {
            Ok(()) => {
                println!("PASSED");
                self.tests_passed += 1;
            }
            Err(msg) => {
                println!("FAILED: {}", msg);
                self.tests_failed += 1;
            }
        }
    }
    
    /// Assert that two values are equal
    pub fn assert_eq<T, U>(left: T, right: U) -> Result<(), String> 
    where
        T: PartialEq<U> + std::fmt::Debug,
        U: std::fmt::Debug,
    {
        if left == right {
            Ok(())
        } else {
            Err(format!("Expected {:?}, got {:?}", right, left))
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
    
    /// Get test suite statistics
    pub fn get_statistics(&self) -> (usize, usize, usize) {
        (self.tests_passed, self.tests_failed, self.tests_run)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mock_service_a() {
        let service = MockServiceA::new();
        assert_eq!(service.get_data(), "Service A Data");
        assert_eq!(service.process_data("test"), "Processed by A: test");
    }
    
    #[test]
    fn test_mock_service_b() {
        let service = MockServiceB::new();
        assert!(service.is_enabled());
        assert_eq!(service.transform_data("test".to_string()), "Transformed by B: test");
    }
    
    #[test]
    fn test_integrated_system() {
        let system = IntegratedSystem::new();
        assert!(system.health_check());
        let result = system.process_workflow("input");
        assert!(result.contains("Service A Data"));
        assert!(result.contains("Processed by A: input"));
        assert!(result.contains("Transformed by B:"));
    }
}

/// Demo function showing how Testing Layer 2 integration tests would work
#[cfg(test)]
mod demo {
    use super::*;
    
    #[test]
    fn demo_testing_layer2_integration_tests() -> Result<(), String> {
        println!("=== Testing Layer 2: Integration Testing and System Validation Demo ===");
        println!();
        
        // Create integration test framework
        let mut framework = IntegrationTestFramework::new("Core Services Integration");
        
        // Test individual service functionality
        framework.run_integration_test("service_a_functionality", || {
            let service_a = MockServiceA::new();
            IntegrationTestFramework::assert_eq(service_a.get_data(), "Service A Data")?;
            IntegrationTestFramework::assert_eq(
                service_a.process_data("test input"), 
                "Processed by A: test input"
            )
        });
        
        // Test service interaction
        framework.run_integration_test("service_interaction", || {
            let service_a = MockServiceA::new();
            let service_b = MockServiceB::new();
            
            let processed = service_a.process_data("integration test");
            let transformed = service_b.transform_data(processed);
            
            IntegrationTestFramework::assert_true(
                transformed.contains("Transformed by B:"), 
                "Service B should transform data from Service A"
            )
        });
        
        // Test complete system workflow
        framework.run_integration_test("complete_workflow", || {
            let system = IntegratedSystem::new();
            
            // Verify system health
            IntegrationTestFramework::assert_true(system.health_check(), "System should be healthy")?;
            
            // Test end-to-end workflow
            let result = system.process_workflow("user request");
            IntegrationTestFramework::assert_true(
                result.contains("Service A Data"), 
                "Result should contain Service A data"
            )?;
            IntegrationTestFramework::assert_true(
                result.contains("Processed by A:"), 
                "Result should show processing by Service A"
            )?;
            IntegrationTestFramework::assert_true(
                result.contains("Transformed by B:"), 
                "Result should show transformation by Service B"
            )?;
            IntegrationTestFramework::assert_true(
                result.contains("Final: user request"), 
                "Result should contain original input"
            )
        });
        
        // Test system resilience
        framework.run_integration_test("system_resilience", || {
            let system = IntegratedSystem::new();
            
            // Test multiple operations
            for i in 0..5 {
                let input = format!("request_{}", i);
                let result = system.process_workflow(&input);
                IntegrationTestFramework::assert_true(
                    result.contains(&input), 
                    "Each request should be processed correctly"
                )?;
            }
            
            Ok(())
        });
        
        // Print statistics
        let (passed, failed, total) = framework.get_statistics();
        println!();
        println!("Integration Test Results:");
        println!("  Passed: {}", passed);
        println!("  Failed: {}", failed);
        println!("  Total:  {}", total);
        if total > 0 {
            println!("  Success Rate: {:.1}%", (passed as f64 / total as f64) * 100.0);
        } else {
            println!("  Success Rate: N/A (no tests run)");
        }
        
        // Verify we had integration tests
        IntegrationTestFramework::assert_true(total > 0, "Should have run integration tests")?;
        IntegrationTestFramework::assert_true(passed > 0, "Should have passing integration tests")?;
        
        Ok(())
    }
}
