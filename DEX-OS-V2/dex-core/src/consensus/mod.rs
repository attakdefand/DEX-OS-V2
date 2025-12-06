//! Consensus module for DEX-OS
//!
//! This module implements consensus algorithms for service coordination.
//! Implements the Priority 3 feature from DEX-OS-V2.csv:
//! - Infrastructure,Network,Network,Raft Consensus,Service Coordination,Medium

pub mod raft;
pub mod paxos;
pub mod two_phase_commit;

pub use raft::{RaftConfig, RaftError, RaftNode};
pub use paxos::{PaxosConfig, PaxosError, PaxosNode, PaxosValue, ProposalNumber};
pub use two_phase_commit::{TwoPhaseCommitConfig, TwoPhaseCommitError, TwoPhaseCommitNode, TransactionId};