//! Timelock execution module for DEX-OS governance
//!
//! This module implements the Timelock Execution feature for DAO Governance,
//! allowing approved proposals to be executed only after a predetermined delay period.

use crate::governance::{Proposal, ProposalStatus, GovernanceAction, GovernanceError};
use crate::types::{TraderId, TokenId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents a scheduled execution operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledOperation {
    /// The proposal ID associated with this operation
    pub proposal_id: String,
    /// The actions to be executed
    pub actions: Vec<GovernanceAction>,
    /// The earliest timestamp when this operation can be executed
    pub eta: u64,
    /// Whether this operation has been executed
    pub executed: bool,
    /// Who scheduled this operation
    pub scheduler: TraderId,
}

/// Timelock controller for governing proposal executions
#[derive(Debug, Clone)]
pub struct TimelockController {
    /// Scheduled operations awaiting execution
    scheduled_operations: HashMap<String, ScheduledOperation>,
    /// Minimum delay for all operations (in seconds)
    min_delay: u64,
    /// Maximum delay for all operations (in seconds)
    max_delay: u64,
    /// Authorized executors who can execute operations
    executors: Vec<TraderId>,
    /// Authorized schedulers who can schedule operations
    schedulers: Vec<TraderId>,
}

impl TimelockController {
    /// Create a new TimelockController with specified parameters
    pub fn new(min_delay: u64, max_delay: u64) -> Self {
        Self {
            scheduled_operations: HashMap::new(),
            min_delay,
            max_delay,
            executors: Vec::new(),
            schedulers: Vec::new(),
        }
    }

    /// Add an authorized executor
    pub fn add_executor(&mut self, executor: TraderId) {
        if !self.executors.contains(&executor) {
            self.executors.push(executor);
        }
    }

    /// Add an authorized scheduler
    pub fn add_scheduler(&mut self, scheduler: TraderId) {
        if !self.schedulers.contains(&scheduler) {
            self.schedulers.push(scheduler);
        }
    }

    /// Check if an address is an authorized executor
    pub fn is_authorized_executor(&self, executor: &TraderId) -> bool {
        self.executors.contains(executor)
    }

    /// Check if an address is an authorized scheduler
    pub fn is_authorized_scheduler(&self, scheduler: &TraderId) -> bool {
        self.schedulers.contains(scheduler)
    }

    /// Schedule a proposal for execution after the delay
    pub fn schedule_proposal_execution(
        &mut self,
        proposal: &Proposal,
        scheduler: TraderId,
        delay: Option<u64>,
    ) -> Result<String, GovernanceError> {
        // Check if the scheduler is authorized
        if !self.is_authorized_scheduler(&scheduler) {
            return Err(GovernanceError::UnauthorizedScheduler);
        }

        // Check if proposal is in the correct state
        if proposal.status != ProposalStatus::Passed {
            return Err(GovernanceError::ProposalNotPassed);
        }

        // Check if proposal has execution plan
        let execution_plan = proposal.execution_plan.as_ref().ok_or(GovernanceError::ExecutionPlanMissing)?;

        // Validate delay
        let delay = delay.unwrap_or(self.min_delay);
        if delay < self.min_delay || delay > self.max_delay {
            return Err(GovernanceError::InvalidDelay);
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let eta = now + delay;

        let operation_id = format!("operation_{}", now);

        let scheduled_operation = ScheduledOperation {
            proposal_id: proposal.id.clone(),
            actions: execution_plan.actions.clone(),
            eta,
            executed: false,
            scheduler,
        };

        self.scheduled_operations.insert(operation_id.clone(), scheduled_operation);

        Ok(operation_id)
    }

    /// Execute a scheduled operation
    pub fn execute_operation(
        &mut self,
        operation_id: &str,
        executor: TraderId,
    ) -> Result<Vec<GovernanceActionResult>, GovernanceError> {
        // Check if the executor is authorized
        if !self.is_authorized_executor(&executor) {
            return Err(GovernanceError::UnauthorizedExecutor);
        }

        // Get all the data we need first to avoid borrowing issues
        let (actions, eta, already_executed) = {
            let operation = self.scheduled_operations.get(operation_id)
                .ok_or(GovernanceError::OperationNotFound)?;
            (operation.actions.clone(), operation.eta, operation.executed)
        };

        // Check if operation has already been executed
        if already_executed {
            return Err(GovernanceError::OperationAlreadyExecuted);
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Check if the operation is ready to be executed
        if now < eta {
            return Err(GovernanceError::OperationNotReady);
        }

        // Execute the actions
        let results = self.execute_actions(&actions)?;

        // Mark as executed
        if let Some(operation) = self.scheduled_operations.get_mut(operation_id) {
            operation.executed = true;
        }

        Ok(results)
    }

    /// Cancel a scheduled operation
    pub fn cancel_operation(
        &mut self,
        operation_id: &str,
        canceller: TraderId,
    ) -> Result<(), GovernanceError> {
        // Check if the canceller is authorized (either the scheduler or an executor)
        if !self.is_authorized_scheduler(&canceller) && !self.is_authorized_executor(&canceller) {
            return Err(GovernanceError::UnauthorizedCancellation);
        }

        // Get the execution status first to avoid borrowing issues
        let already_executed = self.scheduled_operations.get(operation_id)
            .ok_or(GovernanceError::OperationNotFound)?
            .executed;

        // Check if operation has already been executed
        if already_executed {
            return Err(GovernanceError::OperationAlreadyExecuted);
        }

        // Remove the operation
        self.scheduled_operations.remove(operation_id);

        Ok(())
    }

    /// Execute governance actions
    fn execute_actions(
        &self,
        actions: &[GovernanceAction],
    ) -> Result<Vec<GovernanceActionResult>, GovernanceError> {
        let mut results = Vec::new();

        for action in actions {
            let result = match action {
                GovernanceAction::SetParameter { key, value } => {
                    // In a real implementation, this would interact with the protocol's parameter store
                    GovernanceActionResult::ParameterSet {
                        key: key.clone(),
                        value: value.clone(),
                        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    }
                }
                GovernanceAction::TransferTreasury { to, token, amount } => {
                    // In a real implementation, this would transfer tokens from the treasury
                    GovernanceActionResult::TreasuryTransfer {
                        to: to.clone(),
                        token: token.clone(),
                        amount: *amount,
                        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    }
                }
                GovernanceAction::UpgradeProtocol { new_version, code_hash } => {
                    // In a real implementation, this would initiate a protocol upgrade
                    GovernanceActionResult::ProtocolUpgraded {
                        new_version: new_version.clone(),
                        code_hash: code_hash.clone(),
                        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    }
                }
                GovernanceAction::AddMarket { base_token, quote_token } => {
                    // In a real implementation, this would add a new market
                    GovernanceActionResult::MarketAdded {
                        base_token: base_token.clone(),
                        quote_token: quote_token.clone(),
                        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    }
                }
            };
            results.push(result);
        }

        Ok(results)
    }

    /// Get a scheduled operation by ID
    pub fn get_scheduled_operation(&self, operation_id: &str) -> Option<&ScheduledOperation> {
        self.scheduled_operations.get(operation_id)
    }

    /// Get all scheduled operations
    pub fn get_all_scheduled_operations(&self) -> Vec<&ScheduledOperation> {
        self.scheduled_operations.values().collect()
    }

    /// Get minimum delay
    pub fn min_delay(&self) -> u64 {
        self.min_delay
    }

    /// Get maximum delay
    pub fn max_delay(&self) -> u64 {
        self.max_delay
    }
}

/// Result of executing a governance action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GovernanceActionResult {
    ParameterSet {
        key: String,
        value: String,
        timestamp: u64,
    },
    TreasuryTransfer {
        to: TraderId,
        token: TokenId,
        amount: u64,
        timestamp: u64,
    },
    ProtocolUpgraded {
        new_version: String,
        code_hash: String,
        timestamp: u64,
    },
    MarketAdded {
        base_token: TokenId,
        quote_token: TokenId,
        timestamp: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::governance::{Proposal, ProposalType, Proposer, Votes, ExecutionPlan, GovernanceAction};

    fn create_test_proposal() -> Proposal {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        Proposal {
            id: "test_proposal".to_string(),
            title: "Test Proposal".to_string(),
            description: "A test proposal for timelock execution".to_string(),
            proposal_type: ProposalType::ParameterChange,
            proposer: Proposer::Human {
                trader_id: "proposer1".to_string(),
            },
            created_at: now,
            voting_start: now,
            voting_end: now + 3600,
            status: ProposalStatus::Passed,
            votes: Votes {
                yes_votes: HashMap::new(),
                no_votes: HashMap::new(),
                abstain_votes: HashMap::new(),
                total_voting_power: 0,
            },
            execution_plan: Some(ExecutionPlan {
                actions: vec![
                    GovernanceAction::SetParameter {
                        key: "test_param".to_string(),
                        value: "test_value".to_string(),
                    }
                ],
                execution_time: now + 7200,
                requires_confirmation: false,
            }),
            ai_analysis: None,
            reference_control: None,
            reference_acknowledged: true,
        }
    }

    #[test]
    fn test_timelock_controller_creation() {
        let controller = TimelockController::new(3600, 86400);
        assert_eq!(controller.min_delay(), 3600);
        assert_eq!(controller.max_delay(), 86400);
        assert!(controller.get_all_scheduled_operations().is_empty());
    }

    #[test]
    fn test_authorization_management() {
        let mut controller = TimelockController::new(3600, 86400);
        let executor = "executor1".to_string();
        let scheduler = "scheduler1".to_string();

        assert!(!controller.is_authorized_executor(&executor));
        assert!(!controller.is_authorized_scheduler(&scheduler));

        controller.add_executor(executor.clone());
        controller.add_scheduler(scheduler.clone());

        assert!(controller.is_authorized_executor(&executor));
        assert!(controller.is_authorized_scheduler(&scheduler));
    }

    #[test]
    fn test_schedule_proposal_execution() {
        let mut controller = TimelockController::new(3600, 86400);
        let scheduler = "scheduler1".to_string();
        controller.add_scheduler(scheduler.clone());

        let proposal = create_test_proposal();
        let result = controller.schedule_proposal_execution(&proposal, scheduler, Some(7200));

        assert!(result.is_ok());
        let operation_id = result.unwrap();
        assert!(!operation_id.is_empty());

        let operation = controller.get_scheduled_operation(&operation_id);
        assert!(operation.is_some());
        let operation = operation.unwrap();
        assert_eq!(operation.proposal_id, proposal.id);
        assert_eq!(operation.actions.len(), 1);
        assert!(!operation.executed);
    }

    #[test]
    fn test_schedule_unauthorized_scheduler() {
        let mut controller = TimelockController::new(3600, 86400);
        let unauthorized_scheduler = "unauthorized_scheduler".to_string();

        let proposal = create_test_proposal();
        let result = controller.schedule_proposal_execution(&proposal, unauthorized_scheduler, None);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GovernanceError::UnauthorizedScheduler));
    }

    #[test]
    fn test_schedule_proposal_not_passed() {
        let mut controller = TimelockController::new(3600, 86400);
        let scheduler = "scheduler1".to_string();
        controller.add_scheduler(scheduler.clone());

        let mut proposal = create_test_proposal();
        proposal.status = ProposalStatus::Active; // Not passed

        let result = controller.schedule_proposal_execution(&proposal, scheduler, None);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GovernanceError::ProposalNotPassed));
    }

    #[test]
    fn test_execute_operation() {
        let mut controller = TimelockController::new(1, 86400); // 1 second min delay for testing
        let scheduler = "scheduler1".to_string();
        let executor = "executor1".to_string();
        controller.add_scheduler(scheduler.clone());
        controller.add_executor(executor.clone());

        let proposal = create_test_proposal();
        let operation_id = controller.schedule_proposal_execution(&proposal, scheduler, Some(1)).unwrap();

        // Wait a bit to ensure the operation is ready
        std::thread::sleep(std::time::Duration::from_secs(2));

        let result = controller.execute_operation(&operation_id, executor);
        assert!(result.is_ok());

        let results = result.unwrap();
        assert_eq!(results.len(), 1);
        match &results[0] {
            GovernanceActionResult::ParameterSet { key, value, .. } => {
                assert_eq!(key, "test_param");
                assert_eq!(value, "test_value");
            }
            _ => panic!("Unexpected result type"),
        }

        // Verify the operation is marked as executed
        let operation = controller.get_scheduled_operation(&operation_id).unwrap();
        assert!(operation.executed);
    }

    #[test]
    fn test_execute_unauthorized() {
        let mut controller = TimelockController::new(3600, 86400);
        let unauthorized_executor = "unauthorized_executor".to_string();
        let scheduler = "scheduler1".to_string();
        controller.add_scheduler(scheduler.clone());

        let proposal = create_test_proposal();
        let operation_id = controller.schedule_proposal_execution(&proposal, scheduler, None).unwrap();

        let result = controller.execute_operation(&operation_id, unauthorized_executor);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GovernanceError::UnauthorizedExecutor));
    }

    #[test]
    fn test_execute_not_ready() {
        let mut controller = TimelockController::new(3600, 86400); // 1 hour min delay
        let scheduler = "scheduler1".to_string();
        let executor = "executor1".to_string();
        controller.add_scheduler(scheduler.clone());
        controller.add_executor(executor.clone());

        let proposal = create_test_proposal();
        let operation_id = controller.schedule_proposal_execution(&proposal, scheduler, None).unwrap();

        // Try to execute immediately (not ready yet)
        let result = controller.execute_operation(&operation_id, executor);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GovernanceError::OperationNotReady));
    }

    #[test]
    fn test_cancel_operation() {
        let mut controller = TimelockController::new(3600, 86400);
        let scheduler = "scheduler1".to_string();
        let executor = "executor1".to_string();
        controller.add_scheduler(scheduler.clone());
        controller.add_executor(executor.clone());

        let proposal = create_test_proposal();
        let operation_id = controller.schedule_proposal_execution(&proposal, scheduler.clone(), None).unwrap();

        // Cancel as scheduler
        let result = controller.cancel_operation(&operation_id, scheduler);
        assert!(result.is_ok());
        assert!(controller.get_scheduled_operation(&operation_id).is_none());
    }

    #[test]
    fn test_cancel_executed_operation() {
        let mut controller = TimelockController::new(1, 86400); // 1 second min delay for testing
        let scheduler = "scheduler1".to_string();
        let executor = "executor1".to_string();
        controller.add_scheduler(scheduler.clone());
        controller.add_executor(executor.clone());

        let proposal = create_test_proposal();
        let operation_id = controller.schedule_proposal_execution(&proposal, scheduler.clone(), Some(1)).unwrap();

        // Wait a bit to ensure the operation is ready
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Execute the operation first
        controller.execute_operation(&operation_id, executor.clone()).unwrap();

        // Try to cancel the executed operation
        let result = controller.cancel_operation(&operation_id, scheduler);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GovernanceError::OperationAlreadyExecuted));
    }
}