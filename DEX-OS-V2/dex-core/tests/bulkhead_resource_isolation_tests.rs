//! Tests for the Bulkhead resource isolation implementation
//!
//! This module provides full validation of the Priority 3 testing feature from DEX-OS-V2.csv:
//! - Distributed Systems,Distributed Systems,Distributed Systems,Bulkhead,Resource Isolation,Medium

use dex_core::bulkhead::{Bulkhead, BulkheadConfig, BulkheadError};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Test basic bulkhead functionality
#[test]
fn test_bulkhead_basic_functionality() {
    let config = BulkheadConfig {
        max_concurrent: 3,
        timeout_ms: 1000,
        max_queue_size: 5,
    };
    let bulkhead = Bulkhead::new(config);

    // Acquire permits up to the limit
    let permit1 = bulkhead.acquire().expect("Should acquire permit 1");
    let permit2 = bulkhead.acquire().expect("Should acquire permit 2");
    let permit3 = bulkhead.acquire().expect("Should acquire permit 3");

    // Check status
    let status = bulkhead.status();
    assert_eq!(status.active_count, 3);
    assert_eq!(status.max_concurrent, 3);
    assert_eq!(status.is_failed, false);

    // Drop one permit
    drop(permit1);

    // Check status again
    let status = bulkhead.status();
    assert_eq!(status.active_count, 2);

    // Acquire another permit
    let _permit4 = bulkhead.acquire().expect("Should acquire permit 4");

    let status = bulkhead.status();
    assert_eq!(status.active_count, 3);
}

/// Test bulkhead resource limit enforcement
#[test]
fn test_bulkhead_resource_limit_enforcement() {
    let config = BulkheadConfig {
        max_concurrent: 2,
        timeout_ms: 100,
        max_queue_size: 1,
    };
    let bulkhead = Bulkhead::new(config);

    // Acquire all available permits
    let _permit1 = bulkhead.try_acquire().expect("Should acquire permit 1");
    let _permit2 = bulkhead.try_acquire().expect("Should acquire permit 2");

    // Try to acquire one more - should fail immediately
    let result = bulkhead.try_acquire();
    assert!(matches!(result, Err(BulkheadError::ResourceLimitReached)));
}

/// Test bulkhead failure handling
#[test]
fn test_bulkhead_failure_handling() {
    let bulkhead = Bulkhead::default();

    // Mark as failed
    bulkhead.mark_failed();

    // Try to acquire - should fail
    let result = bulkhead.try_acquire();
    assert!(matches!(result, Err(BulkheadError::Failed)));

    // Reset and try again
    bulkhead.reset();
    let result = bulkhead.try_acquire();
    assert!(result.is_ok());
}

/// Test concurrent access with bulkhead protection
#[test]
fn test_concurrent_access_with_bulkhead() {
    let config = BulkheadConfig {
        max_concurrent: 5,
        timeout_ms: 500,
        max_queue_size: 10,
    };
    let bulkhead = Arc::new(Bulkhead::new(config));
    let active_count = Arc::new(AtomicUsize::new(0));
    let max_active_count = Arc::new(AtomicUsize::new(0));

    let mut handles = vec![];

    // Spawn more threads than the bulkhead allows
    for _ in 0..15 {
        let bulkhead_clone = bulkhead.clone();
        let active_count_clone = active_count.clone();
        let max_active_count_clone = max_active_count.clone();

        let handle = thread::spawn(move || {
            // Try to acquire a permit
            if let Ok(_permit) = bulkhead_clone.acquire() {
                // Increment active count
                let current_active = active_count_clone.fetch_add(1, Ordering::SeqCst) + 1;

                // Update max active count if needed
                let current_max = max_active_count_clone.load(Ordering::SeqCst);
                if current_active > current_max {
                    max_active_count_clone.store(current_active, Ordering::SeqCst);
                }

                // Simulate some work
                thread::sleep(Duration::from_millis(100));

                // Decrement active count
                active_count_clone.fetch_sub(1, Ordering::SeqCst);

                true // Successfully acquired permit
            } else {
                false // Failed to acquire permit
            }
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    let results: Vec<bool> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // Count successes and failures
    let success_count = results.iter().filter(|&&r| r).count();
    let failure_count = results.iter().filter(|&&r| !r).count();

    // Check that we never exceeded the concurrent limit
    let max_active = max_active_count.load(Ordering::SeqCst);
    assert!(
        max_active <= 5,
        "Max active count {} exceeded limit of 5",
        max_active
    );

    // All threads should have either succeeded or failed
    assert_eq!(success_count + failure_count, 15);

    // At least some should have succeeded
    assert!(success_count > 0);

    // Final status should show 0 active operations
    let final_status = bulkhead.status();
    assert_eq!(final_status.active_count, 0);
}

/// Test bulkhead with timeout behavior
#[test]
fn test_bulkhead_timeout_behavior() {
    let config = BulkheadConfig {
        max_concurrent: 1,
        timeout_ms: 200, // 200ms timeout
        max_queue_size: 1,
    };
    let bulkhead = Bulkhead::new(config);

    // Acquire the only permit
    let _permit = bulkhead.acquire().expect("Should acquire permit");

    // Try to acquire another one - should timeout
    let start = std::time::Instant::now();
    let result = bulkhead.acquire();
    let duration = start.elapsed();

    assert!(matches!(result, Err(BulkheadError::Timeout)));
    assert!(duration >= Duration::from_millis(200));
    assert!(duration < Duration::from_millis(300)); // Allow some tolerance
}

/// Test bulkhead clone behavior
#[test]
fn test_bulkhead_clone_behavior() {
    let bulkhead1 = Bulkhead::default();
    let bulkhead2 = bulkhead1.clone();

    // Both should have the same initial status
    assert_eq!(bulkhead1.status().active_count, 0);
    assert_eq!(bulkhead2.status().active_count, 0);

    // Acquire a permit from the first instance
    let _permit = bulkhead1.acquire().expect("Should acquire permit");

    // Both should now show 1 active operation
    assert_eq!(bulkhead1.status().active_count, 1);
    assert_eq!(bulkhead2.status().active_count, 1);

    // Drop the permit
    drop(_permit);

    // Both should now show 0 active operations
    assert_eq!(bulkhead1.status().active_count, 0);
    assert_eq!(bulkhead2.status().active_count, 0);
}
