//! AI Treasury implementation for the DEX-OS core engine
//!
//! This module implements the Priority 2 features from DEX-OS-V1.csv:
//! - "Core Components,AI Treasury,Treasury,Prediction Engine,Forecasting,High"
//! - "Core Components,AI Treasury,Treasury,Autonomous Execution,Execution,High"
//! - "Core Components,AI Treasury,Treasury,On-Chain Proposals,Proposal Management,High"
//!
//! It provides functionality for AI-driven treasury management including:
//! - Market prediction and forecasting
//! - Autonomous execution of treasury operations
//! - On-chain proposal management for treasury decisions

use crate::types::{TokenId, Quantity, TraderId};
use std::collections::HashMap;
use thiserror::Error;
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents a market prediction for a specific token
#[derive(Debug, Clone, PartialEq)]
pub struct MarketPrediction {
    /// The token being predicted
    pub token_id: TokenId,
    /// Predicted price
    pub predicted_price: f64,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    /// Timestamp of the prediction
    pub timestamp: u64,
    /// Time horizon for the prediction (in seconds)
    pub horizon: u64,
}

/// Represents a treasury proposal for on-chain voting
#[derive(Debug, Clone, PartialEq)]
pub struct TreasuryProposal {
    /// Unique identifier for the proposal
    pub id: u64,
    /// Title of the proposal
    pub title: String,
    /// Description of the proposal
    pub description: String,
    /// Proposed action (e.g., "allocate", "divest", "rebalance")
    pub action: String,
    /// Target token for the action
    pub token_id: TokenId,
    /// Amount involved in the proposal
    pub amount: Quantity,
    /// Destination address (if applicable)
    pub destination: Option<String>,
    /// Creator of the proposal
    pub creator: TraderId,
    /// Timestamp when the proposal was created
    pub created_timestamp: u64,
    /// Timestamp when voting ends
    pub voting_end_timestamp: u64,
    /// Current status of the proposal
    pub status: ProposalStatus,
    /// Votes for the proposal
    pub votes_for: u64,
    /// Votes against the proposal
    pub votes_against: u64,
    /// Required quorum for the proposal to pass
    pub required_quorum: u64,
}

/// Status of a treasury proposal
#[derive(Debug, Clone, PartialEq)]
pub enum ProposalStatus {
    /// Proposal is active and accepting votes
    Active,
    /// Proposal has passed and is ready for execution
    Passed,
    /// Proposal has been rejected
    Rejected,
    /// Proposal has been executed
    Executed,
    /// Proposal has expired without reaching quorum
    Expired,
}

/// Represents an autonomous treasury operation
#[derive(Debug, Clone, PartialEq)]
pub struct AutonomousOperation {
    /// Unique identifier for the operation
    pub id: u64,
    /// Type of operation (e.g., "rebalance", "allocate", "divest")
    pub operation_type: String,
    /// Target token for the operation
    pub token_id: TokenId,
    /// Amount involved in the operation
    pub amount: Quantity,
    /// Destination address (if applicable)
    pub destination: Option<String>,
    /// Priority level (1-5, where 1 is highest priority)
    pub priority: u8,
    /// Timestamp when the operation was created
    pub created_timestamp: u64,
    /// Timestamp when the operation should be executed
    pub execution_timestamp: u64,
    /// Status of the operation
    pub status: OperationStatus,
}

/// Status of an autonomous operation
#[derive(Debug, Clone, PartialEq)]
pub enum OperationStatus {
    /// Operation is pending execution
    Pending,
    /// Operation is being executed
    Executing,
    /// Operation completed successfully
    Completed,
    /// Operation failed
    Failed,
    /// Operation was cancelled
    Cancelled,
}

/// AI Treasury manager
#[derive(Debug, Clone)]
pub struct AITreasury {
    /// Treasury assets
    assets: HashMap<TokenId, Quantity>,
    /// Market predictions
    predictions: Vec<MarketPrediction>,
    /// Treasury proposals
    proposals: HashMap<u64, TreasuryProposal>,
    /// Autonomous operations
    operations: HashMap<u64, AutonomousOperation>,
    /// Proposal counter for generating unique IDs
    proposal_counter: u64,
    /// Operation counter for generating unique IDs
    operation_counter: u64,
}

/// Errors that can occur in the AI Treasury
#[derive(Debug, Error)]
pub enum AITreasuryError {
    #[error("Insufficient funds for operation")]
    InsufficientFunds,
    #[error("Proposal not found")]
    ProposalNotFound,
    #[error("Operation not found")]
    OperationNotFound,
    #[error("Proposal is not active")]
    ProposalNotActive,
    #[error("Invalid vote direction")]
    InvalidVoteDirection,
    #[error("Voting has ended for this proposal")]
    VotingEnded,
    #[error("Operation is not pending")]
    OperationNotPending,
    #[error("Invalid operation priority")]
    InvalidOperationPriority,
}

impl AITreasury {
    /// Create a new AI Treasury
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            predictions: Vec::new(),
            proposals: HashMap::new(),
            operations: HashMap::new(),
            proposal_counter: 0,
            operation_counter: 0,
        }
    }

    /// Add assets to the treasury
    pub fn deposit(&mut self, token_id: TokenId, amount: Quantity) {
        let current_amount = self.assets.get(&token_id).copied().unwrap_or(0);
        self.assets.insert(token_id, current_amount + amount);
    }

    /// Get the balance of a specific token in the treasury
    pub fn get_balance(&self, token_id: &TokenId) -> Quantity {
        self.assets.get(token_id).copied().unwrap_or(0)
    }

    /// Get all asset balances
    pub fn get_all_balances(&self) -> &HashMap<TokenId, Quantity> {
        &self.assets
    }

    /// Add a market prediction
    pub fn add_prediction(&mut self, prediction: MarketPrediction) {
        self.predictions.push(prediction);
    }

    /// Get recent predictions for a token
    pub fn get_predictions_for_token(&self, token_id: &TokenId, limit: usize) -> Vec<&MarketPrediction> {
        self.predictions
            .iter()
            .filter(|p| &p.token_id == token_id)
            .take(limit)
            .collect()
    }

    /// Get all predictions with confidence above threshold
    pub fn get_high_confidence_predictions(&self, min_confidence: f64) -> Vec<&MarketPrediction> {
        self.predictions
            .iter()
            .filter(|p| p.confidence >= min_confidence)
            .collect()
    }

    /// Create a new treasury proposal
    pub fn create_proposal(
        &mut self,
        title: String,
        description: String,
        action: String,
        token_id: TokenId,
        amount: Quantity,
        destination: Option<String>,
        creator: TraderId,
        voting_duration: u64, // in seconds
        required_quorum: u64,
    ) -> u64 {
        self.proposal_counter += 1;
        let proposal_id = self.proposal_counter;
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let proposal = TreasuryProposal {
            id: proposal_id,
            title,
            description,
            action,
            token_id,
            amount,
            destination,
            creator,
            created_timestamp: now,
            voting_end_timestamp: now + voting_duration,
            status: ProposalStatus::Active,
            votes_for: 0,
            votes_against: 0,
            required_quorum,
        };
        
        self.proposals.insert(proposal_id, proposal);
        proposal_id
    }

    /// Vote on a treasury proposal
    pub fn vote_on_proposal(
        &mut self,
        proposal_id: u64,
        vote_for: bool,
    ) -> Result<(), AITreasuryError> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or(AITreasuryError::ProposalNotFound)?;
        
        // Check if proposal is active
        if !matches!(proposal.status, ProposalStatus::Active) {
            return Err(AITreasuryError::ProposalNotActive);
        }
        
        // Check if voting has ended
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if now > proposal.voting_end_timestamp {
            // Update status to expired if quorum not reached
            if proposal.votes_for + proposal.votes_against < proposal.required_quorum {
                proposal.status = ProposalStatus::Expired;
            } else if proposal.votes_for > proposal.votes_against {
                proposal.status = ProposalStatus::Passed;
            } else {
                proposal.status = ProposalStatus::Rejected;
            }
            return Err(AITreasuryError::VotingEnded);
        }
        
        // Record the vote
        if vote_for {
            proposal.votes_for += 1;
        } else {
            proposal.votes_against += 1;
        }
        
        Ok(())
    }

    /// Get a proposal by ID
    pub fn get_proposal(&self, proposal_id: u64) -> Option<&TreasuryProposal> {
        self.proposals.get(&proposal_id)
    }

    /// Get all active proposals
    pub fn get_active_proposals(&self) -> Vec<&TreasuryProposal> {
        self.proposals
            .values()
            .filter(|p| matches!(p.status, ProposalStatus::Active))
            .collect()
    }

    /// Check if a proposal has passed (quorum reached and more votes for than against)
    pub fn is_proposal_passed(&self, proposal_id: u64) -> Result<bool, AITreasuryError> {
        let proposal = self.proposals.get(&proposal_id)
            .ok_or(AITreasuryError::ProposalNotFound)?;
        
        // Check if voting has ended
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if now <= proposal.voting_end_timestamp {
            return Ok(false); // Voting still active
        }
        
        // Check quorum
        let total_votes = proposal.votes_for + proposal.votes_against;
        if total_votes < proposal.required_quorum {
            return Ok(false); // Quorum not reached
        }
        
        // Check if more votes for than against
        Ok(proposal.votes_for > proposal.votes_against)
    }

    /// Execute a passed proposal
    pub fn execute_proposal(&mut self, proposal_id: u64) -> Result<(), AITreasuryError> {
        // First, get all the necessary data without borrowing
        let (action, token_id, amount) = {
            let proposal = self.proposals.get(&proposal_id)
                .ok_or(AITreasuryError::ProposalNotFound)?;
            
            // Check if proposal has passed
            if !matches!(proposal.status, ProposalStatus::Passed) {
                return Err(AITreasuryError::ProposalNotActive);
            }
            
            (proposal.action.clone(), proposal.token_id.clone(), proposal.amount)
        };
        
        // Execute the proposal action based on the cloned data
        match action.as_str() {
            "allocate" => {
                // Check if we have sufficient funds
                let balance = self.get_balance(&token_id);
                if amount > balance {
                    return Err(AITreasuryError::InsufficientFunds);
                }
                
                // Deduct from treasury
                self.assets.insert(
                    token_id,
                    balance - amount
                );
            },
            "divest" => {
                // Add to treasury
                let balance = self.get_balance(&token_id);
                self.assets.insert(
                    token_id,
                    balance + amount
                );
            },
            _ => {
                // For other actions, we might need custom logic
                // For now, we'll just mark as executed
            }
        }
        
        // Update proposal status
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or(AITreasuryError::ProposalNotFound)?;
        proposal.status = ProposalStatus::Executed;
        
        Ok(())
    }

    /// Create an autonomous operation
    pub fn create_autonomous_operation(
        &mut self,
        operation_type: String,
        token_id: TokenId,
        amount: Quantity,
        destination: Option<String>,
        priority: u8,
        execution_delay: u64, // in seconds
    ) -> Result<u64, AITreasuryError> {
        // Validate priority
        if priority < 1 || priority > 5 {
            return Err(AITreasuryError::InvalidOperationPriority);
        }
        
        self.operation_counter += 1;
        let operation_id = self.operation_counter;
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let operation = AutonomousOperation {
            id: operation_id,
            operation_type,
            token_id,
            amount,
            destination,
            priority,
            created_timestamp: now,
            execution_timestamp: now + execution_delay,
            status: OperationStatus::Pending,
        };
        
        self.operations.insert(operation_id, operation);
        Ok(operation_id)
    }

    /// Get an autonomous operation by ID
    pub fn get_operation(&self, operation_id: u64) -> Option<&AutonomousOperation> {
        self.operations.get(&operation_id)
    }

    /// Get all pending operations
    pub fn get_pending_operations(&self) -> Vec<&AutonomousOperation> {
        self.operations
            .values()
            .filter(|o| matches!(o.status, OperationStatus::Pending))
            .collect()
    }

    /// Execute an autonomous operation
    pub fn execute_operation(&mut self, operation_id: u64) -> Result<(), AITreasuryError> {
        // First, get all the necessary data without borrowing
        let (operation_type, token_id, amount, execution_timestamp) = {
            let operation = self.operations.get(&operation_id)
                .ok_or(AITreasuryError::OperationNotFound)?;
            
            // Check if operation is pending
            if !matches!(operation.status, OperationStatus::Pending) {
                return Err(AITreasuryError::OperationNotPending);
            }
            
            // Check if it's time to execute
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            if now < operation.execution_timestamp {
                return Ok(()); // Not time to execute yet
            }
            
            (operation.operation_type.clone(), operation.token_id.clone(), operation.amount, operation.execution_timestamp)
        };
        
        // Mark as executing
        {
            let operation = self.operations.get_mut(&operation_id)
                .ok_or(AITreasuryError::OperationNotFound)?;
            operation.status = OperationStatus::Executing;
        }
        
        // Execute the operation based on the cloned data
        match operation_type.as_str() {
            "rebalance" => {
                // Rebalancing logic would go here
                // For now, we'll just mark as completed
            },
            "allocate" => {
                // Check if we have sufficient funds
                let balance = self.get_balance(&token_id);
                if amount > balance {
                    let operation = self.operations.get_mut(&operation_id)
                        .ok_or(AITreasuryError::OperationNotFound)?;
                    operation.status = OperationStatus::Failed;
                    return Err(AITreasuryError::InsufficientFunds);
                }
                
                // Deduct from treasury
                self.assets.insert(
                    token_id,
                    balance - amount
                );
            },
            "divest" => {
                // Add to treasury
                let balance = self.get_balance(&token_id);
                self.assets.insert(
                    token_id,
                    balance + amount
                );
            },
            _ => {
                // For other operations, we might need custom logic
                // For now, we'll just mark as completed
            }
        }
        
        // Mark as completed
        let operation = self.operations.get_mut(&operation_id)
            .ok_or(AITreasuryError::OperationNotFound)?;
        operation.status = OperationStatus::Completed;
        
        Ok(())
    }

    /// Cancel an autonomous operation
    pub fn cancel_operation(&mut self, operation_id: u64) -> Result<(), AITreasuryError> {
        let operation = self.operations.get_mut(&operation_id)
            .ok_or(AITreasuryError::OperationNotFound)?;
        
        // Check if operation is pending
        if !matches!(operation.status, OperationStatus::Pending) {
            return Err(AITreasuryError::OperationNotPending);
        }
        
        // Mark as cancelled
        operation.status = OperationStatus::Cancelled;
        
        Ok(())
    }

    /// Get the number of proposals
    pub fn proposal_count(&self) -> usize {
        self.proposals.len()
    }

    /// Get the number of operations
    pub fn operation_count(&self) -> usize {
        self.operations.len()
    }
}

impl Default for AITreasury {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_treasury_creation() {
        let treasury = AITreasury::new();
        assert_eq!(treasury.proposal_count(), 0);
        assert_eq!(treasury.operation_count(), 0);
    }

    #[test]
    fn test_asset_management() {
        let mut treasury = AITreasury::new();
        let token_id = "BTC".to_string();
        let amount = 1000;
        
        // Test deposit
        treasury.deposit(token_id.clone(), amount);
        assert_eq!(treasury.get_balance(&token_id), amount);
        
        // Test get all balances
        let balances = treasury.get_all_balances();
        assert_eq!(balances.len(), 1);
        assert_eq!(balances.get(&token_id), Some(&amount));
    }

    #[test]
    fn test_market_predictions() {
        let mut treasury = AITreasury::new();
        let token_id = "BTC".to_string();
        
        let prediction = MarketPrediction {
            token_id: token_id.clone(),
            predicted_price: 50000.0,
            confidence: 0.85,
            timestamp: 1234567890,
            horizon: 86400, // 1 day
        };
        
        treasury.add_prediction(prediction.clone());
        
        // Test getting predictions for token
        let predictions = treasury.get_predictions_for_token(&token_id, 5);
        assert_eq!(predictions.len(), 1);
        assert_eq!(predictions[0], &prediction);
        
        // Test getting high confidence predictions
        let high_confidence = treasury.get_high_confidence_predictions(0.8);
        assert_eq!(high_confidence.len(), 1);
        assert_eq!(high_confidence[0], &prediction);
    }

    #[test]
    fn test_proposal_creation_and_voting() {
        let mut treasury = AITreasury::new();
        let creator = "creator1".to_string();
        let token_id = "BTC".to_string();
        
        // Create a proposal
        let proposal_id = treasury.create_proposal(
            "Allocate BTC".to_string(),
            "Allocate 100 BTC to investment fund".to_string(),
            "allocate".to_string(),
            token_id.clone(),
            100,
            Some("investment_fund".to_string()),
            creator.clone(),
            86400, // 1 day voting
            10, // required quorum
        );
        
        assert_eq!(proposal_id, 1);
        assert_eq!(treasury.proposal_count(), 1);
        
        // Get the proposal
        let proposal = treasury.get_proposal(proposal_id);
        assert!(proposal.is_some());
        let proposal = proposal.unwrap();
        assert_eq!(proposal.title, "Allocate BTC");
        assert_eq!(proposal.creator, creator);
        assert_eq!(proposal.status, ProposalStatus::Active);
        
        // Test voting
        assert!(treasury.vote_on_proposal(proposal_id, true).is_ok());
        assert!(treasury.vote_on_proposal(proposal_id, false).is_ok());
        
        // Get updated proposal
        let proposal = treasury.get_proposal(proposal_id).unwrap();
        assert_eq!(proposal.votes_for, 1);
        assert_eq!(proposal.votes_against, 1);
    }

    #[test]
    fn test_autonomous_operations() {
        let mut treasury = AITreasury::new();
        let token_id = "BTC".to_string();
        
        // Create an operation
        let result = treasury.create_autonomous_operation(
            "allocate".to_string(),
            token_id.clone(),
            100,
            Some("destination".to_string()),
            1, // highest priority
            3600, // execute in 1 hour
        );
        
        assert!(result.is_ok());
        let operation_id = result.unwrap();
        assert_eq!(operation_id, 1);
        assert_eq!(treasury.operation_count(), 1);
        
        // Get the operation
        let operation = treasury.get_operation(operation_id);
        assert!(operation.is_some());
        let operation = operation.unwrap();
        assert_eq!(operation.operation_type, "allocate");
        assert_eq!(operation.priority, 1);
        assert_eq!(operation.status, OperationStatus::Pending);
    }
}