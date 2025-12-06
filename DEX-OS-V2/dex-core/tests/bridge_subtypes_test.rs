//! Integration test for bridge subtypes functionality

#[cfg(test)]
mod tests {
    use dex_core::federated_peg::{FederatedPegManager, FederatedPegConfig, Signer};
    use dex_core::mpc_threshold::{MpcThresholdManager, MpcThresholdConfig, MpcParticipant};

    #[test]
    fn test_federated_peg_module() {
        let manager = FederatedPegManager::new(FederatedPegConfig::default());
        assert_eq!(manager.signer_count(), 0);
    }

    #[test]
    fn test_mpc_threshold_module() {
        let manager = MpcThresholdManager::new(MpcThresholdConfig::default());
        assert_eq!(manager.participant_count(), 0);
    }

    #[test]
    fn test_federated_peg_with_signers() {
        let mut manager = FederatedPegManager::new(FederatedPegConfig::default());
        
        let signer = Signer {
            id: "test_signer".to_string(),
            public_key: "test_public_key".to_string(),
            weight: 10,
            last_activity: 0,
        };

        assert!(manager.add_signer(signer).is_ok());
        assert_eq!(manager.signer_count(), 1);
        assert!(manager.get_signer("test_signer").is_some());
    }

    #[test]
    fn test_mpc_threshold_with_participants() {
        let mut manager = MpcThresholdManager::new(MpcThresholdConfig::default());
        
        let participant = MpcParticipant {
            id: "test_participant".to_string(),
            public_key_share: "test_public_key_share".to_string(),
            index: 1,
            last_activity: 0,
        };

        assert!(manager.add_participant(participant).is_ok());
        assert_eq!(manager.participant_count(), 1);
        assert!(manager.get_participant("test_participant").is_some());
    }
}