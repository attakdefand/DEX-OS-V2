//! Two-Phase Commit consensus implementation for distributed transactions
//!
//! This module implements the Two-Phase Commit consensus algorithm for coordinating
//! distributed transactions in the DEX-OS infrastructure, covering the Priority 3
//! distributed-systems feature:
//! - Distributed Systems,Distributed Systems,Distributed Systems,Consensus,Two-Phase Commit,Medium

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::time::sleep;

/// Errors that can occur in the Two-Phase Commit protocol
#[derive(Error, Debug, PartialEq)]
pub enum TwoPhaseCommitError {
    /// Node is not a participant in the transaction
    #[error("Node is not a participant in the transaction")]
    NotParticipant,
    
    /// Coordinator is not available
    #[error("Coordinator is not available")]
    CoordinatorNotAvailable,
    
    /// Transaction timeout
    #[error("Transaction timeout")]
    Timeout,
    
    /// Network communication error
    #[error("Network communication error")]
    NetworkError,
    
    /// Transaction aborted
    #[error("Transaction aborted")]
    Aborted,
    
    /// Invalid transaction state
    #[error("Invalid transaction state")]
    InvalidState,
}

/// Transaction states in Two-Phase Commit
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransactionState {
    /// Initial state
    Active,
    /// Prepared to commit
    Prepared,
    /// Committed
    Committed,
    /// Aborted
    Aborted,
}

/// Participant states in Two-Phase Commit
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParticipantState {
    /// Initial state
    Working,
    /// Ready to commit
    Ready,
    /// Committed
    Committed,
    /// Aborted
    Aborted,
}

/// Two-Phase Commit node roles
#[derive(Debug, Clone, PartialEq)]
pub enum NodeRole {
    /// Transaction coordinator
    Coordinator,
    /// Transaction participant
    Participant,
}

/// Transaction identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransactionId {
    /// Unique identifier for the transaction
    pub id: String,
    /// Timestamp when the transaction was created
    pub timestamp: u64,
}

impl TransactionId {
    /// Create a new transaction ID
    pub fn new(id: String) -> Self {
        Self {
            id,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }
}

/// Two-Phase Commit configuration
#[derive(Debug, Clone)]
pub struct TwoPhaseCommitConfig {
    /// Node ID
    pub node_id: String,
    /// List of participant node addresses
    pub participant_addresses: HashMap<String, String>,
    /// Timeout for transaction operations in milliseconds
    pub timeout_ms: u64,
    /// Maximum number of retries
    pub max_retries: usize,
}

impl Default for TwoPhaseCommitConfig {
    fn default() -> Self {
        Self {
            node_id: "coordinator-0".to_string(),
            participant_addresses: HashMap::new(),
            timeout_ms: 5000,
            max_retries: 3,
        }
    }
}

/// Prepare request from coordinator to participants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrepareRequest {
    /// Transaction identifier
    pub transaction_id: TransactionId,
    /// Transaction data
    pub data: Vec<u8>,
}

/// Prepare response from participant to coordinator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrepareResponse {
    /// Transaction identifier
    pub transaction_id: TransactionId,
    /// Participant's vote (true = yes, false = no)
    pub vote: bool,
    /// Participant node ID
    pub participant_id: String,
}

/// Commit request from coordinator to participants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitRequest {
    /// Transaction identifier
    pub transaction_id: TransactionId,
}

/// Abort request from coordinator to participants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbortRequest {
    /// Transaction identifier
    pub transaction_id: TransactionId,
}

/// Acknowledgment from participant to coordinator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AckResponse {
    /// Transaction identifier
    pub transaction_id: TransactionId,
    /// Participant node ID
    pub participant_id: String,
    /// Success status
    pub success: bool,
}

/// Coordinator state
#[derive(Debug, Clone)]
pub struct CoordinatorState {
    /// Active transactions
    pub transactions: HashMap<TransactionId, TransactionState>,
    /// Participant votes for each transaction
    pub votes: HashMap<TransactionId, HashMap<String, bool>>,
    /// Transaction data
    pub transaction_data: HashMap<TransactionId, Vec<u8>>,
}

impl Default for CoordinatorState {
    fn default() -> Self {
        Self {
            transactions: HashMap::new(),
            votes: HashMap::new(),
            transaction_data: HashMap::new(),
        }
    }
}

/// Participant state
#[derive(Debug, Clone)]
pub struct ParticipantStateData {
    /// Current state
    pub state: ParticipantState,
    /// Transaction data
    pub transaction_data: HashMap<TransactionId, Vec<u8>>,
}

impl Default for ParticipantStateData {
    fn default() -> Self {
        Self {
            state: ParticipantState::Working,
            transaction_data: HashMap::new(),
        }
    }
}

/// Two-Phase Commit node implementation
pub struct TwoPhaseCommitNode {
    /// Node configuration
    config: TwoPhaseCommitConfig,
    /// Node role
    role: NodeRole,
    /// Coordinator state (if node is a coordinator)
    coordinator_state: Option<CoordinatorState>,
    /// Participant state (if node is a participant)
    participant_state: Option<ParticipantStateData>,
    /// Timeout duration
    timeout: Duration,
}

impl TwoPhaseCommitNode {
    /// Create a new Two-Phase Commit node
    pub fn new(config: TwoPhaseCommitConfig, role: NodeRole) -> Self {
        let timeout = Duration::from_millis(config.timeout_ms);
        
        let coordinator_state = if matches!(role, NodeRole::Coordinator) {
            Some(CoordinatorState::default())
        } else {
            None
        };
        
        let participant_state = if matches!(role, NodeRole::Participant) {
            Some(ParticipantStateData::default())
        } else {
            None
        };
        
        Self {
            config,
            role,
            coordinator_state,
            participant_state,
            timeout,
        }
    }
    
    /// Begin a new transaction (coordinator role)
    pub fn begin_transaction(&mut self, transaction_id: TransactionId, data: Vec<u8>) -> Result<(), TwoPhaseCommitError> {
        if !matches!(self.role, NodeRole::Coordinator) {
            return Err(TwoPhaseCommitError::NotParticipant);
        }
        
        let state = self.coordinator_state.as_mut().unwrap();
        
        // Check if transaction already exists
        if state.transactions.contains_key(&transaction_id) {
            return Err(TwoPhaseCommitError::InvalidState);
        }
        
        // Initialize transaction state
        state.transactions.insert(transaction_id.clone(), TransactionState::Active);
        state.votes.insert(transaction_id.clone(), HashMap::new());
        state.transaction_data.insert(transaction_id, data);
        
        Ok(())
    }
    
    /// Handle a prepare request (participant role)
    pub fn handle_prepare(&mut self, request: PrepareRequest) -> Result<PrepareResponse, TwoPhaseCommitError> {
        if !matches!(self.role, NodeRole::Participant) {
            return Err(TwoPhaseCommitError::NotParticipant);
        }
        
        let state = self.participant_state.as_mut().unwrap();
        
        // Store transaction data
        state.transaction_data.insert(request.transaction_id.clone(), request.data.clone());
        
        // Simulate processing the transaction
        // In a real implementation, this would involve validating the transaction
        let vote = true; // Assume transaction is valid
        
        // Update participant state
        state.state = if vote {
            ParticipantState::Ready
        } else {
            ParticipantState::Aborted
        };
        
        Ok(PrepareResponse {
            transaction_id: request.transaction_id,
            vote,
            participant_id: self.config.node_id.clone(),
        })
    }
    
    /// Handle a commit request (participant role)
    pub fn handle_commit(&mut self, request: CommitRequest) -> Result<AckResponse, TwoPhaseCommitError> {
        if !matches!(self.role, NodeRole::Participant) {
            return Err(TwoPhaseCommitError::NotParticipant);
        }
        
        let state = self.participant_state.as_mut().unwrap();
        
        // Commit the transaction
        // In a real implementation, this would involve actually committing the transaction
        state.state = ParticipantState::Committed;
        
        // Remove transaction data
        state.transaction_data.remove(&request.transaction_id);
        
        Ok(AckResponse {
            transaction_id: request.transaction_id,
            participant_id: self.config.node_id.clone(),
            success: true,
        })
    }
    
    /// Handle an abort request (participant role)
    pub fn handle_abort(&mut self, request: AbortRequest) -> Result<AckResponse, TwoPhaseCommitError> {
        if !matches!(self.role, NodeRole::Participant) {
            return Err(TwoPhaseCommitError::NotParticipant);
        }
        
        let state = self.participant_state.as_mut().unwrap();
        
        // Abort the transaction
        state.state = ParticipantState::Aborted;
        
        // Remove transaction data
        state.transaction_data.remove(&request.transaction_id);
        
        Ok(AckResponse {
            transaction_id: request.transaction_id,
            participant_id: self.config.node_id.clone(),
            success: true,
        })
    }
    
    /// Execute the Two-Phase Commit protocol (coordinator role)
    pub async fn execute_transaction(&mut self, transaction_id: TransactionId) -> Result<(), TwoPhaseCommitError> {
        if !matches!(self.role, NodeRole::Coordinator) {
            return Err(TwoPhaseCommitError::NotParticipant);
        }
        
        let state = self.coordinator_state.as_mut().unwrap();
        
        // Check if transaction exists
        if !state.transactions.contains_key(&transaction_id) {
            return Err(TwoPhaseCommitError::InvalidState);
        }
        
        // Get transaction data
        let data = state.transaction_data.get(&transaction_id).unwrap().clone();
        
        // Phase 1: Prepare
        let prepare_request = PrepareRequest {
            transaction_id: transaction_id.clone(),
            data: data.clone(),
        };
        
        // In a real implementation, we would send this to all participants and collect responses
        // For this implementation, we'll simulate the process
        
        let mut votes = HashMap::new();
        for (participant_id, _address) in &self.config.participant_addresses {
            // Simulate a prepare response
            let response = PrepareResponse {
                transaction_id: transaction_id.clone(),
                vote: true, // Assume participant votes yes
                participant_id: participant_id.clone(),
            };
            votes.insert(participant_id.clone(), response.vote);
        }
        
        // Store votes
        state.votes.insert(transaction_id.clone(), votes.clone());
        
        // Check if all participants voted yes
        let all_yes = votes.values().all(|&vote| vote);
        
        if all_yes {
            // Phase 2: Commit
            state.transactions.insert(transaction_id.clone(), TransactionState::Prepared);
            
            // In a real implementation, we would send commit requests to all participants
            // For this implementation, we'll simulate the process
            
            for (participant_id, _address) in &self.config.participant_addresses {
                // Simulate successful commit acknowledgment
                // In a real implementation, we would handle failures and retries
            }
            
            // Mark transaction as committed
            state.transactions.insert(transaction_id.clone(), TransactionState::Committed);
            
            // Remove transaction data
            state.transaction_data.remove(&transaction_id);
            
            Ok(())
        } else {
            // Phase 2: Abort
            state.transactions.insert(transaction_id.clone(), TransactionState::Aborted);
            
            // In a real implementation, we would send abort requests to all participants
            // For this implementation, we'll simulate the process
            
            for (participant_id, _address) in &self.config.participant_addresses {
                // Simulate successful abort acknowledgment
            }
            
            // Remove transaction data
            state.transaction_data.remove(&transaction_id);
            
            Err(TwoPhaseCommitError::Aborted)
        }
    }
    
    /// Get transaction state (coordinator role)
    pub fn get_transaction_state(&self, transaction_id: &TransactionId) -> Option<&TransactionState> {
        if let Some(state) = &self.coordinator_state {
            state.transactions.get(transaction_id)
        } else {
            None
        }
    }
    
    /// Get participant state (participant role)
    pub fn get_participant_state(&self) -> Option<&ParticipantState> {
        if let Some(state) = &self.participant_state {
            Some(&state.state)
        } else {
            None
        }
    }
    
    /// Get the current role of the node
    pub fn get_role(&self) -> &NodeRole {
        &self.role
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transaction_id_creation() {
        let tx_id = TransactionId::new("test-transaction".to_string());
        
        assert_eq!(tx_id.id, "test-transaction");
        assert!(tx_id.timestamp > 0);
    }
    
    #[test]
    fn test_two_phase_commit_node_creation() {
        let config = TwoPhaseCommitConfig::default();
        let node = TwoPhaseCommitNode::new(config, NodeRole::Coordinator);
        
        assert_eq!(*node.get_role(), NodeRole::Coordinator);
    }
    
    #[test]
    fn test_begin_transaction() {
        let config = TwoPhaseCommitConfig::default();
        let mut node = TwoPhaseCommitNode::new(config, NodeRole::Coordinator);
        
        let tx_id = TransactionId::new("test-transaction".to_string());
        let data = b"test_data".to_vec();
        
        assert!(node.begin_transaction(tx_id.clone(), data.clone()).is_ok());
        
        // Try to begin the same transaction again
        assert_eq!(node.begin_transaction(tx_id, data), Err(TwoPhaseCommitError::InvalidState));
    }
    
    #[test]
    fn test_prepare_request() {
        let config = TwoPhaseCommitConfig::default();
        let mut node = TwoPhaseCommitNode::new(config, NodeRole::Participant);
        
        let tx_id = TransactionId::new("test-transaction".to_string());
        let request = PrepareRequest {
            transaction_id: tx_id.clone(),
            data: b"test_data".to_vec(),
        };
        
        let response = node.handle_prepare(request).unwrap();
        
        assert_eq!(response.transaction_id.id, "test-transaction");
        assert_eq!(response.vote, true);
        assert_eq!(response.participant_id, "coordinator-0"); // Default node ID
    }
    
    #[test]
    fn test_commit_request() {
        let config = TwoPhaseCommitConfig::default();
        let mut node = TwoPhaseCommitNode::new(config, NodeRole::Participant);
        
        let tx_id = TransactionId::new("test-transaction".to_string());
        let request = CommitRequest {
            transaction_id: tx_id.clone(),
        };
        
        let response = node.handle_commit(request).unwrap();
        
        assert_eq!(response.transaction_id.id, "test-transaction");
        assert_eq!(response.success, true);
        assert_eq!(response.participant_id, "coordinator-0"); // Default node ID
    }
    
    #[test]
    fn test_abort_request() {
        let config = TwoPhaseCommitConfig::default();
        let mut node = TwoPhaseCommitNode::new(config, NodeRole::Participant);
        
        let tx_id = TransactionId::new("test-transaction".to_string());
        let request = AbortRequest {
            transaction_id: tx_id.clone(),
        };
        
        let response = node.handle_abort(request).unwrap();
        
        assert_eq!(response.transaction_id.id, "test-transaction");
        assert_eq!(response.success, true);
        assert_eq!(response.participant_id, "coordinator-0"); // Default node ID
    }
    
    #[test]
    fn test_participant_state() {
        let config = TwoPhaseCommitConfig::default();
        let node = TwoPhaseCommitNode::new(config, NodeRole::Participant);
        
        assert_eq!(*node.get_participant_state().unwrap(), ParticipantState::Working);
    }
    
    #[test]
    fn test_invalid_role() {
        let config = TwoPhaseCommitConfig::default();
        let mut node = TwoPhaseCommitNode::new(config, NodeRole::Participant);
        
        let tx_id = TransactionId::new("test-transaction".to_string());
        
        // Try to begin transaction as participant (should fail)
        assert_eq!(
            node.begin_transaction(tx_id, b"test_data".to_vec()),
            Err(TwoPhaseCommitError::NotParticipant)
        );
    }
}