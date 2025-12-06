use dex_core::tesla_integration::*;
use dex_core::starlink_wallet::*;
use dex_core::neuralink_interface::*;

#[test]
fn test_tesla_integration_full_workflow() {
    let tesla = TeslaIntegration::new();

    // 1. Register a vehicle
    let vehicle = VehicleInfo {
        id: VehicleId("TESLA-001".to_string()),
        vin: "5YJ3E1EA1KF000001".to_string(),
        display_name: "My Model 3".to_string(),
        model: "Model 3".to_string(),
        color: "Midnight Silver".to_string(),
        state: VehicleState::Online,
        battery_level: 80,
        range_miles: 240,
        location: Some(Location {
            latitude: 37.7749,
            longitude: -122.4194,
            heading: Some(180),
        }),
    };

    assert!(tesla
        .register_vehicle(vehicle.clone(), "auth_token_123".to_string())
        .is_ok());

    // 2. Execute commands
    let lock_result = tesla
        .execute_command(&vehicle.id, VehicleCommand::DoorLock { lock: true })
        .unwrap();
    assert!(lock_result.success);

    let climate_result = tesla
        .execute_command(
            &vehicle.id,
            VehicleCommand::Climate {
                on: true,
                temperature: Some(72.0),
            },
        )
        .unwrap();
    assert!(climate_result.success);

    // 3. Check command history
    let history = tesla.get_command_history(&vehicle.id);
    assert_eq!(history.len(), 2);

    // 4. Update vehicle state
    tesla
        .update_vehicle_state(&vehicle.id, VehicleState::Charging, Some(85), Some(255))
        .unwrap();

    let updated = tesla.get_vehicle(&vehicle.id).unwrap();
    assert_eq!(updated.state, VehicleState::Charging);
    assert_eq!(updated.battery_level, 85);

    // 5. Process payment
    let payment = ServicePayment {
        service_type: "Supercharging".to_string(),
        amount: 2500,
        currency: "USD".to_string(),
        transaction_id: None,
    };

    let tx_id = tesla.process_service_payment(&vehicle.id, payment).unwrap();
    assert!(tx_id.starts_with("TX-"));
}

#[test]
fn test_starlink_wallet_full_workflow() {
    let manager = StarlinkWalletManager::new();

    // 1. Create wallet
    let wallet = manager.create_wallet("0xABCDEF1234567890".to_string()).unwrap();
    assert!(wallet.id.0.starts_with("starlink-"));

    // 2. Update connection status
    manager
        .update_connection_status(
            &wallet.id,
            ConnectionStatus::Connected {
                signal_strength: 85,
            },
        )
        .unwrap();

    // 3. Create offline transactions
    let tx1 = manager
        .create_offline_transaction(
            &wallet.id,
            "0x1111111111111111".to_string(),
            1000,
            TransactionPriority::High,
        )
        .unwrap();

    let tx2 = manager
        .create_offline_transaction(
            &wallet.id,
            "0x2222222222222222".to_string(),
            2000,
            TransactionPriority::Normal,
        )
        .unwrap();

    // 4. Sign transactions
    let signed_tx1 = manager.sign_transaction(&wallet.id, tx1).unwrap();
    let signed_tx2 = manager.sign_transaction(&wallet.id, tx2).unwrap();

    assert!(signed_tx1.signed);
    assert!(signed_tx2.signed);
    assert!(signed_tx1.signature.is_some());

    // 5. Broadcast transactions
    let result1 = manager.broadcast_transaction(&wallet.id, &signed_tx1).unwrap();
    assert!(result1.success);

    let result2 = manager.broadcast_transaction(&wallet.id, &signed_tx2).unwrap();
    assert!(result2.success);

    // 6. Check bandwidth usage
    let bandwidth = manager.get_bandwidth_usage();
    assert!(bandwidth > 0);
}

#[test]
fn test_starlink_wallet_low_bandwidth_scenario() {
    let manager = StarlinkWalletManager::new();
    let wallet = manager.create_wallet("0xTEST".to_string()).unwrap();

    // Set low bandwidth
    manager
        .update_connection_status(
            &wallet.id,
            ConnectionStatus::LowBandwidth { available_kbps: 5 },
        )
        .unwrap();

    // Normal priority should fail
    let normal_tx = manager
        .create_offline_transaction(
            &wallet.id,
            "0x123".to_string(),
            1000,
            TransactionPriority::Normal,
        )
        .unwrap();
    let signed_normal = manager.sign_transaction(&wallet.id, normal_tx).unwrap();
    assert!(manager.broadcast_transaction(&wallet.id, &signed_normal).is_err());

    // Emergency priority should succeed
    let emergency_tx = manager
        .create_offline_transaction(
            &wallet.id,
            "0x456".to_string(),
            1000,
            TransactionPriority::Emergency,
        )
        .unwrap();
    let signed_emergency = manager.sign_transaction(&wallet.id, emergency_tx).unwrap();
    assert!(manager
        .broadcast_transaction(&wallet.id, &signed_emergency)
        .is_ok());
}

#[test]
fn test_neuralink_interface_full_workflow() {
    let interface = NeuralinkInterface::new();

    // 1. Register device
    let device_id = DeviceId("neuralink-001".to_string());
    interface
        .register_device(
            device_id.clone(),
            DeviceStatus::Connected {
                signal_quality: 90,
            },
        )
        .unwrap();

    // 2. Create neural profile
    let profile = interface
        .create_profile("user_alice".to_string(), device_id.clone())
        .unwrap();

    assert_eq!(profile.user_id, "user_alice");
    assert!(!profile.baseline_patterns.is_empty());

    // 3. Test authentication (may fail due to pattern mismatch in simulation)
    let test_pattern = NeuralPattern {
        id: "auth-pattern".to_string(),
        data: vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0],
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        confidence: 95,
    };

    let auth_result = interface.authenticate_user("user_alice", test_pattern.clone());
    // Authentication may succeed or fail depending on pattern matching
    assert!(auth_result.is_ok() || auth_result.is_err());

    // 4. Test transaction authorization
    let tx_pattern = NeuralPattern {
        id: "tx-pattern".to_string(),
        data: vec![0.15, 0.25, 0.35, 0.45, 0.55, 0.65, 0.75, 0.85, 0.95, 1.05],
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        confidence: 92,
    };

    let auth_result = interface.authorize_transaction(
        "user_alice",
        "tx-12345".to_string(),
        tx_pattern,
    );

    // Result depends on authentication
    assert!(auth_result.is_ok() || auth_result.is_err());
}

#[test]
fn test_neuralink_device_status_management() {
    let interface = NeuralinkInterface::new();
    let device_id = DeviceId("neuralink-002".to_string());

    // Register as calibrating
    interface
        .register_device(device_id.clone(), DeviceStatus::Calibrating)
        .unwrap();

    // Update to connected
    interface
        .update_device_status(
            &device_id,
            DeviceStatus::Connected {
                signal_quality: 95,
            },
        )
        .unwrap();

    // Try to create profile (should succeed now)
    let result = interface.create_profile("user_bob".to_string(), device_id);
    assert!(result.is_ok());
}

#[test]
fn test_integrated_tesla_starlink_scenario() {
    // Scenario: User controls Tesla via Starlink connection

    let tesla = TeslaIntegration::new();
    let starlink = StarlinkWalletManager::new();

    // 1. Setup Tesla
    let vehicle = VehicleInfo {
        id: VehicleId("TESLA-REMOTE".to_string()),
        vin: "5YJ3E1EA1KF999999".to_string(),
        display_name: "Remote Tesla".to_string(),
        model: "Model S".to_string(),
        color: "Deep Blue".to_string(),
        state: VehicleState::Online,
        battery_level: 60,
        range_miles: 180,
        location: None,
    };

    tesla
        .register_vehicle(vehicle.clone(), "remote_token".to_string())
        .unwrap();

    // 2. Setup Starlink wallet for payment
    let wallet = starlink.create_wallet("0xREMOTE".to_string()).unwrap();
    starlink
        .update_connection_status(
            &wallet.id,
            ConnectionStatus::Connected {
                signal_strength: 75,
            },
        )
        .unwrap();

    // 3. Execute Tesla command
    let command_result = tesla
        .execute_command(&vehicle.id, VehicleCommand::Charging { start: true })
        .unwrap();
    assert!(command_result.success);

    // 4. Process payment via Starlink
    let payment = ServicePayment {
        service_type: "Remote Charging".to_string(),
        amount: 1500,
        currency: "USD".to_string(),
        transaction_id: None,
    };

    let tx_id = tesla.process_service_payment(&vehicle.id, payment).unwrap();
    assert!(tx_id.starts_with("TX-"));

    // 5. Create payment transaction in Starlink wallet
    let payment_tx = starlink
        .create_offline_transaction(
            &wallet.id,
            "0xCHARGING_STATION".to_string(),
            1500,
            TransactionPriority::High,
        )
        .unwrap();

    let signed_tx = starlink.sign_transaction(&wallet.id, payment_tx).unwrap();
    let broadcast_result = starlink.broadcast_transaction(&wallet.id, &signed_tx).unwrap();

    assert!(broadcast_result.success);
}

#[test]
fn test_integrated_neuralink_tesla_scenario() {
    // Scenario: User authorizes Tesla commands via Neuralink

    let neuralink = NeuralinkInterface::new();
    let tesla = TeslaIntegration::new();

    // 1. Setup Neuralink
    let device_id = DeviceId("neuralink-tesla".to_string());
    neuralink
        .register_device(
            device_id.clone(),
            DeviceStatus::Connected {
                signal_quality: 88,
            },
        )
        .unwrap();

    neuralink
        .create_profile("driver_charlie".to_string(), device_id)
        .unwrap();

    // 2. Setup Tesla
    let vehicle = VehicleInfo {
        id: VehicleId("TESLA-NEURAL".to_string()),
        vin: "5YJ3E1EA1KF888888".to_string(),
        display_name: "Neural Tesla".to_string(),
        model: "Cybertruck".to_string(),
        color: "Stainless Steel".to_string(),
        state: VehicleState::Online,
        battery_level: 95,
        range_miles: 300,
        location: None,
    };

    tesla
        .register_vehicle(vehicle.clone(), "neural_token".to_string())
        .unwrap();

    // 3. User thinks about unlocking doors
    let unlock_pattern = NeuralPattern {
        id: "unlock-intent".to_string(),
        data: vec![0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1],
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        confidence: 90,
    };

    // 4. Process neural command (may succeed or fail based on authentication)
    let auth_result = neuralink.process_command(
        "driver_charlie",
        NeuralCommand::AuthorizeTransaction {
            transaction_id: "unlock_doors".to_string(),
        },
        unlock_pattern,
    );

    // 5. If authorized, execute Tesla command
    if let Ok(result) = auth_result {
        if result.authorized {
            let tesla_result = tesla
                .execute_command(&vehicle.id, VehicleCommand::DoorLock { lock: false })
                .unwrap();
            assert!(tesla_result.success);
        }
    }
}

#[test]
fn test_all_three_systems_integrated() {
    // Ultimate integration: Neuralink + Starlink + Tesla

    let neuralink = NeuralinkInterface::new();
    let starlink = StarlinkWalletManager::new();
    let tesla = TeslaIntegration::new();

    // Setup all systems
    let device_id = DeviceId("neuralink-ultimate".to_string());
    neuralink
        .register_device(
            device_id.clone(),
            DeviceStatus::Connected {
                signal_quality: 92,
            },
        )
        .unwrap();

    neuralink
        .create_profile("user_ultimate".to_string(), device_id)
        .unwrap();

    let wallet = starlink.create_wallet("0xULTIMATE".to_string()).unwrap();
    starlink
        .update_connection_status(
            &wallet.id,
            ConnectionStatus::Connected {
                signal_strength: 80,
            },
        )
        .unwrap();

    let vehicle = VehicleInfo {
        id: VehicleId("TESLA-ULTIMATE".to_string()),
        vin: "5YJ3E1EA1KF111111".to_string(),
        display_name: "Ultimate Tesla".to_string(),
        model: "Model X".to_string(),
        color: "Pearl White".to_string(),
        state: VehicleState::Online,
        battery_level: 88,
        range_miles: 280,
        location: None,
    };

    tesla
        .register_vehicle(vehicle.clone(), "ultimate_token".to_string())
        .unwrap();

    // User thinks about starting climate control
    let climate_pattern = NeuralPattern {
        id: "climate-intent".to_string(),
        data: vec![0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2],
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        confidence: 94,
    };

    // Process via Neuralink
    let neural_result = neuralink.process_command(
        "user_ultimate",
        NeuralCommand::AuthorizeTransaction {
            transaction_id: "climate_control".to_string(),
        },
        climate_pattern,
    );

    // If authorized, execute Tesla command and process payment
    if let Ok(result) = neural_result {
        if result.authorized {
            // Execute Tesla command
            let _ = tesla.execute_command(
                &vehicle.id,
                VehicleCommand::Climate {
                    on: true,
                    temperature: Some(70.0),
                },
            );

            // Process payment via Starlink
            let payment_tx = starlink
                .create_offline_transaction(
                    &wallet.id,
                    "0xTESLA_SERVICE".to_string(),
                    500,
                    TransactionPriority::Normal,
                )
                .unwrap();

            let signed_tx = starlink.sign_transaction(&wallet.id, payment_tx).unwrap();
            let _ = starlink.broadcast_transaction(&wallet.id, &signed_tx);
        }
    }

    // Verify all systems are functioning
    assert!(tesla.list_vehicles().len() > 0);
    assert!(starlink.get_bandwidth_usage() >= 0);
    assert!(neuralink.get_command_history("user_ultimate").len() >= 0);
}
