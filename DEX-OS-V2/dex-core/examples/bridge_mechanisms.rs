//! Example demonstrating the bridge mechanisms in DEX-OS
//!
//! This example shows how to use the different bridge mechanisms:
//! 1. Standard Universal Bridge
//! 2. Federated Peg
//! 3. MPC Threshold

use dex_core::federated_peg::{FederatedPegManager, FederatedPegConfig, Signer};
use dex_core::mpc_threshold::{MpcThresholdManager, MpcThresholdConfig, MpcParticipant};
use dex_core::universal_bridge::{UniversalBridgeManager, BridgeConfig, BlockchainNetwork};
use std::collections::HashSet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== DEX-OS Bridge Mechanisms Demo ===\n");

    // 1. Standard Universal Bridge Example
    println!("1. Standard Universal Bridge Example");
    standard_bridge_example()?;

    // 2. Federated Peg Example
    println!("\n2. Federated Peg Example");
    federated_peg_example()?;

    // 3. MPC Threshold Example
    println!("\n3. MPC Threshold Example");
    mpc_threshold_example()?;

    println!("\n=== All examples completed successfully ===");
    Ok(())
}

fn standard_bridge_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create bridge manager with default configuration
    let mut bridge_manager = UniversalBridgeManager::new(BridgeConfig::default());

    // Add blockchain networks
    let ethereum = BlockchainNetwork {
        id: "ethereum".to_string(),
        name: "Ethereum".to_string(),
        chain_type: "EVM".to_string(),
        rpc_endpoint: "https://ethereum.rpc".to_string(),
        chain_id: 1,
        native_token: "ETH".to_string(),
        features: HashSet::new(),
        metrics: Default::default(),
    };

    let polygon = BlockchainNetwork {
        id: "polygon".to_string(),
        name: "Polygon".to_string(),
        chain_type: "EVM".to_string(),
        rpc_endpoint: "https://polygon.rpc".to_string(),
        chain_id: 137,
        native_token: "MATIC".to_string(),
        features: HashSet::new(),
        metrics: Default::default(),
    };

    bridge_manager.add_network(ethereum)?;
    bridge_manager.add_network(polygon)?;

    // Initiate a bridge transaction
    bridge_manager.initiate_bridge_transaction(
        "bridge_001".to_string(),
        "ethereum".to_string(),
        "polygon".to_string(),
        "0xSenderAddress".to_string(),
        "0xReceiverAddress".to_string(),
        "ETH".to_string(),
        1000,
    )?;

    println!("  - Initiated standard bridge transaction: bridge_001");
    println!("  - Transferring 1000 ETH from Ethereum to Polygon");

    // Activate and process the transaction
    bridge_manager.activate_bridge_transaction("bridge_001")?;
    bridge_manager.process_bridge_transaction("bridge_001")?;

    println!("  - Bridge transaction activated and processing");

    // Complete the transaction
    bridge_manager.complete_bridge_transaction(
        "bridge_001",
        Some("0xSourceTxHash".to_string()),
        Some("0xDestTxHash".to_string()),
    )?;

    println!("  - Bridge transaction completed successfully");
    Ok(())
}

fn federated_peg_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create federated peg manager with custom configuration
    let mut peg_manager = FederatedPegManager::new(FederatedPegConfig {
        min_signatures: 3,
        timeout_secs: 3600,
        max_concurrent_operations: 100,
    });

    // Add signers to the federation
    let signer1 = Signer {
        id: "signer_1".to_string(),
        public_key: "pubkey_1".to_string(),
        weight: 5,
        last_activity: 0,
    };

    let signer2 = Signer {
        id: "signer_2".to_string(),
        public_key: "pubkey_2".to_string(),
        weight: 5,
        last_activity: 0,
    };

    let signer3 = Signer {
        id: "signer_3".to_string(),
        public_key: "pubkey_3".to_string(),
        weight: 5,
        last_activity: 0,
    };

    peg_manager.add_signer(signer1)?;
    peg_manager.add_signer(signer2)?;
    peg_manager.add_signer(signer3)?;

    println!("  - Added 3 signers to the federation");
    println!("  - Total signer weight: {}", peg_manager.get_total_weight());
    println!("  - Threshold weight: {}", peg_manager.get_threshold_weight());

    // Initiate a federated peg transaction
    peg_manager.initiate_peg_transaction(
        "peg_001".to_string(),
        "ethereum".to_string(),
        "bsc".to_string(),
        "0xSenderAddress".to_string(),
        "0xReceiverAddress".to_string(),
        "ETH".to_string(),
        500,
    )?;

    println!("  - Initiated federated peg transaction: peg_001");
    println!("  - Transferring 500 ETH from Ethereum to BSC");

    // Add signatures from signers
    peg_manager.add_signature("peg_001", "signer_1", "signature_1".to_string())?;
    peg_manager.add_signature("peg_001", "signer_2", "signature_2".to_string())?;
    peg_manager.add_signature("peg_001", "signer_3", "signature_3".to_string())?;

    println!("  - Collected signatures from all 3 signers");

    // Check if we have sufficient signatures
    if peg_manager.has_sufficient_signatures("peg_001")? {
        println!("  - Sufficient signatures collected for transaction");
    }

    // Complete the transaction
    peg_manager.complete_peg_transaction(
        "peg_001",
        Some("0xSourceTxHash".to_string()),
        Some("0xDestTxHash".to_string()),
    )?;

    println!("  - Federated peg transaction completed successfully");
    Ok(())
}

fn mpc_threshold_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create MPC threshold manager with custom configuration
    let mut mpc_manager = MpcThresholdManager::new(MpcThresholdConfig {
        threshold: 3,
        total_participants: 5,
        timeout_secs: 3600,
        max_concurrent_operations: 100,
    });

    // Add participants to the MPC network
    let participant1 = MpcParticipant {
        id: "participant_1".to_string(),
        public_key_share: "pubkey_share_1".to_string(),
        index: 1,
        last_activity: 0,
    };

    let participant2 = MpcParticipant {
        id: "participant_2".to_string(),
        public_key_share: "pubkey_share_2".to_string(),
        index: 2,
        last_activity: 0,
    };

    let participant3 = MpcParticipant {
        id: "participant_3".to_string(),
        public_key_share: "pubkey_share_3".to_string(),
        index: 3,
        last_activity: 0,
    };

    mpc_manager.add_participant(participant1)?;
    mpc_manager.add_participant(participant2)?;
    mpc_manager.add_participant(participant3)?;

    println!("  - Added 3 participants to the MPC network");
    println!("  - Threshold: {} shares required", mpc_manager.get_threshold());
    println!("  - Total participants: {}", mpc_manager.get_total_participants());

    // Initiate an MPC threshold transaction
    mpc_manager.initiate_mpc_transaction(
        "mpc_001".to_string(),
        "polygon".to_string(),
        "avalanche".to_string(),
        "0xSenderAddress".to_string(),
        "0xReceiverAddress".to_string(),
        "MATIC".to_string(),
        2000,
    )?;

    println!("  - Initiated MPC threshold transaction: mpc_001");
    println!("  - Transferring 2000 MATIC from Polygon to Avalanche");

    // Add shares from participants
    mpc_manager.add_share("mpc_001", 1, "share_1".to_string())?;
    mpc_manager.add_share("mpc_001", 2, "share_2".to_string())?;
    mpc_manager.add_share("mpc_001", 3, "share_3".to_string())?;

    println!("  - Collected shares from 3 participants");

    // Check if we have sufficient shares
    if mpc_manager.has_sufficient_shares("mpc_001")? {
        println!("  - Sufficient shares collected for transaction");
    }

    // Complete the transaction
    mpc_manager.complete_mpc_transaction(
        "mpc_001",
        Some("0xSourceTxHash".to_string()),
        Some("0xDestTxHash".to_string()),
    )?;

    println!("  - MPC threshold transaction completed successfully");
    Ok(())
}