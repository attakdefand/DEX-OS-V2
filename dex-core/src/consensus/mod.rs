//! Consensus module for DEX-OS
//!
//! This module implements consensus algorithms for service coordination.
//! Implements the Priority 3 feature from DEX-OS-V2.csv:
//! - Infrastructure,Network,Network,Raft Consensus,Service Coordination,Medium

pub mod raft;

pub use raft::{RaftConfig, RaftError, RaftNode};
