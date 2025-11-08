//! Snapshot mechanism for off-chain voting
//!
//! This module implements the Priority 3 feature from DEX-OS-V2.csv:
//! - Core Trading,Governance,Governance,Snapshot Mechanism,Off-chain Voting,Medium

use crate::governance::{Proposal, ProposalStatus};
use crate::types::TraderId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Snapshot manager for off-chain voting
#[derive(Debug, Clone)]
pub struct SnapshotManager {
    /// Stored snapshots
    snapshots: HashMap<String, VotingSnapshot>,
    /// Index for quick lookup by proposal
    proposal_index: HashMap<String, String>,
}

/// A snapshot of voting state for off-chain voting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingSnapshot {
    /// Unique identifier for this snapshot
    pub id: String,
    /// Proposal this snapshot is for
    pub proposal_id: String,
    /// Timestamp when the snapshot was taken
    pub taken_at: u64,
    /// Voting power distribution at the time of snapshot
    pub voting_power: HashMap<TraderId, u64>,
    /// Total voting power
    pub total_voting_power: u64,
    /// Metadata about the snapshot
    pub metadata: SnapshotMetadata,
}

/// Metadata about a snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    /// Block number or state version
    pub block_number: u64,
    /// Network identifier
    pub network: String,
    /// Additional custom metadata
    pub custom: HashMap<String, String>,
}

impl SnapshotManager {
    /// Create a new snapshot manager
    pub fn new() -> Self {
        Self {
            snapshots: HashMap::new(),
            proposal_index: HashMap::new(),
        }
    }

    /// Take a snapshot of voting power for a proposal
    pub fn take_snapshot(
        &mut self,
        proposal_id: String,
        voting_power: HashMap<TraderId, u64>,
        metadata: SnapshotMetadata,
    ) -> Result<String, SnapshotError> {
        let total_voting_power = voting_power.values().sum();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let snapshot_id = format!("snapshot_{}_{}", proposal_id, now);

        let snapshot = VotingSnapshot {
            id: snapshot_id.clone(),
            proposal_id: proposal_id.clone(),
            taken_at: now,
            voting_power,
            total_voting_power,
            metadata,
        };

        // Store the snapshot
        self.snapshots.insert(snapshot_id.clone(), snapshot);

        // Update index
        self.proposal_index.insert(proposal_id, snapshot_id.clone());

        Ok(snapshot_id)
    }

    /// Get a snapshot by ID
    pub fn get_snapshot(&self, id: &str) -> Option<&VotingSnapshot> {
        self.snapshots.get(id)
    }

    /// Get the latest snapshot for a proposal
    pub fn get_latest_snapshot_for_proposal(&self, proposal_id: &str) -> Option<&VotingSnapshot> {
        if let Some(snapshot_id) = self.proposal_index.get(proposal_id) {
            self.snapshots.get(snapshot_id)
        } else {
            None
        }
    }

    /// Calculate voting weight for a voter in a snapshot
    pub fn calculate_voting_weight(
        &self,
        snapshot_id: &str,
        voter_id: &TraderId,
    ) -> Result<f64, SnapshotError> {
        if let Some(snapshot) = self.snapshots.get(snapshot_id) {
            if let Some(voting_power) = snapshot.voting_power.get(voter_id) {
                Ok(*voting_power as f64 / snapshot.total_voting_power as f64)
            } else {
                Ok(0.0)
            }
        } else {
            Err(SnapshotError::SnapshotNotFound)
        }
    }

    /// Validate that a snapshot is still valid for a proposal
    pub fn validate_snapshot(
        &self,
        snapshot_id: &str,
        proposal: &Proposal,
    ) -> Result<bool, SnapshotError> {
        if let Some(snapshot) = self.snapshots.get(snapshot_id) {
            // Check that the snapshot is for the correct proposal
            if snapshot.proposal_id != proposal.id {
                return Ok(false);
            }

            // Check that the proposal is still in voting phase
            if proposal.status != ProposalStatus::Active {
                return Ok(false);
            }

            // Check that the snapshot was taken before voting started
            if snapshot.taken_at > proposal.voting_start {
                return Ok(false);
            }

            Ok(true)
        } else {
            Err(SnapshotError::SnapshotNotFound)
        }
    }
}

impl Default for SnapshotManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during snapshot operations
#[derive(Debug, Error)]
pub enum SnapshotError {
    #[error("Snapshot not found")]
    SnapshotNotFound,
    #[error("Invalid snapshot for proposal")]
    InvalidSnapshot,
}
