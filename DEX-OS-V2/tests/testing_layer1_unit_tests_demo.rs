//! Demo of Testing Layer 1: Unit Testing and Component Validation
//!
//! This file demonstrates unit tests for core DEX-OS components following
//! Testing Layer 1 requirements from RULES.md:
//! @RULES.md ###Testing Layer 1: Unit Testing and Component Validation

/// Mock struct to demonstrate unit testing concepts
struct MockComponent {
    value: i32,
}

impl MockComponent {
    /// Create a new mock component
    pub fn new(value: i32) -> Self {
        Self { value }
    }
    
    /// Get the value
    pub fn get_value(&self) -> i32 {
        self.value
    }
    
    /// Set the value
    pub fn set_value(&mut self, value: i32) {
        self.value = value;
    }
    
    /// Add to the value
    pub fn add(&self, other: i32) -> i32 {
        self.value + other
    }
    
    /// Divide the value, returning None if division by zero
    pub fn divide(&self, divisor: i32) -> Option<i32> {
        if divisor == 0 {
            None
        } else {
            Some(self.value / divisor)
        }
    }
}

/// Test framework for unit testing demonstration
pub struct UnitTestFramework {
    passed: usize,
    failed: usize,
    total: usize,
}

impl UnitTestFramework {
    /// Create a new unit test framework
    pub fn new() -> Self {
        Self {
            passed: 0,
            failed: 0,
            total: 0,
        }
    }
    
    /// Run a test
    pub fn run_test<F>(&mut self, name: &str, test_fn: F) 
    where
        F: FnOnce() -> Result<(), String>,
    {
        self.total += 1;
        print!("Running test: {} ... ", name);
        
        match test_fn() {
            Ok(()) => {
                println!("PASSED");
                self.passed += 1;
            }
            Err(msg) => {
                println!("FAILED: {}", msg);
                self.failed += 1;
            }
        }
    }
    
    /// Assert that two values are equal
    pub fn assert_eq<T: PartialEq + std::fmt::Debug>(left: T, right: T) -> Result<(), String> {
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
    
    /// Assert that a result is OK
    pub fn assert_ok<T, E>(result: Result<T, E>) -> Result<(), String> {
        match result {
            Ok(_) => Ok(()),
            Err(_) => Err("Expected Ok, got Err".to_string()),
        }
    }
    
    /// Assert that a result is Err
    pub fn assert_err<T, E>(result: Result<T, E>) -> Result<(), String> {
        match result {
            Ok(_) => Err("Expected Err, got Ok".to_string()),
            Err(_) => Ok(()),
        }
    }
    
    /// Get test statistics
    pub fn get_statistics(&self) -> (usize, usize, usize) {
        (self.passed, self.failed, self.total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mock_component_creation() {
        let component = MockComponent::new(42);
        assert_eq!(component.get_value(), 42);
    }
    
    #[test]
    fn test_mock_component_set_value() {
        let mut component = MockComponent::new(10);
        component.set_value(20);
        assert_eq!(component.get_value(), 20);
    }
    
    #[test]
    fn test_mock_component_add() {
        let component = MockComponent::new(5);
        assert_eq!(component.add(3), 8);
    }
    
    #[test]
    fn test_mock_component_divide() {
        let component = MockComponent::new(10);
        assert_eq!(component.divide(2), Some(5));
        assert_eq!(component.divide(0), None);
    }
    
    #[test]
    fn test_unit_test_framework() {
        let mut framework = UnitTestFramework::new();
        
        framework.run_test("successful_test", || {
            UnitTestFramework::assert_true(true, "Should pass")
        });
        
        framework.run_test("failing_test", || {
            UnitTestFramework::assert_true(false, "Should fail")
        });
        
        let (passed, failed, total) = framework.get_statistics();
        assert_eq!(passed, 1);
        assert_eq!(failed, 1);
        assert_eq!(total, 2);
    }
}

/// Demo function showing how Testing Layer 1 unit tests would work
#[cfg(test)]
mod demo {
    use super::*;
    
    #[test]
    fn demo_testing_layer1_unit_tests() -> Result<(), String> {
        println!("=== Testing Layer 1: Unit Testing and Component Validation Demo ===");
        println!();
        
        // Create test framework
        let mut framework = UnitTestFramework::new();
        
        // Test component creation
        framework.run_test("component_creation", || {
            let component = MockComponent::new(100);
            UnitTestFramework::assert_eq(component.get_value(), 100)
        });
        
        // Test component modification
        framework.run_test("component_modification", || {
            let mut component = MockComponent::new(50);
            component.set_value(75);
            UnitTestFramework::assert_eq(component.get_value(), 75)
        });
        
        // Test mathematical operations
        framework.run_test("mathematical_operations", || {
            let component = MockComponent::new(20);
            UnitTestFramework::assert_eq(component.add(5), 25)?;
            UnitTestFramework::assert_eq(component.divide(4), Some(5))?;
            UnitTestFramework::assert_eq(component.divide(0), None)
        });
        
        // Test error conditions
        framework.run_test("error_conditions", || {
            let component = MockComponent::new(10);
            let result = component.divide(0);
            UnitTestFramework::assert_eq(result, None)
        });
        
        // Print statistics
        let (passed, failed, total) = framework.get_statistics();
        println!();
        println!("Test Results:");
        println!("  Passed: {}", passed);
        println!("  Failed: {}", failed);
        println!("  Total:  {}", total);
        println!("  Success Rate: {:.1}%", (passed as f64 / total as f64) * 100.0);
        
        // Verify we had tests
        UnitTestFramework::assert_true(total > 0, "Should have run tests")?;
        UnitTestFramework::assert_true(passed > 0, "Should have passing tests")?;
        Ok(())
    }
}
