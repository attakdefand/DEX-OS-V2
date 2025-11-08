//! Cryptographic modules for DEX-OS
//!
//! This module implements various cryptographic features for the DEX-OS core engine.

pub mod zk_proof;

pub use zk_proof::{PrivacyProtectionService, ZkProof, ZkProofSystem};