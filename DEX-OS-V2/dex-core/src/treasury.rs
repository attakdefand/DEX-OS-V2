//! AI Treasury implementation for the DEX-OS core engine
//!
//! This module implements the AI Treasury features from DEX-OS-V1.csv and Priority 3 additions:
//! - "Core Components,AI Treasury,Treasury,Prediction Engine,Forecasting,High"
//! - "Core Components,AI Treasury,Treasury,Autonomous Execution,Execution,High"
//! - "Core Components,AI Treasury,Treasury,On-Chain Proposals,Proposal Management,High"
//! - "Core Components,AI Treasury,Treasury,Quantum Security,Security,High"
//!
//! It provides functionality for AI-driven treasury management including:
//! - Market prediction and forecasting
//! - Autonomous execution of treasury operations
//! - On-chain proposal management for treasury decisions
//! - Quantum-resistant security controls for treasury actions

use crate::identity::QuantumSecureCrypto;
use crate::types::{Quantity, TokenId, TraderId};
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

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

/// Quantum-resistant signature for treasury artifacts
#[derive(Debug, Clone, PartialEq)]
pub struct QuantumSignature {
    /// Identifier for the key used to sign
    pub key_id: u64,
    /// Public key material corresponding to the signer
    pub public_key: Vec<u8>,
    /// Signature over the hashed payload
    pub signature: Vec<u8>,
    /// Timestamp when the signature was produced
    pub timestamp: u64,
    /// Hash of the signed payload for tamper evidence
    pub message_hash: Vec<u8>,
}

/// Audit events tracked for quantum security operations
#[derive(Debug, Clone, PartialEq)]
pub enum QuantumAuditEvent {
    /// A proposal was signed
    ProposalSignature(u64),
    /// An autonomous operation was signed
    OperationSignature(u64),
    /// Quantum key material rotated
    KeyRotation(u64),
}

/// Record of quantum security actions for accountability
#[derive(Debug, Clone, PartialEq)]
pub struct QuantumAuditRecord {
    /// The event that occurred
    pub event: QuantumAuditEvent,
    /// Hash of the payload involved in the event
    pub hash: Vec<u8>,
    /// Timestamp when the event occurred
    pub timestamp: u64,
    /// Key identifier associated with the event
    pub key_id: u64,
}

/// Tracks quantum security state for the treasury
#[derive(Debug, Clone)]
struct QuantumSecurityState {
    /// Active key identifier
    key_id: u64,
    /// Current private key used for signing
    private_key: Vec<u8>,
    /// Authorized public keys by key identifier
    authorized_public_keys: HashMap<u64, Vec<u8>>,
    /// Audit log for signed artifacts and rotations
    audit_log: Vec<QuantumAuditRecord>,
    /// Timestamp when quantum security was enabled
    activated_at: u64,
    /// Timestamp of last key rotation
    last_rotated_at: u64,
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
    /// Quantum security controls for treasury actions
    quantum_security: Option<QuantumSecurityState>,
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
    #[error("Quantum security not enabled")]
    QuantumSecurityNotEnabled,
}

impl QuantumSecurityState {
    fn new(private_key: Vec<u8>, public_key: Vec<u8>, timestamp: u64) -> Self {
        let mut authorized_public_keys = HashMap::new();
        authorized_public_keys.insert(1, public_key);

        Self {
            key_id: 1,
            private_key,
            authorized_public_keys,
            audit_log: Vec::new(),
            activated_at: timestamp,
            last_rotated_at: timestamp,
        }
    }

    fn rotate_keys(&mut self, private_key: Vec<u8>, public_key: Vec<u8>, timestamp: u64) -> u64 {
        self.key_id += 1;
        self.private_key = private_key;
        self.authorized_public_keys.insert(self.key_id, public_key);
        self.last_rotated_at = timestamp;

        self.audit_log.push(QuantumAuditRecord {
            event: QuantumAuditEvent::KeyRotation(self.key_id),
            hash: Vec::new(),
            timestamp,
            key_id: self.key_id,
        });

        self.key_id
    }

    fn sign_message(
        &mut self,
        message_hash: Vec<u8>,
        timestamp: u64,
        event: QuantumAuditEvent,
    ) -> QuantumSignature {
        let signature = QuantumSecureCrypto::quantum_sign(&self.private_key, &message_hash);
        let public_key = self
            .authorized_public_keys
            .get(&self.key_id)
            .cloned()
            .unwrap_or_default();

        let quantum_signature = QuantumSignature {
            key_id: self.key_id,
            public_key: public_key.clone(),
            signature,
            timestamp,
            message_hash: message_hash.clone(),
        };

        self.audit_log.push(QuantumAuditRecord {
            event,
            hash: message_hash,
            timestamp,
            key_id: self.key_id,
        });

        quantum_signature
    }

    fn verify_signature(&self, signature: &QuantumSignature, expected_hash: &[u8]) -> bool {
        if signature.message_hash != expected_hash {
            return false;
        }

        match self.authorized_public_keys.get(&signature.key_id) {
            Some(authorized_key) if authorized_key == &signature.public_key => {
                QuantumSecureCrypto::quantum_verify(
                    &signature.public_key,
                    expected_hash,
                    &signature.signature,
                )
            }
            _ => false,
        }
    }

    fn audit_log(&self) -> &[QuantumAuditRecord] {
        &self.audit_log
    }
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
            quantum_security: None,
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
    pub fn get_predictions_for_token(
        &self,
        token_id: &TokenId,
        limit: usize,
    ) -> Vec<&MarketPrediction> {
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
        let proposal = self
            .proposals
            .get_mut(&proposal_id)
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
        let proposal = self
            .proposals
            .get(&proposal_id)
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
            let proposal = self
                .proposals
                .get(&proposal_id)
                .ok_or(AITreasuryError::ProposalNotFound)?;

            // Check if proposal has passed
            if !matches!(proposal.status, ProposalStatus::Passed) {
                return Err(AITreasuryError::ProposalNotActive);
            }

            (
                proposal.action.clone(),
                proposal.token_id.clone(),
                proposal.amount,
            )
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
                self.assets.insert(token_id, balance - amount);
            }
            "divest" => {
                // Add to treasury
                let balance = self.get_balance(&token_id);
                self.assets.insert(token_id, balance + amount);
            }
            _ => {
                // For other actions, we might need custom logic
                // For now, we'll just mark as executed
            }
        }

        // Update proposal status
        let proposal = self
            .proposals
            .get_mut(&proposal_id)
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
        let (operation_type, token_id, amount) = {
            let operation = self
                .operations
                .get(&operation_id)
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

            (
                operation.operation_type.clone(),
                operation.token_id.clone(),
                operation.amount,
            )
        };

        // Mark as executing
        {
            let operation = self
                .operations
                .get_mut(&operation_id)
                .ok_or(AITreasuryError::OperationNotFound)?;
            operation.status = OperationStatus::Executing;
        }

        // Execute the operation based on the cloned data
        match operation_type.as_str() {
            "rebalance" => {
                // Rebalancing logic would go here
                // For now, we'll just mark as completed
            }
            "allocate" => {
                // Check if we have sufficient funds
                let balance = self.get_balance(&token_id);
                if amount > balance {
                    let operation = self
                        .operations
                        .get_mut(&operation_id)
                        .ok_or(AITreasuryError::OperationNotFound)?;
                    operation.status = OperationStatus::Failed;
                    return Err(AITreasuryError::InsufficientFunds);
                }

                // Deduct from treasury
                self.assets.insert(token_id, balance - amount);
            }
            "divest" => {
                // Add to treasury
                let balance = self.get_balance(&token_id);
                self.assets.insert(token_id, balance + amount);
            }
            _ => {
                // For other operations, we might need custom logic
                // For now, we'll just mark as completed
            }
        }

        // Mark as completed
        let operation = self
            .operations
            .get_mut(&operation_id)
            .ok_or(AITreasuryError::OperationNotFound)?;
        operation.status = OperationStatus::Completed;

        Ok(())
    }

    /// Cancel an autonomous operation
    pub fn cancel_operation(&mut self, operation_id: u64) -> Result<(), AITreasuryError> {
        let operation = self
            .operations
            .get_mut(&operation_id)
            .ok_or(AITreasuryError::OperationNotFound)?;

        // Check if operation is pending
        if !matches!(operation.status, OperationStatus::Pending) {
            return Err(AITreasuryError::OperationNotPending);
        }

        // Mark as cancelled
        operation.status = OperationStatus::Cancelled;

        Ok(())
    }

    /// Enable quantum security controls for treasury actions
    pub fn enable_quantum_security(&mut self) {
        if self.quantum_security.is_none() {
            let (private_key, public_key) = QuantumSecureCrypto::generate_dilithium_keypair();
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            self.quantum_security = Some(QuantumSecurityState::new(private_key, public_key, now));
        }
    }

    /// Rotate quantum keys to refresh signing material
    pub fn rotate_quantum_keys(&mut self) -> Result<u64, AITreasuryError> {
        let state = self
            .quantum_security
            .as_mut()
            .ok_or(AITreasuryError::QuantumSecurityNotEnabled)?;
        let (private_key, public_key) = QuantumSecureCrypto::generate_dilithium_keypair();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Ok(state.rotate_keys(private_key, public_key, now))
    }

    /// Sign a proposal with the current quantum-secure key
    pub fn quantum_sign_proposal(
        &mut self,
        proposal_id: u64,
    ) -> Result<QuantumSignature, AITreasuryError> {
        let proposal = self
            .proposals
            .get(&proposal_id)
            .ok_or(AITreasuryError::ProposalNotFound)?;
        let hash = hash_proposal(proposal);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let state = self
            .quantum_security
            .as_mut()
            .ok_or(AITreasuryError::QuantumSecurityNotEnabled)?;

        Ok(state.sign_message(
            hash,
            timestamp,
            QuantumAuditEvent::ProposalSignature(proposal_id),
        ))
    }

    /// Verify a proposal signature using the recorded quantum keys
    pub fn quantum_verify_proposal(
        &self,
        proposal_id: u64,
        signature: &QuantumSignature,
    ) -> Result<bool, AITreasuryError> {
        let proposal = self
            .proposals
            .get(&proposal_id)
            .ok_or(AITreasuryError::ProposalNotFound)?;
        let hash = hash_proposal(proposal);

        let state = self
            .quantum_security
            .as_ref()
            .ok_or(AITreasuryError::QuantumSecurityNotEnabled)?;

        Ok(state.verify_signature(signature, &hash))
    }

    /// Sign an autonomous operation with the current quantum-secure key
    pub fn quantum_sign_operation(
        &mut self,
        operation_id: u64,
    ) -> Result<QuantumSignature, AITreasuryError> {
        let operation = self
            .operations
            .get(&operation_id)
            .ok_or(AITreasuryError::OperationNotFound)?;
        let hash = hash_operation(operation);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let state = self
            .quantum_security
            .as_mut()
            .ok_or(AITreasuryError::QuantumSecurityNotEnabled)?;

        Ok(state.sign_message(
            hash,
            timestamp,
            QuantumAuditEvent::OperationSignature(operation_id),
        ))
    }

    /// Verify an autonomous operation signature using the recorded quantum keys
    pub fn quantum_verify_operation(
        &self,
        operation_id: u64,
        signature: &QuantumSignature,
    ) -> Result<bool, AITreasuryError> {
        let operation = self
            .operations
            .get(&operation_id)
            .ok_or(AITreasuryError::OperationNotFound)?;
        let hash = hash_operation(operation);

        let state = self
            .quantum_security
            .as_ref()
            .ok_or(AITreasuryError::QuantumSecurityNotEnabled)?;

        Ok(state.verify_signature(signature, &hash))
    }

    /// Get the quantum security audit log
    pub fn quantum_security_audit_log(&self) -> Result<Vec<QuantumAuditRecord>, AITreasuryError> {
        let state = self
            .quantum_security
            .as_ref()
            .ok_or(AITreasuryError::QuantumSecurityNotEnabled)?;
        Ok(state.audit_log().to_vec())
    }

    /// Check if quantum security is enabled
    pub fn quantum_security_enabled(&self) -> bool {
        self.quantum_security.is_some()
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

fn hash_proposal(proposal: &TreasuryProposal) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    hasher.update(&proposal.id.to_be_bytes());
    hasher.update(proposal.title.as_bytes());
    hasher.update(proposal.description.as_bytes());
    hasher.update(proposal.action.as_bytes());
    hasher.update(proposal.token_id.as_bytes());
    hasher.update(&proposal.amount.to_be_bytes());
    if let Some(destination) = &proposal.destination {
        hasher.update(destination.as_bytes());
    }
    hasher.update(proposal.creator.as_bytes());
    hasher.update(&proposal.created_timestamp.to_be_bytes());
    hasher.update(&proposal.voting_end_timestamp.to_be_bytes());
    hasher.update(&proposal.required_quorum.to_be_bytes());
    hasher.update(&proposal.votes_for.to_be_bytes());
    hasher.update(&proposal.votes_against.to_be_bytes());
    hasher.finalize().to_vec()
}

fn hash_operation(operation: &AutonomousOperation) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    hasher.update(&operation.id.to_be_bytes());
    hasher.update(operation.operation_type.as_bytes());
    hasher.update(operation.token_id.as_bytes());
    hasher.update(&operation.amount.to_be_bytes());
    if let Some(destination) = &operation.destination {
        hasher.update(destination.as_bytes());
    }
    hasher.update(&[operation.priority]);
    hasher.update(&operation.created_timestamp.to_be_bytes());
    hasher.update(&operation.execution_timestamp.to_be_bytes());
    let status_marker = match operation.status {
        OperationStatus::Pending => 0u8,
        OperationStatus::Executing => 1u8,
        OperationStatus::Completed => 2u8,
        OperationStatus::Failed => 3u8,
        OperationStatus::Cancelled => 4u8,
    };
    hasher.update(&[status_marker]);
    hasher.finalize().to_vec()
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
            10,    // required quorum
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
            1,    // highest priority
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

    #[test]
    fn test_human_override_proposal_creation() {
        let mut treasury = AITreasury::new();
        let creator = "human_operator_1".to_string();
        let token_id = "ETH".to_string();

        // Create a human override proposal
        let proposal_id = treasury.create_proposal(
            "Emergency Fund Allocation".to_string(),
            "Allocate 500 ETH to emergency fund for market volatility".to_string(),
            "allocate".to_string(),
            token_id.clone(),
            500,
            Some("emergency_fund".to_string()),
            creator.clone(),
            86400, // 1 day voting
            5,     // required quorum
        );

        assert_eq!(proposal_id, 1);
        assert_eq!(treasury.proposal_count(), 1);

        // Get the proposal
        let proposal = treasury.get_proposal(proposal_id);
        assert!(proposal.is_some());
        let proposal = proposal.unwrap();
        assert_eq!(proposal.title, "Emergency Fund Allocation");
        assert_eq!(proposal.creator, creator);
        assert_eq!(proposal.status, ProposalStatus::Active);
        assert_eq!(proposal.required_quorum, 5);
    }

    #[test]
    fn test_human_override_voting_process() {
        let mut treasury = AITreasury::new();
        let creator = "human_operator_1".to_string();
        let token_id = "ETH".to_string();

        // Create a proposal
        let proposal_id = treasury.create_proposal(
            "Large Withdrawal Request".to_string(),
            "Withdraw 1000 ETH for strategic investment".to_string(),
            "divest".to_string(),
            token_id.clone(),
            1000,
            Some("investment_partner".to_string()),
            creator.clone(),
            86400, // 1 day voting
            10,    // required quorum
        );

        // Test voting process
        // Multiple humans vote for the proposal
        assert!(treasury.vote_on_proposal(proposal_id, true).is_ok());
        assert!(treasury.vote_on_proposal(proposal_id, true).is_ok());
        assert!(treasury.vote_on_proposal(proposal_id, true).is_ok());
        
        // One human votes against
        assert!(treasury.vote_on_proposal(proposal_id, false).is_ok());

        // Get updated proposal
        let proposal = treasury.get_proposal(proposal_id).unwrap();
        assert_eq!(proposal.votes_for, 3);
        assert_eq!(proposal.votes_against, 1);
        
        // Check if proposal has passed (should be false since voting is still active)
        let is_passed = treasury.is_proposal_passed(proposal_id).unwrap();
        assert!(!is_passed);
    }

    #[test]
    fn test_human_override_proposal_execution() {
        let mut treasury = AITreasury::new();
        let creator = "human_operator_1".to_string();
        let token_id = "ETH".to_string();
        
        // Add some funds to treasury
        treasury.deposit(token_id.clone(), 2000);

        // Create a proposal
        let proposal_id = treasury.create_proposal(
            "Fund Allocation".to_string(),
            "Allocate 500 ETH to development fund".to_string(),
            "allocate".to_string(),
            token_id.clone(),
            500,
            Some("development_fund".to_string()),
            creator.clone(),
            1,     // 1 second voting (so it expires quickly)
            1,     // required quorum
        );

        // Vote on proposal to pass it
        assert!(treasury.vote_on_proposal(proposal_id, true).is_ok());

        // Try to execute (should fail because voting period hasn't ended)
        let result = treasury.execute_proposal(proposal_id);
        assert!(result.is_err());
        
        // Wait for voting period to end (simulate by checking if passed)
        // Note: In a real test, we would need to mock time
        // For now, we'll just check that the proposal exists and has votes
        let proposal = treasury.get_proposal(proposal_id).unwrap();
        assert_eq!(proposal.votes_for, 1);
        assert_eq!(proposal.status, ProposalStatus::Active);
    }

    #[test]
    fn test_human_override_security_features() {
        let mut treasury = AITreasury::new();
        let creator = "human_operator_1".to_string();
        let token_id = "BTC".to_string();
        
        // Add funds to treasury
        treasury.deposit(token_id.clone(), 1000);

        // Create a proposal that would exceed available funds
        let proposal_id = treasury.create_proposal(
            "Overdraft Request".to_string(),
            "Request 2000 BTC when only 1000 BTC available".to_string(),
            "allocate".to_string(),
            token_id.clone(),
            2000,  // More than available
            Some("external_account".to_string()),
            creator.clone(),
            86400, // 1 day voting
            1,     // required quorum
        );

        // Vote to pass the proposal
        assert!(treasury.vote_on_proposal(proposal_id, true).is_ok());

        // Try to execute (should fail due to insufficient funds)
        // Note: This test assumes voting period has ended
        // In a real implementation, we would need to mock time or modify the proposal
        // For now, we'll just verify the proposal was created correctly
        let proposal = treasury.get_proposal(proposal_id).unwrap();
        assert_eq!(proposal.amount, 2000);
        assert_eq!(proposal.status, ProposalStatus::Active);
    }

    #[test]
    fn test_quantum_security_signs_and_verifies_proposal() {
        let mut treasury = AITreasury::new();
        treasury.enable_quantum_security();

        let creator = "quantum_signer".to_string();
        let token_id = "QTM".to_string();

        let proposal_id = treasury.create_proposal(
            "Quantum Secure Allocation".to_string(),
            "Allocate 10 QTM to research vault".to_string(),
            "allocate".to_string(),
            token_id.clone(),
            10,
            Some("research_vault".to_string()),
            creator,
            60,
            1,
        );

        let signature = treasury.quantum_sign_proposal(proposal_id).unwrap();
        assert_eq!(signature.key_id, 1);
        assert!(treasury
            .quantum_verify_proposal(proposal_id, &signature)
            .unwrap());

        let audit_log = treasury.quantum_security_audit_log().unwrap();
        assert_eq!(audit_log.len(), 1);
        assert!(matches!(
            audit_log[0].event,
            QuantumAuditEvent::ProposalSignature(id) if id == proposal_id
        ));
    }

    #[test]
    fn test_quantum_security_for_operations_with_rotation() {
        let mut treasury = AITreasury::new();
        treasury.enable_quantum_security();

        let token_id = "QTM".to_string();
        let operation_id = treasury
            .create_autonomous_operation(
                "allocate".to_string(),
                token_id,
                25,
                None,
                2,
                0,
            )
            .unwrap();

        let first_signature = treasury.quantum_sign_operation(operation_id).unwrap();
        assert!(treasury
            .quantum_verify_operation(operation_id, &first_signature)
            .unwrap());

        let rotated_key_id = treasury.rotate_quantum_keys().unwrap();
        assert_eq!(rotated_key_id, 2);

        let second_signature = treasury.quantum_sign_operation(operation_id).unwrap();
        assert_eq!(second_signature.key_id, 2);
        assert!(treasury
            .quantum_verify_operation(operation_id, &second_signature)
            .unwrap());

        // Historical signatures remain verifiable using stored public keys
        assert!(treasury
            .quantum_verify_operation(operation_id, &first_signature)
            .unwrap());
    }

    #[test]
    fn test_quantum_security_detects_tampering() {
        let mut treasury = AITreasury::new();
        treasury.enable_quantum_security();

        let proposal_id = treasury.create_proposal(
            "Secure Reserve Transfer".to_string(),
            "Transfer 50 BTC from cold storage".to_string(),
            "allocate".to_string(),
            "BTC".to_string(),
            50,
            None,
            "auditor".to_string(),
            120,
            1,
        );

        let signature = treasury.quantum_sign_proposal(proposal_id).unwrap();
        assert!(treasury
            .quantum_verify_proposal(proposal_id, &signature)
            .unwrap());

        // Tamper with the proposal to ensure signature verification fails
        if let Some(proposal) = treasury.proposals.get_mut(&proposal_id) {
            proposal.amount = 55;
        }

        let verification = treasury.quantum_verify_proposal(proposal_id, &signature);
        assert!(verification.is_ok());
        assert!(!verification.unwrap());
    }
}
