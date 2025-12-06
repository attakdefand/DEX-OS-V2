//! Tests for the Saga Pattern implementation for distributed transactions.
//!
//! These tests validate the Priority 3 feature:
//! "Distributed Systems,Distributed Systems,Distributed Systems,Saga Pattern,Distributed Transactions,Medium"

use dex_core::saga::{SagaError, SagaOrchestrator, SagaStatus};
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicUsize, Ordering};
use std::sync::Arc;

/// Test a successful saga execution with multiple steps.
#[tokio::test]
async fn test_successful_saga_execution() {
    let mut saga = SagaOrchestrator::<i32, String>::new();

    // Add steps that all succeed
    saga.add_step(
        "Create Order",
        || Ok(1001),
        || Ok(()), // Compensation does nothing
    );

    saga.add_step(
        "Reserve Funds",
        || Ok(2002),
        || Ok(()), // Compensation does nothing
    );

    saga.add_step(
        "Update Inventory",
        || Ok(3003),
        || Ok(()), // Compensation does nothing
    );

    let results = saga.execute().await.unwrap();

    // Verify results
    assert_eq!(results.len(), 3);
    assert_eq!(results[0], Some(1001));
    assert_eq!(results[1], Some(2002));
    assert_eq!(results[2], Some(3003));

    // Verify status
    assert_eq!(saga.status().await, SagaStatus::Success);
}

/// Test saga failure and compensation.
#[tokio::test]
async fn test_saga_failure_with_compensation() {
    static ORDER_CREATED: AtomicBool = AtomicBool::new(false);
    static FUNDS_RESERVED: AtomicBool = AtomicBool::new(false);
    static INVENTORY_UPDATED: AtomicBool = AtomicBool::new(false);
    static ORDER_CANCELLED: AtomicBool = AtomicBool::new(false);
    static FUNDS_RELEASED: AtomicBool = AtomicBool::new(false);

    let mut saga = SagaOrchestrator::<String, String>::new();

    // Step 1: Create order (succeeds)
    saga.add_step(
        "Create Order",
        || {
            ORDER_CREATED.store(true, Ordering::SeqCst);
            Ok("order-123".to_string())
        },
        || {
            ORDER_CANCELLED.store(true, Ordering::SeqCst);
            Ok(())
        },
    );

    // Step 2: Reserve funds (succeeds)
    saga.add_step(
        "Reserve Funds",
        || {
            FUNDS_RESERVED.store(true, Ordering::SeqCst);
            Ok("funds-reserved".to_string())
        },
        || {
            FUNDS_RELEASED.store(true, Ordering::SeqCst);
            Ok(())
        },
    );

    // Step 3: Update inventory (fails)
    saga.add_step(
        "Update Inventory",
        || {
            INVENTORY_UPDATED.store(true, Ordering::SeqCst);
            Err("Insufficient inventory".to_string())
        },
        || Ok(()), // Compensation does nothing
    );

    let result = saga.execute().await;

    // Verify the saga failed
    assert!(result.is_err());
    match result.unwrap_err() {
        SagaError::StepFailed {
            step_index,
            message,
        } => {
            assert_eq!(step_index, 2); // Third step (0-indexed)
            assert_eq!(message, "Insufficient inventory");
        }
        _ => panic!("Expected StepFailed error"),
    }

    // Verify status
    assert_eq!(saga.status().await, SagaStatus::Failed);

    // Verify execution flow
    assert!(ORDER_CREATED.load(Ordering::SeqCst));
    assert!(FUNDS_RESERVED.load(Ordering::SeqCst));
    assert!(INVENTORY_UPDATED.load(Ordering::SeqCst));

    // Verify compensation
    assert!(ORDER_CANCELLED.load(Ordering::SeqCst));
    assert!(FUNDS_RELEASED.load(Ordering::SeqCst));
}

/// Test empty saga execution.
#[tokio::test]
async fn test_empty_saga() {
    let saga = SagaOrchestrator::<(), String>::new();
    let results = saga.execute().await.unwrap();

    assert_eq!(results.len(), 0);
    assert_eq!(saga.status().await, SagaStatus::Success);
}

/// Test saga cannot be executed twice.
#[tokio::test]
async fn test_saga_cannot_execute_twice() {
    let mut saga = SagaOrchestrator::<i32, String>::new();

    saga.add_step("Single Step", || Ok(42), || Ok(()));

    // First execution should succeed
    assert!(saga.execute().await.is_ok());

    // Second execution should fail
    let result = saga.execute().await;
    assert!(matches!(result, Err(SagaError::AlreadyExecuted)));
}

/// Test saga cannot be executed concurrently.
#[tokio::test]
async fn test_concurrent_saga_execution() {
    let saga = Arc::new(SagaOrchestrator::<i32, String>::new());

    // Try to execute the saga concurrently
    let saga_clone1 = saga.clone();
    let saga_clone2 = saga.clone();

    let handle1 = tokio::spawn(async move { saga_clone1.execute().await });

    let handle2 = tokio::spawn(async move { saga_clone2.execute().await });

    let result1 = handle1.await.unwrap();
    let result2 = handle2.await.unwrap();

    // One should succeed, the other should fail with Executing error
    assert!(result1.is_ok() || result2.is_ok());
    assert!(
        matches!(result1, Err(SagaError::Executing))
            || matches!(result2, Err(SagaError::Executing))
    );
}

/// Test compensation failure handling.
#[tokio::test]
async fn test_compensation_failure() {
    static STEP_EXECUTED: AtomicBool = AtomicBool::new(false);
    static COMPENSATION_ATTEMPTED: AtomicBool = AtomicBool::new(false);

    let mut saga = SagaOrchestrator::<(), String>::new();

    saga.add_step(
        "Failing Step",
        || {
            STEP_EXECUTED.store(true, Ordering::SeqCst);
            Ok(())
        },
        || {
            COMPENSATION_ATTEMPTED.store(true, Ordering::SeqCst);
            Err("Compensation failed".to_string())
        },
    );

    saga.add_step(
        "Trigger Failure",
        || Err("Step failed".to_string()),
        || Ok(()),
    );

    let result = saga.execute().await;

    // Verify the saga failed
    assert!(result.is_err());
    match result.unwrap_err() {
        SagaError::CompensationFailed {
            step_index,
            message,
        } => {
            assert_eq!(step_index, 0); // First step
            assert_eq!(message, "Compensation failed");
        }
        _ => panic!("Expected CompensationFailed error"),
    }

    // Verify execution flow
    assert!(STEP_EXECUTED.load(Ordering::SeqCst));
    assert!(COMPENSATION_ATTEMPTED.load(Ordering::SeqCst));
}

/// Test complex saga with state management.
#[tokio::test]
async fn test_complex_saga_with_state() {
    static STATE: AtomicI32 = AtomicI32::new(0);
    static COMPENSATION_COUNT: AtomicUsize = AtomicUsize::new(0);

    let mut saga = SagaOrchestrator::<i32, String>::new();

    // Step 1: Add 10 to state
    saga.add_step(
        "Add 10",
        || {
            let current = STATE.load(Ordering::SeqCst);
            let new_value = current + 10;
            STATE.store(new_value, Ordering::SeqCst);
            Ok(new_value)
        },
        || {
            let current = STATE.load(Ordering::SeqCst);
            STATE.store(current - 10, Ordering::SeqCst);
            COMPENSATION_COUNT.fetch_add(1, Ordering::SeqCst);
            Ok(())
        },
    );

    // Step 2: Multiply by 2
    saga.add_step(
        "Multiply by 2",
        || {
            let current = STATE.load(Ordering::SeqCst);
            let new_value = current * 2;
            STATE.store(new_value, Ordering::SeqCst);
            Ok(new_value)
        },
        || {
            let current = STATE.load(Ordering::SeqCst);
            STATE.store(current / 2, Ordering::SeqCst);
            COMPENSATION_COUNT.fetch_add(1, Ordering::SeqCst);
            Ok(())
        },
    );

    // Step 3: Subtract 5 (this will fail)
    saga.add_step(
        "Subtract 5",
        || {
            let current = STATE.load(Ordering::SeqCst);
            if current < 5 {
                Err("Cannot subtract 5 from value less than 5".to_string())
            } else {
                let new_value = current - 5;
                STATE.store(new_value, Ordering::SeqCst);
                Ok(new_value)
            }
        },
        || {
            let current = STATE.load(Ordering::SeqCst);
            STATE.store(current + 5, Ordering::SeqCst);
            COMPENSATION_COUNT.fetch_add(1, Ordering::SeqCst);
            Ok(())
        },
    );

    // Initially state should be 0
    assert_eq!(STATE.load(Ordering::SeqCst), 0);

    let result = saga.execute().await;

    // Should fail because we can't subtract 5 from 0
    assert!(result.is_err());

    // After compensation, state should be back to 0
    assert_eq!(STATE.load(Ordering::SeqCst), 0);

    // Both steps should have been compensated
    assert_eq!(COMPENSATION_COUNT.load(Ordering::SeqCst), 2);
}
