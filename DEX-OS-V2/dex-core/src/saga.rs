//! Saga Pattern implementation for distributed transactions.
//!
//! Implements Priority 3 feature:
//! "Distributed Systems,Distributed Systems,Distributed Systems,Saga Pattern,Distributed Transactions,Medium"
//!
//! The Saga pattern is a sequence of local transactions where each transaction updates data
//! within a single service. The first transaction in a saga is initiated by an external request
//! corresponding to the system operation, and then each subsequent step is triggered by the
//! previous completion.
//!
//! If a step fails, the saga executes compensating transactions (rollback) in reverse order
//! to undo the changes made by previous steps.

use std::fmt::Debug;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

/// Errors that can occur during saga execution.
#[derive(Error, Debug, PartialEq)]
pub enum SagaError {
    /// A step in the saga failed.
    #[error("Step {step_index} failed: {message}")]
    StepFailed {
        /// Index of the step that failed.
        step_index: usize,
        /// Error message from the failed step.
        message: String,
    },
    /// A compensation step failed during rollback.
    #[error("Compensation for step {step_index} failed: {message}")]
    CompensationFailed {
        /// Index of the step whose compensation failed.
        step_index: usize,
        /// Error message from the failed compensation.
        message: String,
    },
    /// The saga has already been executed.
    #[error("Saga has already been executed")]
    AlreadyExecuted,
    /// The saga is currently executing.
    #[error("Saga is currently executing")]
    Executing,
}

/// Represents a single step in a saga.
pub struct SagaStep<T, E> {
    /// Name of the step for logging and debugging.
    pub name: String,
    /// Function to execute the step.
    pub action: Arc<dyn Fn() -> Result<T, E> + Send + Sync>,
    /// Function to compensate (undo) the step.
    pub compensate: Arc<dyn Fn() -> Result<(), E> + Send + Sync>,
}

// Manual Debug implementation since dyn Fn doesn't implement Debug
impl<T, E> std::fmt::Debug for SagaStep<T, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SagaStep")
            .field("name", &self.name)
            .finish()
    }
}

// Manual Clone implementation
impl<T, E> Clone for SagaStep<T, E> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            action: self.action.clone(),
            compensate: self.compensate.clone(),
        }
    }
}

impl<T, E> SagaStep<T, E> {
    /// Create a new saga step.
    pub fn new<F, C>(
        name: impl Into<String>,
        action: F,
        compensate: C,
    ) -> Self
    where
        F: Fn() -> Result<T, E> + Send + Sync + 'static,
        C: Fn() -> Result<(), E> + Send + Sync + 'static,
    {
        Self {
            name: name.into(),
            action: Arc::new(action),
            compensate: Arc::new(compensate),
        }
    }
}

/// Represents the execution status of a saga.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SagaStatus {
    /// The saga has not been executed yet.
    Pending,
    /// The saga is currently executing.
    Executing,
    /// The saga completed successfully.
    Success,
    /// The saga failed and was rolled back.
    Failed,
}

/// A saga orchestrator that manages distributed transactions.
pub struct SagaOrchestrator<T, E> {
    /// Steps in the saga.
    steps: Vec<SagaStep<T, E>>,
    /// Current execution status.
    status: Arc<Mutex<SagaStatus>>,
    /// Results of executed steps.
    results: Arc<Mutex<Vec<Option<T>>>>,
}

impl<T, E> SagaOrchestrator<T, E>
where
    T: Send + Clone,
    E: Send + Debug,
{
    /// Create a new saga orchestrator.
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            status: Arc::new(Mutex::new(SagaStatus::Pending)),
            results: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add a step to the saga.
    pub fn add_step<F, C>(&mut self, name: impl Into<String>, action: F, compensate: C)
    where
        F: Fn() -> Result<T, E> + Send + Sync + 'static,
        C: Fn() -> Result<(), E> + Send + Sync + 'static,
    {
        self.steps
            .push(SagaStep::new(name, action, compensate));
    }

    /// Execute the saga.
    ///
    /// Returns the results of all successful steps if the saga completes successfully,
    /// or an error if any step fails.
    pub async fn execute(&self) -> Result<Vec<Option<T>>, SagaError>
    where
        T: Clone,
        E: ToString,
    {
        {
            let mut status = self.status.lock().await;
            match *status {
                SagaStatus::Executing => return Err(SagaError::Executing),
                SagaStatus::Success | SagaStatus::Failed => {
                    return Err(SagaError::AlreadyExecuted)
                }
                SagaStatus::Pending => {
                    *status = SagaStatus::Executing;
                }
            }
        }

        let mut results = vec![None; self.steps.len()];
        let mut executed_steps = 0;

        // Execute steps forward
        for (index, step) in self.steps.iter().enumerate() {
            match (step.action)() {
                Ok(result) => {
                    results[index] = Some(result);
                    executed_steps = index + 1;
                }
                Err(e) => {
                    // Rollback previously executed steps
                    self.rollback(executed_steps).await?;
                    *self.status.lock().await = SagaStatus::Failed;
                    return Err(SagaError::StepFailed {
                        step_index: index,
                        message: e.to_string(),
                    });
                }
            }
        }

        *self.results.lock().await = results.clone();
        *self.status.lock().await = SagaStatus::Success;
        Ok(results)
    }

    /// Rollback the saga by executing compensation steps in reverse order.
    async fn rollback(&self, executed_steps: usize) -> Result<(), SagaError>
    where
        E: ToString,
    {
        for i in (0..executed_steps).rev() {
            let step = &self.steps[i];
            if let Err(e) = (step.compensate)() {
                return Err(SagaError::CompensationFailed {
                    step_index: i,
                    message: e.to_string(),
                });
            }
        }
        Ok(())
    }

    /// Get the current status of the saga.
    pub async fn status(&self) -> SagaStatus {
        self.status.lock().await.clone()
    }

    /// Get the results of executed steps.
    pub async fn results(&self) -> Vec<Option<T>>
    where
        T: Clone,
    {
        self.results.lock().await.clone()
    }
}

impl<T, E> Default for SagaOrchestrator<T, E>
where
    T: Send + Clone,
    E: Send + Debug,
{
    fn default() -> Self {
        Self::new()
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_successful_saga() {
        let mut saga = SagaOrchestrator::<String, String>::new();

        saga.add_step(
            "step1",
            || Ok("result1".to_string()),
            || Ok(()),
        );

        saga.add_step(
            "step2",
            || Ok("result2".to_string()),
            || Ok(()),
        );

        let results = saga.execute().await.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], Some("result1".to_string()));
        assert_eq!(results[1], Some("result2".to_string()));

        assert_eq!(saga.status().await, SagaStatus::Success);
    }

    #[tokio::test]
    async fn test_saga_with_failure_and_compensation() {
        static STEP1_EXECUTED: AtomicBool = AtomicBool::new(false);
        static STEP1_COMPENSATED: AtomicBool = AtomicBool::new(false);
        static STEP2_EXECUTED: AtomicBool = AtomicBool::new(false);

        let mut saga = SagaOrchestrator::<String, String>::new();

        saga.add_step(
            "step1",
            || {
                STEP1_EXECUTED.store(true, Ordering::SeqCst);
                Ok("result1".to_string())
            },
            || {
                STEP1_COMPENSATED.store(true, Ordering::SeqCst);
                Ok(())
            },
        );

        saga.add_step(
            "step2",
            || {
                STEP2_EXECUTED.store(true, Ordering::SeqCst);
                Err("step2 failed".to_string())
            },
            || Ok(()),
        );

        let result = saga.execute().await;
        assert!(result.is_err());
        assert_eq!(saga.status().await, SagaStatus::Failed);

        // Check that compensation was called
        assert!(STEP1_EXECUTED.load(Ordering::SeqCst));
        assert!(STEP1_COMPENSATED.load(Ordering::SeqCst));
        assert!(STEP2_EXECUTED.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_empty_saga() {
        let saga = SagaOrchestrator::<String, String>::new();
        let results = saga.execute().await.unwrap();
        assert_eq!(results.len(), 0);
        assert_eq!(saga.status().await, SagaStatus::Success);
    }

    #[tokio::test]
    async fn test_cannot_execute_twice() {
        let mut saga = SagaOrchestrator::<String, String>::new();
        saga.add_step("step1", || Ok("result".to_string()), || Ok(()));

        // First execution should succeed
        assert!(saga.execute().await.is_ok());

        // Second execution should fail
        let result = saga.execute().await;
        assert!(matches!(result, Err(SagaError::AlreadyExecuted)));
    }

    #[tokio::test]
    async fn test_compensation_failure() {
        static COMPENSATION_CALLED: AtomicUsize = AtomicUsize::new(0);

        let mut saga = SagaOrchestrator::<String, String>::new();

        saga.add_step(
            "step1",
            || Ok("result1".to_string()),
            || {
                COMPENSATION_CALLED.fetch_add(1, Ordering::SeqCst);
                Err("compensation failed".to_string())
            },
        );

        saga.add_step(
            "step2",
            || Err("step2 failed".to_string()),
            || Ok(()),
        );

        let result = saga.execute().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SagaError::CompensationFailed { .. }));

        // Check that compensation was attempted
        assert_eq!(COMPENSATION_CALLED.load(Ordering::SeqCst), 1);
    }
}
