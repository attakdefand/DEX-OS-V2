//! Bulkhead pattern implementation for resource isolation
//!
//! Implements the Priority 3 feature from DEX-OS-V2.csv:
//! - Distributed Systems,Distributed Systems,Distributed Systems,Bulkhead,Resource Isolation,Medium

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use thiserror::Error;

/// Errors that can occur when using the bulkhead pattern
#[derive(Error, Debug, PartialEq)]
pub enum BulkheadError {
    /// The bulkhead has reached its resource limit
    #[error("Bulkhead resource limit reached")]
    ResourceLimitReached,
    
    /// The operation timed out while waiting for resources
    #[error("Operation timed out waiting for bulkhead resources")]
    Timeout,
    
    /// The bulkhead is in a failed state
    #[error("Bulkhead is in a failed state")]
    Failed,
}

/// Configuration for a bulkhead
#[derive(Debug, Clone)]
pub struct BulkheadConfig {
    /// Maximum number of concurrent operations allowed
    pub max_concurrent: usize,
    
    /// Maximum time to wait for a resource slot
    pub timeout_ms: u64,
    
    /// Maximum queue size for waiting operations
    pub max_queue_size: usize,
}

impl Default for BulkheadConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 10,
            timeout_ms: 5000,
            max_queue_size: 100,
        }
    }
}

/// Internal state of the bulkhead
#[derive(Debug)]
struct BulkheadState {
    /// Current number of active operations
    active_count: usize,
    
    /// Current number of queued operations
    queue_count: usize,
    
    /// Whether the bulkhead is in a failed state
    is_failed: bool,
}

/// Bulkhead for resource isolation
#[derive(Debug, Clone)]
pub struct Bulkhead {
    /// Configuration for the bulkhead
    config: BulkheadConfig,
    
    /// Internal state protected by a mutex
    state: Arc<Mutex<BulkheadState>>,
}

impl Bulkhead {
    /// Create a new bulkhead with the given configuration
    pub fn new(config: BulkheadConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(BulkheadState {
                active_count: 0,
                queue_count: 0,
                is_failed: false,
            })),
        }
    }
    
    /// Create a new bulkhead with default configuration
    pub fn default() -> Self {
        Self::new(BulkheadConfig::default())
    }
    
    /// Attempt to acquire a resource slot from the bulkhead
    /// 
    /// Returns Ok(Permit) if a slot is acquired, or an error if the bulkhead is at capacity
    /// or in a failed state.
    pub fn try_acquire(&self) -> Result<Permit, BulkheadError> {
        let mut state = self.state.lock().unwrap();
        
        // Check if bulkhead is in failed state
        if state.is_failed {
            return Err(BulkheadError::Failed);
        }
        
        // Check if we can acquire immediately
        if state.active_count < self.config.max_concurrent {
            state.active_count += 1;
            Ok(Permit {
                bulkhead_state: self.state.clone(),
            })
        } else {
            Err(BulkheadError::ResourceLimitReached)
        }
    }
    
    /// Acquire a resource slot from the bulkhead, waiting if necessary
    /// 
    /// Returns Ok(Permit) if a slot is acquired within the timeout period,
    /// or an error if the timeout is exceeded or the bulkhead is in a failed state.
    pub fn acquire(&self) -> Result<Permit, BulkheadError> {
        let start_time = Instant::now();
        let timeout = Duration::from_millis(self.config.timeout_ms);
        
        loop {
            // Try to acquire immediately
            match self.try_acquire() {
                Ok(permit) => return Ok(permit),
                Err(BulkheadError::ResourceLimitReached) => {
                    // Check if we've timed out
                    if start_time.elapsed() >= timeout {
                        return Err(BulkheadError::Timeout);
                    }
                    
                    // Briefly sleep before trying again
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(e) => return Err(e),
            }
        }
    }
    
    /// Get the current status of the bulkhead
    pub fn status(&self) -> BulkheadStatus {
        let state = self.state.lock().unwrap();
        BulkheadStatus {
            active_count: state.active_count,
            queue_count: state.queue_count,
            max_concurrent: self.config.max_concurrent,
            is_failed: state.is_failed,
        }
    }
    
    /// Mark the bulkhead as failed
    pub fn mark_failed(&self) {
        let mut state = self.state.lock().unwrap();
        state.is_failed = true;
    }
    
    /// Reset the bulkhead to a healthy state
    pub fn reset(&self) {
        let mut state = self.state.lock().unwrap();
        state.is_failed = false;
        state.active_count = 0;
        state.queue_count = 0;
    }
}

/// Status information for a bulkhead
#[derive(Debug, Clone)]
pub struct BulkheadStatus {
    /// Current number of active operations
    pub active_count: usize,
    
    /// Current number of queued operations
    pub queue_count: usize,
    
    /// Maximum number of concurrent operations allowed
    pub max_concurrent: usize,
    
    /// Whether the bulkhead is in a failed state
    pub is_failed: bool,
}

/// A permit representing a held resource slot in a bulkhead
/// 
/// When this permit is dropped, the resource slot is automatically released.
#[derive(Debug)]
pub struct Permit {
    /// Reference to the bulkhead state to update when the permit is dropped
    bulkhead_state: Arc<Mutex<BulkheadState>>,
}

impl Drop for Permit {
    fn drop(&mut self) {
        if let Ok(mut state) = self.bulkhead_state.lock() {
            if state.active_count > 0 {
                state.active_count -= 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::thread;
    
    #[test]
    fn test_bulkhead_creation() {
        let bulkhead = Bulkhead::default();
        let status = bulkhead.status();
        
        assert_eq!(status.active_count, 0);
        assert_eq!(status.queue_count, 0);
        assert_eq!(status.max_concurrent, 10);
        assert_eq!(status.is_failed, false);
    }
    
    #[test]
    fn test_try_acquire_success() {
        let bulkhead = Bulkhead::default();
        let permit = bulkhead.try_acquire();
        
        assert!(permit.is_ok());
        
        let status = bulkhead.status();
        assert_eq!(status.active_count, 1);
    }
    
    #[test]
    fn test_try_acquire_limit_reached() {
        let config = BulkheadConfig {
            max_concurrent: 2,
            ..Default::default()
        };
        let bulkhead = Bulkhead::new(config);
        
        // Acquire all available slots
        let _permit1 = bulkhead.try_acquire().unwrap();
        let _permit2 = bulkhead.try_acquire().unwrap();
        
        // Try to acquire one more - should fail
        let permit3 = bulkhead.try_acquire();
        assert!(matches!(
            permit3,
            Err(BulkheadError::ResourceLimitReached)
        ));
        
        let status = bulkhead.status();
        assert_eq!(status.active_count, 2);
    }
    
    #[test]
    fn test_permit_drop_releases_resource() {
        let bulkhead = Bulkhead::default();
        
        {
            let _permit = bulkhead.try_acquire().unwrap();
            let status = bulkhead.status();
            assert_eq!(status.active_count, 1);
        } // Permit is dropped here
        
        let status = bulkhead.status();
        assert_eq!(status.active_count, 0);
    }
    
    #[test]
    fn test_bulkhead_failure() {
        let bulkhead = Bulkhead::default();
        bulkhead.mark_failed();
        
        let permit = bulkhead.try_acquire();
        assert!(matches!(permit, Err(BulkheadError::Failed)));
        
        let status = bulkhead.status();
        assert_eq!(status.is_failed, true);
        
        // Reset and verify it works again
        bulkhead.reset();
        let status = bulkhead.status();
        assert_eq!(status.is_failed, false);
        assert_eq!(status.active_count, 0);
        
        let permit = bulkhead.try_acquire();
        assert!(permit.is_ok());
    }
    
    #[test]
    fn test_concurrent_access() {
        let config = BulkheadConfig {
            max_concurrent: 5,
            ..Default::default()
        };
        let bulkhead = Bulkhead::new(config);
        let bulkhead_clone = bulkhead.clone();
        
        // Spawn multiple threads to acquire permits
        let (tx, rx) = mpsc::channel();
        
        for _ in 0..10 {
            let tx_clone = tx.clone();
            let bulkhead_clone = bulkhead_clone.clone();
            thread::spawn(move || {
                match bulkhead_clone.try_acquire() {
                    Ok(_permit) => tx_clone.send(true).unwrap(),
                    Err(_) => tx_clone.send(false).unwrap(),
                }
            });
        }
        
        // Collect results
        let mut success_count = 0;
        let mut failure_count = 0;
        
        for _ in 0..10 {
            if rx.recv().unwrap() {
                success_count += 1;
            } else {
                failure_count += 1;
            }
        }
        
        // Exactly 5 should succeed (the max_concurrent limit)
        assert_eq!(success_count, 5);
        assert_eq!(failure_count, 5);
        
        // After all threads complete, active count should be 0
        // because all permits were dropped when threads ended
        let status = bulkhead.status();
        assert_eq!(status.active_count, 0);
    }
}
