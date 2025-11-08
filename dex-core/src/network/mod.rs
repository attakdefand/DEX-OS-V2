//! Network module for DEX-OS
//!
//! This module implements network protocols for node discovery and communication.
//! Implements the Priority 3 feature from DEX-OS-V2.csv:
//! - Infrastructure,Network,Network,Gossip Protocol,Node Discovery,Medium

pub mod gossip;

pub use gossip::{GossipConfig, GossipError, GossipNode};
