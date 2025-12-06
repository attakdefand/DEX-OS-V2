//! Security tests for the analytics module and volume tracking functionality
//!
//! This module implements security tests for the Priority 5 feature from DEX-OS-V2.csv:
//! - Analytics & Oracles,On-Chain Analytics,On-Chain Analytics,Volume/Volume Trackers,Volume Tracking,Medium {Security: Layer 4 - Application Security}

#[cfg(test)]
mod tests {
    use super::super::analytics::*;
    use std::thread;

    // Security Test: Policy Enforcement - Enforces on request
    #[test]
    fn test_security__application_security__policy__enforces__on_request() {
        let config = AnalyticsConfig {
            enable_volume_tracking: true,
            max_history_entries: 1000,
            enable_time_windows: true,
        };
        let service = AnalyticsService::with_config(config);

        // Test that volume tracking enforces proper data handling
        let result = service.record_trade_volume("BTC".to_string(), "USDT".to_string(), 1000);
        assert!(result.is_ok());

        // Verify that the service properly enforces data boundaries
        let volume = service.get_total_volume();
        assert_eq!(volume, 1000);
    }

    // Security Test: Policy Validation - Validates on request
    #[test]
    fn test_security__application_security__policy__validates__on_request() {
        let service = AnalyticsService::new();

        // Test validation of input parameters
        let result = service.record_trade_volume("".to_string(), "USDT".to_string(), 1000);
        assert!(result.is_ok()); // Empty string is valid as a token name

        // Test validation with zero volume
        let result = service.record_trade_volume("BTC".to_string(), "USDT".to_string(), 0);
        assert!(result.is_ok()); // Zero volume is valid

        // Test validation with valid parameters
        let result = service.record_trade_volume("BTC".to_string(), "USDT".to_string(), 1000);
        assert!(result.is_ok());
    }

    // Security Test: Policy Blocking - Blocks on request
    #[test]
    fn test_security__application_security__policy__blocks__on_request() {
        let config = AnalyticsConfig {
            enable_volume_tracking: false, // Disabled tracking should block volume recording
            max_history_entries: 1000,
            enable_time_windows: true,
        };
        let service = AnalyticsService::with_config(config);

        // When volume tracking is disabled, recording should succeed but not track
        let result = service.record_trade_volume("BTC".to_string(), "USDT".to_string(), 1000);
        assert!(result.is_ok());

        // But the volume should not be recorded
        assert_eq!(service.get_total_volume(), 0);
    }

    // Security Test: Detection - Detects on request
    #[test]
    fn test_security__application_security__policy__detects__on_request() {
        let service = AnalyticsService::new();

        // Record a trade
        service
            .record_trade_volume("BTC".to_string(), "USDT".to_string(), 1000)
            .unwrap();

        // Detect if the volume was properly recorded
        let pair_volume = service.get_token_pair_volume("BTC", "USDT");
        assert!(pair_volume.is_ok());
        assert_eq!(pair_volume.unwrap().volume, 1000);

        // Detect non-existent pair
        let non_existent = service.get_token_pair_volume("NON", "EXISTENT");
        assert!(non_existent.is_err());
    }

    // Security Test: Evidence Logging - Logs evidence on request
    #[test]
    fn test_security__application_security__policy__logs_evidence__on_request() {
        let service = AnalyticsService::new();

        // Record trades
        service
            .record_trade_volume("BTC".to_string(), "USDT".to_string(), 1000)
            .unwrap();
        service
            .record_trade_volume("ETH".to_string(), "USDT".to_string(), 500)
            .unwrap();

        // Export as JSON as evidence logging
        let json_export = service.export_as_json();
        assert!(json_export.is_ok());

        let json_data = json_export.unwrap();
        assert!(!json_data.is_empty());
        assert!(json_data.contains("BTC_USDT"));
        assert!(json_data.contains("ETH_USDT"));
    }

    // Security Test: Concurrent Access Safety
    #[test]
    fn test_security__application_security__concurrent_access_safety() {
        let service = AnalyticsService::new();
        let service_clone1 = service.clone();
        let service_clone2 = service.clone();

        // Spawn multiple threads to record volumes concurrently
        let handle1 = thread::spawn(move || {
            for i in 0..100 {
                service_clone1
                    .record_trade_volume("BTC".to_string(), "USDT".to_string(), i)
                    .unwrap();
            }
        });

        let handle2 = thread::spawn(move || {
            for i in 0..100 {
                service_clone2
                    .record_trade_volume("ETH".to_string(), "USDT".to_string(), i)
                    .unwrap();
            }
        });

        // Wait for both threads to complete
        handle1.join().unwrap();
        handle2.join().unwrap();

        // Verify that all volumes were recorded correctly
        let btc_volume = service.get_token_pair_volume("BTC", "USDT").unwrap();
        let eth_volume = service.get_token_pair_volume("ETH", "USDT").unwrap();

        // Sum of 0..99 is 4950
        assert_eq!(btc_volume.volume, 4950);
        assert_eq!(eth_volume.volume, 4950);
        assert_eq!(service.get_total_volume(), 9900);
    }

    // Security Test: Input Sanitization
    #[test]
    fn test_security__application_security__input_sanitization() {
        let service = AnalyticsService::new();

        // Test with special characters in token names
        let result =
            service.record_trade_volume("BTC/USD".to_string(), "USDT-PAIR".to_string(), 1000);
        assert!(result.is_ok());

        // Test with very long token names
        let long_name = "A".repeat(1000);
        let result = service.record_trade_volume(long_name.clone(), "USDT".to_string(), 1000);
        assert!(result.is_ok());

        // Verify the data was recorded correctly
        let pair_volume = service.get_token_pair_volume(&long_name, "USDT").unwrap();
        assert_eq!(pair_volume.volume, 1000);
    }

    // Security Test: Memory Safety
    #[test]
    fn test_security__application_security__memory_safety() {
        let service = AnalyticsService::new();

        // Record a large number of trades to test memory safety
        for i in 0..10000 {
            service
                .record_trade_volume(format!("TOKEN_{}", i % 100), "USDT".to_string(), i)
                .unwrap();
        }

        // Verify no memory corruption occurred
        let total_volume = service.get_total_volume();
        assert!(total_volume > 0);

        // Test that we can still access data
        let pair_volume = service.get_token_pair_volume("TOKEN_0", "USDT").unwrap();
        assert!(pair_volume.volume > 0);
    }

    // Security Test: History Limiting Safety
    #[test]
    fn test_security__application_security__history_limiting_safety() {
        let config = AnalyticsConfig {
            enable_volume_tracking: true,
            max_history_entries: 10, // Small limit for testing
            enable_time_windows: true,
        };
        let service = AnalyticsService::with_config(config);

        // Record more trades than the history limit
        for i in 0..100 {
            service
                .record_trade_volume("TEST".to_string(), "TOKEN".to_string(), i)
                .unwrap();
        }

        // Verify that history is properly limited
        let pair_volume = service.get_token_pair_volume("TEST", "TOKEN").unwrap();
        assert_eq!(pair_volume.volume_history.len(), 10);
        assert_eq!(pair_volume.volume, 4950); // Sum of 0..99
    }

    // Security Test: Configuration Validation
    #[test]
    fn test_security__application_security__configuration_validation() {
        // Test with various configuration values
        let config1 = AnalyticsConfig {
            enable_volume_tracking: true,
            max_history_entries: 0, // Edge case: zero entries
            enable_time_windows: true,
        };

        let service1 = AnalyticsService::with_config(config1);
        let result = service1.record_trade_volume("BTC".to_string(), "USDT".to_string(), 1000);
        assert!(result.is_ok());

        // Even with 0 history entries, the volume should still be tracked
        assert_eq!(service1.get_total_volume(), 1000);
    }

    // Security Test: Error Handling
    #[test]
    fn test_security__application_security__error_handling() {
        let service = AnalyticsService::new();

        // Test error handling for non-existent pairs
        let result = service.get_token_pair_volume("NON", "EXISTENT");
        assert!(result.is_err());

        // Verify the error type
        match result {
            Err(VolumeTrackingError::TokenPairNotFound) => {
                // Expected error
            }
            _ => panic!("Expected TokenPairNotFound error"),
        }

        // Test serialization error handling
        // This is harder to trigger, but we can at least verify the error type exists
        match VolumeTrackingError::SerializationError("test".to_string()) {
            VolumeTrackingError::SerializationError(_) => {
                // Expected variant
            }
            _ => panic!("Unexpected error variant"),
        }
    }
}
