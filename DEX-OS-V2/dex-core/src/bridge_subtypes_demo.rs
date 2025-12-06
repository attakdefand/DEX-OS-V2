//! Demonstration of Bridge Subtypes integration with Universal Bridge
//!
//! This module demonstrates how the Federated Peg and MPC Threshold mechanisms
//! integrate with the existing Universal Bridge system in DEX-OS.

use crate::federated_peg::{FederatedPegManager, FederatedPegConfig, Signer};
use crate::mpc_threshold::{MpcThresholdManager, MpcThresholdConfig, MpcParticipant};
use crate::universal_bridge::{UniversalBridgeManager, BridgeConfig, BridgeTransaction};
use crate::types::{TokenId, TraderId, Quantity};

/// Bridge subtype selector that determines which mechanism to use
#[derive(Debug, Clone)]
pub enum BridgeSubtype {
    /// Standard universal bridge mechanism
    Standard,
    /// Federated peg mechanism
    FederatedPeg,
    /// MPC threshold mechanism
    MpcThreshold,
}

/// Extended bridge manager that supports multiple bridge subtypes
pub struct ExtendedBridgeManager {
    /// Standard universal bridge manager
    standard_bridge: UniversalBridgeManager,
    /// Federated peg bridge manager
    federated_peg: FederatedPegManager,
    /// MPC threshold bridge manager
    mpc_threshold: MpcThresholdManager,
}

impl ExtendedBridgeManager {
    /// Create a new extended bridge manager
    pub fn new(
        standard_config: BridgeConfig,
        federated_config: FederatedPegConfig,
        mpc_config: MpcThresholdConfig,
    ) -> Self {
        Self {
            standard_bridge: UniversalBridgeManager::new(standard_config),
            federated_peg: FederatedPegManager::new(federated_config),
            mpc_threshold: MpcThresholdManager::new(mpc_config),
        }
    }

    /// Create a new extended bridge manager with default configurations
    pub fn with_default() -> Self {
        Self::new(
            BridgeConfig::default(),
            FederatedPegConfig::default(),
            MpcThresholdConfig::default(),
        )
    }

    /// Initiate a bridge transaction using the specified subtype
    pub fn initiate_bridge_transaction_with_subtype(
        &mut self,
        id: String,
        source_chain: String,
        destination_chain: String,
        sender: TraderId,
        receiver: TraderId,
        token_id: TokenId,
        amount: Quantity,
        subtype: BridgeSubtype,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match subtype {
            BridgeSubtype::Standard => {
                self.standard_bridge.initiate_bridge_transaction(
                    id, source_chain, destination_chain, sender, receiver, token_id, amount,
                )?;
                Ok(())
            }
            BridgeSubtype::FederatedPeg => {
                self.federated_peg.initiate_peg_transaction(
                    id, source_chain, destination_chain, sender, receiver, token_id, amount,
                )?;
                Ok(())
            }
            BridgeSubtype::MpcThreshold => {
                self.mpc_threshold.initiate_mpc_transaction(
                    id, source_chain, destination_chain, sender, receiver, token_id, amount,
                )?;
                Ok(())
            }
        }
    }

    /// Add a signer for federated peg mechanism
    pub fn add_federated_signer(&mut self, signer: Signer) -> Result<(), Box<dyn std::error::Error>> {
        self.federated_peg.add_signer(signer)?;
        Ok(())
    }

    /// Add a participant for MPC threshold mechanism
    pub fn add_mpc_participant(&mut self, participant: MpcParticipant) -> Result<(), Box<dyn std::error::Error>> {
        self.mpc_threshold.add_participant(participant)?;
        Ok(())
    }

    /// Get reference to standard bridge manager
    pub fn get_standard_bridge(&self) -> &UniversalBridgeManager {
        &self.standard_bridge
    }

    /// Get mutable reference to standard bridge manager
    pub fn get_standard_bridge_mut(&mut self) -> &mut UniversalBridgeManager {
        &mut self.standard_bridge
    }

    /// Get reference to federated peg manager
    pub fn get_federated_peg(&self) -> &FederatedPegManager {
        &self.federated_peg
    }

    /// Get mutable reference to federated peg manager
    pub fn get_federated_peg_mut(&mut self) -> &mut FederatedPegManager {
        &mut self.federated_peg
    }

    /// Get reference to MPC threshold manager
    pub fn get_mpc_threshold(&self) -> &MpcThresholdManager {
        &self.mpc_threshold
    }

    /// Get mutable reference to MPC threshold manager
    pub fn get_mpc_threshold_mut(&mut self) -> &mut MpcThresholdManager {
        &mut self.mpc_threshold
    }
}

impl Default for ExtendedBridgeManager {
    fn default() -> Self {
        Self::with_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universal_bridge::BlockchainNetwork;
    use std::collections::HashSet;

    #[test]
    fn test_extended_bridge_manager_creation() {
        let manager = ExtendedBridgeManager::new(
            BridgeConfig::default(),
            FederatedPegConfig::default(),
            MpcThresholdConfig::default(),
        );
        
        assert_eq!(manager.get_federated_peg().signer_count(), 0);
        assert_eq!(manager.get_mpc_threshold().participant_count(), 0);
    }

    #[test]
    fn test_initiate_standard_bridge_transaction() {
        let mut manager = ExtendedBridgeManager::with_default();
        
        // Add networks to standard bridge
        let ethereum = BlockchainNetwork {
            id: "ethereum".to_string(),
            name: "Ethereum".to_string(),
            chain_type: "EVM".to_string(),
            rpc_endpoint: "https://ethereum.rpc".to_string(),
            chain_id: 1,
            native_token: "ETH".to_string(),
            features: HashSet::new(),
            metrics: crate::universal_bridge::NetworkMetrics::default(),
        };
        
        let polygon = BlockchainNetwork {
            id: "polygon".to_string(),
            name: "Polygon".to_string(),
            chain_type: "EVM".to_string(),
            rpc_endpoint: "https://polygon.rpc".to_string(),
            chain_id: 137,
            native_token: "MATIC".to_string(),
            features: HashSet::new(),
            metrics: crate::universal_bridge::NetworkMetrics::default(),
        };
        
        manager.get_standard_bridge_mut().add_network(ethereum).unwrap();
        manager.get_standard_bridge_mut().add_network(polygon).unwrap();
        
        // Initiate standard bridge transaction
        let result = manager.initiate_bridge_transaction_with_subtype(
            "bridge1".to_string(),
            "ethereum".to_string(),
            "polygon".to_string(),
            "sender1".to_string(),
            "receiver1".to_string(),
            "ETH".to_string(),
            1000,
            BridgeSubtype::Standard,
        );
        
        assert!(result.is_ok());
        assert!(manager.get_standard_bridge().get_transaction("bridge1").is_some());
    }

    #[test]
    fn test_initiate_federated_peg_transaction() {
        let mut manager = ExtendedBridgeManager::with_default();
        
        // Initiate federated peg transaction
        let result = manager.initiate_bridge_transaction_with_subtype(
            "peg1".to_string(),
            "ethereum".to_string(),
            "polygon".to_string(),
            "sender1".to_string(),
            "receiver1".to_string(),
            "ETH".to_string(),
            1000,
            BridgeSubtype::FederatedPeg,
        );
        
        assert!(result.is_ok());
        assert!(manager.get_federated_peg().get_transaction("peg1").is_some());
    }

    #[test]
    fn test_initiate_mpc_threshold_transaction() {
        let mut manager = ExtendedBridgeManager::with_default();
        
        // Initiate MPC threshold transaction
        let result = manager.initiate_bridge_transaction_with_subtype(
            "mpc1".to_string(),
            "ethereum".to_string(),
            "polygon".to_string(),
            "sender1".to_string(),
            "receiver1".to_string(),
            "ETH".to_string(),
            1000,
            BridgeSubtype::MpcThreshold,
        );
        
        assert!(result.is_ok());
        assert!(manager.get_mpc_threshold().get_transaction("mpc1").is_some());
    }

    #[test]
    fn test_add_federated_signer() {
        let mut manager = ExtendedBridgeManager::with_default();
        
        let signer = Signer {
            id: "signer1".to_string(),
            public_key: "public_key_1".to_string(),
            weight: 10,
            last_activity: 0,
        };

        let result = manager.add_federated_signer(signer);
        assert!(result.is_ok());
        assert_eq!(manager.get_federated_peg().signer_count(), 1);
    }

    #[test]
    fn test_add_mpc_participant() {
        let mut manager = ExtendedBridgeManager::with_default();
        
        let participant = MpcParticipant {
            id: "participant1".to_string(),
            public_key_share: "public_key_share_1".to_string(),
            index: 1,
            last_activity: 0,
        };

        let result = manager.add_mpc_participant(participant);
        assert!(result.is_ok());
        assert_eq!(manager.get_mpc_threshold().participant_count(), 1);
    }
}