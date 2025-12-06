//! Paxos consensus implementation for distributed agreement
//!
//! This module implements the Paxos consensus algorithm for coordinating services
//! in the DEX-OS infrastructure, covering the Priority 3 distributed-systems feature:
//! - Distributed Systems,Distributed Systems,Distributed Systems,Consensus,Paxos Algorithm,Medium

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Errors that can occur in the Paxos consensus algorithm
#[derive(Error, Debug, PartialEq)]
pub enum PaxosError {
    /// Node is not a participant in the consensus
    #[error("Node is not a participant in the consensus")]
    NotParticipant,
    
    /// Proposal number is not valid
    #[error("Invalid proposal number")]
    InvalidProposalNumber,
    
    /// Node is not the leader
    #[error("Node is not the leader")]
    NotLeader,
    
    /// Timeout occurred during consensus
    #[error("Timeout occurred during consensus")]
    Timeout,
}

/// Proposal number for Paxos consensus
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProposalNumber {
    /// Round number
    pub round: u64,
    /// Node ID (tie breaker)
    pub node_id: u64,
}

impl ProposalNumber {
    /// Create a new proposal number
    pub fn new(round: u64, node_id: u64) -> Self {
        Self { round, node_id }
    }
}

/// Value to be agreed upon in Paxos
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaxosValue {
    /// The actual value
    pub data: Vec<u8>,
    /// Timestamp when the value was proposed
    pub timestamp: u64,
}

impl PaxosValue {
    /// Create a new Paxos value
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }
}

/// Paxos node roles
#[derive(Debug, Clone, PartialEq)]
pub enum NodeRole {
    /// Proposer node
    Proposer,
    /// Acceptor node
    Acceptor,
    /// Learner node
    Learner,
}

/// Paxos node configuration
#[derive(Debug, Clone)]
pub struct PaxosConfig {
    /// Node ID
    pub node_id: u64,
    /// List of all acceptor node IDs
    pub acceptor_ids: Vec<u64>,
    /// List of all proposer node IDs
    pub proposer_ids: Vec<u64>,
    /// List of all learner node IDs
    pub learner_ids: Vec<u64>,
    /// Timeout for consensus operations in milliseconds
    pub timeout_ms: u64,
    /// Majority threshold (quorum size)
    pub quorum_size: usize,
}

impl Default for PaxosConfig {
    fn default() -> Self {
        Self {
            node_id: 0,
            acceptor_ids: vec![0, 1, 2],
            proposer_ids: vec![0, 1, 2],
            learner_ids: vec![0, 1, 2],
            timeout_ms: 5000,
            quorum_size: 2,
        }
    }
}

/// Prepare request from proposer to acceptors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrepareRequest {
    /// Proposal number
    pub proposal_number: ProposalNumber,
    /// Node ID of the proposer
    pub proposer_id: u64,
}

/// Promise response from acceptor to proposer
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromiseResponse {
    /// Proposal number of the promise
    pub proposal_number: ProposalNumber,
    /// Previously accepted proposal number (if any)
    pub prev_proposal_number: Option<ProposalNumber>,
    /// Previously accepted value (if any)
    pub prev_value: Option<PaxosValue>,
    /// Node ID of the acceptor
    pub acceptor_id: u64,
}

/// Accept request from proposer to acceptors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptRequest {
    /// Proposal number
    pub proposal_number: ProposalNumber,
    /// Value to be accepted
    pub value: PaxosValue,
    /// Node ID of the proposer
    pub proposer_id: u64,
}

/// Accepted response from acceptor to proposer
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AcceptedResponse {
    /// Proposal number
    pub proposal_number: ProposalNumber,
    /// Node ID of the acceptor
    pub acceptor_id: u64,
}

/// Learn notification from proposer to learners
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnNotification {
    /// Proposal number
    pub proposal_number: ProposalNumber,
    /// Value that was agreed upon
    pub value: PaxosValue,
    /// Node ID of the proposer
    pub proposer_id: u64,
}

/// Acceptor state
#[derive(Debug, Clone)]
pub struct AcceptorState {
    /// Highest proposal number for which the acceptor has responded to a prepare request
    pub promised_proposal: Option<ProposalNumber>,
    /// Highest proposal number that the acceptor has accepted
    pub accepted_proposal: Option<ProposalNumber>,
    /// Value corresponding to the accepted proposal
    pub accepted_value: Option<PaxosValue>,
}

impl Default for AcceptorState {
    fn default() -> Self {
        Self {
            promised_proposal: None,
            accepted_proposal: None,
            accepted_value: None,
        }
    }
}

/// Proposer state
#[derive(Debug, Clone)]
pub struct ProposerState {
    /// Current proposal number
    pub current_proposal: ProposalNumber,
    /// Number of promises received
    pub promises_received: usize,
    /// Number of acceptances received
    pub acceptances_received: usize,
    /// Values received in promises
    pub promised_values: Vec<(ProposalNumber, PaxosValue)>,
}

impl Default for ProposerState {
    fn default() -> Self {
        Self {
            current_proposal: ProposalNumber::new(1, 0),
            promises_received: 0,
            acceptances_received: 0,
            promised_values: Vec::new(),
        }
    }
}

/// Learner state
#[derive(Debug, Clone)]
pub struct LearnerState {
    /// Learned values mapped by proposal number
    pub learned_values: HashMap<ProposalNumber, PaxosValue>,
    /// Decided value (if consensus has been reached)
    pub decided_value: Option<PaxosValue>,
}

impl Default for LearnerState {
    fn default() -> Self {
        Self {
            learned_values: HashMap::new(),
            decided_value: None,
        }
    }
}

/// Paxos node implementation
pub struct PaxosNode {
    /// Node configuration
    config: PaxosConfig,
    /// Node role
    role: NodeRole,
    /// Acceptor state (if node is an acceptor)
    acceptor_state: Option<AcceptorState>,
    /// Proposer state (if node is a proposer)
    proposer_state: Option<ProposerState>,
    /// Learner state (if node is a learner)
    learner_state: Option<LearnerState>,
    /// Timeout duration
    timeout: Duration,
}

impl PaxosNode {
    /// Create a new Paxos node
    pub fn new(config: PaxosConfig, role: NodeRole) -> Self {
        let timeout = Duration::from_millis(config.timeout_ms);
        
        let acceptor_state = if matches!(role, NodeRole::Acceptor) {
            Some(AcceptorState::default())
        } else {
            None
        };
        
        let proposer_state = if matches!(role, NodeRole::Proposer) {
            Some(ProposerState::default())
        } else {
            None
        };
        
        let learner_state = if matches!(role, NodeRole::Learner) {
            Some(LearnerState::default())
        } else {
            None
        };
        
        Self {
            config,
            role,
            acceptor_state,
            proposer_state,
            learner_state,
            timeout,
        }
    }
    
    /// Handle a prepare request (acceptor role)
    pub fn handle_prepare(&mut self, request: PrepareRequest) -> Result<PromiseResponse, PaxosError> {
        if !matches!(self.role, NodeRole::Acceptor) {
            return Err(PaxosError::NotParticipant);
        }
        
        let state = self.acceptor_state.as_mut().unwrap();
        
        // Check if the proposal number is higher than any we've seen
        if state.promised_proposal.map_or(true, |p| request.proposal_number > p) {
            // Update the promised proposal number
            state.promised_proposal = Some(request.proposal_number);
            
            // Return a promise with the previously accepted value (if any)
            Ok(PromiseResponse {
                proposal_number: request.proposal_number,
                prev_proposal_number: state.accepted_proposal,
                prev_value: state.accepted_value.clone(),
                acceptor_id: self.config.node_id,
            })
        } else {
            // Reject the prepare request
            Err(PaxosError::InvalidProposalNumber)
        }
    }
    
    /// Handle an accept request (acceptor role)
    pub fn handle_accept(&mut self, request: AcceptRequest) -> Result<AcceptedResponse, PaxosError> {
        if !matches!(self.role, NodeRole::Acceptor) {
            return Err(PaxosError::NotParticipant);
        }
        
        let state = self.acceptor_state.as_mut().unwrap();
        
        // Check if we've promised this or a higher proposal number
        if state.promised_proposal.map_or(true, |p| request.proposal_number >= p) {
            // Accept the proposal
            state.accepted_proposal = Some(request.proposal_number);
            state.accepted_value = Some(request.value.clone());
            state.promised_proposal = Some(request.proposal_number);
            
            Ok(AcceptedResponse {
                proposal_number: request.proposal_number,
                acceptor_id: self.config.node_id,
            })
        } else {
            // Reject the accept request
            Err(PaxosError::InvalidProposalNumber)
        }
    }
    
    /// Handle a learn notification (learner role)
    pub fn handle_learn(&mut self, notification: LearnNotification) -> Result<(), PaxosError> {
        if !matches!(self.role, NodeRole::Learner) {
            return Err(PaxosError::NotParticipant);
        }
        
        let state = self.learner_state.as_mut().unwrap();
        
        // Store the learned value
        state.learned_values.insert(notification.proposal_number, notification.value.clone());
        
        // Check if we have enough learnings to decide (simplified)
        if state.decided_value.is_none() && !state.learned_values.is_empty() {
            // In a real implementation, we would check for a quorum of matching values
            // For simplicity, we'll just take the first learned value
            state.decided_value = Some(notification.value);
        }
        
        Ok(())
    }
    
    /// Get the decided value (learner role)
    pub fn get_decided_value(&self) -> Option<PaxosValue> {
        if let Some(state) = &self.learner_state {
            state.decided_value.clone()
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
    fn test_proposal_number_ordering() {
        let p1 = ProposalNumber::new(1, 1);
        let p2 = ProposalNumber::new(1, 2);
        let p3 = ProposalNumber::new(2, 1);
        
        assert!(p1 < p2);
        assert!(p1 < p3);
        assert!(p2 < p3);
    }
    
    #[test]
    fn test_paxos_node_creation() {
        let config = PaxosConfig::default();
        let node = PaxosNode::new(config, NodeRole::Proposer);
        
        assert_eq!(*node.get_role(), NodeRole::Proposer);
    }
    
    #[test]
    fn test_prepare_request() {
        let config = PaxosConfig::default();
        let mut node = PaxosNode::new(config, NodeRole::Acceptor);
        
        let request = PrepareRequest {
            proposal_number: ProposalNumber::new(1, 1),
            proposer_id: 1,
        };
        
        let response = node.handle_prepare(request).unwrap();
        
        assert_eq!(response.proposal_number.round, 1);
        assert_eq!(response.acceptor_id, 0); // Default node ID
    }
    
    #[test]
    fn test_accept_request() {
        let config = PaxosConfig::default();
        let mut node = PaxosNode::new(config, NodeRole::Acceptor);
        
        let value = PaxosValue::new(b"test_value".to_vec());
        let request = AcceptRequest {
            proposal_number: ProposalNumber::new(1, 1),
            value: value.clone(),
            proposer_id: 1,
        };
        
        let response = node.handle_accept(request).unwrap();
        
        assert_eq!(response.proposal_number.round, 1);
        assert_eq!(response.acceptor_id, 0); // Default node ID
    }
    
    #[test]
    fn test_learn_notification() {
        let config = PaxosConfig::default();
        let mut node = PaxosNode::new(config, NodeRole::Learner);
        
        let value = PaxosValue::new(b"test_value".to_vec());
        let notification = LearnNotification {
            proposal_number: ProposalNumber::new(1, 1),
            value: value.clone(),
            proposer_id: 1,
        };
        
        assert!(node.handle_learn(notification).is_ok());
        assert_eq!(node.get_decided_value().unwrap().data, b"test_value");
    }
    
    #[test]
    fn test_prepare_rejection() {
        let config = PaxosConfig::default();
        let mut node = PaxosNode::new(config, NodeRole::Acceptor);
        
        // First prepare request
        let request1 = PrepareRequest {
            proposal_number: ProposalNumber::new(1, 1),
            proposer_id: 1,
        };
        
        assert!(node.handle_prepare(request1).is_ok());
        
        // Second prepare request with lower proposal number
        let request2 = PrepareRequest {
            proposal_number: ProposalNumber::new(0, 1),
            proposer_id: 1,
        };
        
        assert_eq!(node.handle_prepare(request2), Err(PaxosError::InvalidProposalNumber));
    }
    
    #[test]
    fn test_accept_rejection() {
        let config = PaxosConfig::default();
        let mut node = PaxosNode::new(config, NodeRole::Acceptor);
        
        // First prepare request to set the promised proposal
        let prepare_request = PrepareRequest {
            proposal_number: ProposalNumber::new(2, 1),
            proposer_id: 1,
        };
        
        assert!(node.handle_prepare(prepare_request).is_ok());
        
        // Accept request with lower proposal number
        let value = PaxosValue::new(b"test_value".to_vec());
        let accept_request = AcceptRequest {
            proposal_number: ProposalNumber::new(1, 1),
            value: value.clone(),
            proposer_id: 1,
        };
        
        assert_eq!(node.handle_accept(accept_request), Err(PaxosError::InvalidProposalNumber));
    }
}
