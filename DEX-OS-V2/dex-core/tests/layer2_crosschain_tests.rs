//! Comprehensive Integration Tests for Layer 2 Scaling and Cross-Chain Protocols
//!
//! Tests for:
//! - State Channels for Off-chain Orders
//! - Batch Settlements
//! - IBC (Inter-Blockchain Communication)

use dex_core::state_channels::{
    ChannelConfig, OffChainOrder, Participant, StateChannelManager, StateUpdate,
};
use dex_core::batch_settlements::{
    BatchConfig, BatchSettlementManager, Transaction, TransactionType,
};
use dex_core::ibc_protocol::{
    ConsensusState, IBCManager, PacketOrdering, Proof, TransferData,
};
use std::collections::HashMap;

// ============================================================================
// State Channels Tests
// ============================================================================

#[test]
fn test_state_channel_full_lifecycle() {
    let config = ChannelConfig::default();
    let manager = StateChannelManager::new(config);

    let participants = vec![
        Participant {
            address: "alice".to_string(),
            public_key: vec![1, 2, 3],
        },
        Participant {
            address: "bob".to_string(),
            public_key: vec![4, 5, 6],
        },
    ];

    // Open channel
    assert!(manager
        .open_channel("channel1".to_string(), participants)
        .is_ok());

    // Deposit funds
    assert!(manager.deposit("channel1", "alice", 10000).is_ok());
    assert!(manager.deposit("channel1", "bob", 5000).is_ok());

    // Activate channel
    assert!(manager.activate_channel("channel1").is_ok());

    // Submit off-chain orders
    let order1 = OffChainOrder {
        id: "order1".to_string(),
        channel_id: "channel1".to_string(),
        maker: "alice".to_string(),
        taker: "bob".to_string(),
        base_asset: "ETH".to_string(),
        quote_asset: "USDT".to_string(),
        amount: 100,
        price: 2000,
        nonce: 1,
        timestamp: 1000,
        signature: vec![1, 2, 3],
    };

    assert!(manager.submit_order("channel1", order1).is_ok());

    // Update state
    let mut balances = HashMap::new();
    balances.insert("alice".to_string(), 9800);
    balances.insert("bob".to_string(), 5100);

    let mut signatures = HashMap::new();
    signatures.insert("alice".to_string(), vec![1, 2, 3]);
    signatures.insert("bob".to_string(), vec![4, 5, 6]);

    let update = StateUpdate {
        nonce: 2,
        balances,
        orders: Vec::new(),
        timestamp: 2000,
        signatures,
    };

    assert!(manager.update_state("channel1", update).is_ok());

    // Close channel
    assert!(manager.close_channel("channel1").is_ok());

    // Get statistics
    let stats = manager.get_statistics();
    assert_eq!(stats.total_channels, 1);
}

#[test]
fn test_state_channel_multiple_orders() {
    let config = ChannelConfig::default();
    let manager = StateChannelManager::new(config);

    let participants = vec![
        Participant {
            address: "trader1".to_string(),
            public_key: vec![1],
        },
        Participant {
            address: "trader2".to_string(),
            public_key: vec![2],
        },
    ];

    manager
        .open_channel("trading_channel".to_string(), participants)
        .unwrap();
    manager.deposit("trading_channel", "trader1", 50000).unwrap();
    manager.deposit("trading_channel", "trader2", 50000).unwrap();
    manager.activate_channel("trading_channel").unwrap();

    // Submit multiple orders
    for i in 0..10 {
        let order = OffChainOrder {
            id: format!("order{}", i),
            channel_id: "trading_channel".to_string(),
            maker: "trader1".to_string(),
            taker: "trader2".to_string(),
            base_asset: "BTC".to_string(),
            quote_asset: "USDT".to_string(),
            amount: 10,
            price: 50000,
            nonce: i as u64,
            timestamp: 1000 + i as u64,
            signature: vec![i as u8],
        };

        assert!(manager.submit_order("trading_channel", order).is_ok());
    }

    let channel = manager.get_channel("trading_channel").unwrap();
    assert_eq!(channel.current_state.orders.len(), 10);
}

#[test]
fn test_state_channel_concurrent_channels() {
    let config = ChannelConfig::default();
    let manager = StateChannelManager::new(config);

    // Create multiple channels
    for i in 0..5 {
        let participants = vec![
            Participant {
                address: format!("user{}", i * 2),
                public_key: vec![i * 2],
            },
            Participant {
                address: format!("user{}", i * 2 + 1),
                public_key: vec![i * 2 + 1],
            },
        ];

        let channel_id = format!("channel{}", i);
        manager.open_channel(channel_id.clone(), participants).unwrap();
        manager.deposit(&channel_id, &format!("user{}", i * 2), 1000).unwrap();
        manager.activate_channel(&channel_id).unwrap();
    }

    let active_channels = manager.get_active_channels();
    assert_eq!(active_channels.len(), 5);
}

// ============================================================================
// Batch Settlements Tests
// ============================================================================

#[test]
fn test_batch_settlement_full_lifecycle() {
    let config = BatchConfig::default();
    let manager = BatchSettlementManager::new(config);

    // Create batch
    assert!(manager.create_batch("batch1".to_string()).is_ok());

    // Add transactions
    for i in 0..50 {
        let tx = Transaction::new(
            format!("tx{}", i),
            TransactionType::Transfer,
            format!("sender{}", i),
            format!("receiver{}", i),
            "ETH".to_string(),
            100 + i as u64,
            1,
        );

        assert!(manager.add_transaction(tx).is_ok());
    }

    // Get batch
    let batch = manager.get_batch("batch1").unwrap();
    assert_eq!(batch.transactions.len(), 50);

    // Settle batch
    assert!(manager
        .settle_batch("batch1", "0xabc123".to_string())
        .is_ok());

    // Verify settlement
    let settled_batch = manager.get_batch("batch1").unwrap();
    assert!(settled_batch.settlement_tx_hash.is_some());
}

#[test]
fn test_batch_settlement_auto_batching() {
    let mut config = BatchConfig::default();
    config.max_batch_size = 10;
    let manager = BatchSettlementManager::new(config);

    // Add transactions that will fill multiple batches
    for i in 0..25 {
        let tx = Transaction::new(
            format!("tx{}", i),
            TransactionType::Trade,
            "alice".to_string(),
            "bob".to_string(),
            "BTC".to_string(),
            1000,
            10,
        );

        let _ = manager.add_transaction(tx);
    }

    let stats = manager.get_statistics();
    assert!(stats.total_batches >= 2); // Should have created multiple batches
}

#[test]
fn test_batch_settlement_net_balances() {
    let config = BatchConfig::default();
    let manager = BatchSettlementManager::new(config);

    manager.create_batch("batch1".to_string()).unwrap();

    // Create circular transactions
    let tx1 = Transaction::new(
        "tx1".to_string(),
        TransactionType::Transfer,
        "alice".to_string(),
        "bob".to_string(),
        "ETH".to_string(),
        1000,
        10,
    );

    let tx2 = Transaction::new(
        "tx2".to_string(),
        TransactionType::Transfer,
        "bob".to_string(),
        "charlie".to_string(),
        "ETH".to_string(),
        500,
        5,
    );

    let tx3 = Transaction::new(
        "tx3".to_string(),
        TransactionType::Transfer,
        "charlie".to_string(),
        "alice".to_string(),
        "ETH".to_string(),
        250,
        3,
    );

    manager.add_transaction(tx1).unwrap();
    manager.add_transaction(tx2).unwrap();
    manager.add_transaction(tx3).unwrap();

    let batch = manager.get_batch("batch1").unwrap();

    // Verify net balances
    assert!(batch.net_balances.contains_key("alice"));
    assert!(batch.net_balances.contains_key("bob"));
    assert!(batch.net_balances.contains_key("charlie"));
}

#[test]
fn test_batch_settlement_merkle_proofs() {
    let config = BatchConfig::default();
    let manager = BatchSettlementManager::new(config);

    manager.create_batch("batch1".to_string()).unwrap();

    // Add transactions
    for i in 0..10 {
        let tx = Transaction::new(
            format!("tx{}", i),
            TransactionType::Swap,
            "user1".to_string(),
            "user2".to_string(),
            "USDT".to_string(),
            100,
            1,
        );

        manager.add_transaction(tx).unwrap();
    }

    // Get batch and verify Merkle root
    let _batch = manager.get_batch("batch1").unwrap();
    manager.settle_batch("batch1", "0xhash".to_string()).unwrap();

    let settled_batch = manager.get_batch("batch1").unwrap();
    assert!(settled_batch.merkle_root.is_some());

    // Get Merkle proof for a transaction
    let proof = settled_batch.get_merkle_proof("tx5").unwrap();
    assert_eq!(proof.leaf_index, 5);
}

#[test]
fn test_batch_settlement_statistics() {
    let config = BatchConfig::default();
    let manager = BatchSettlementManager::new(config);

    manager.create_batch("batch1".to_string()).unwrap();

    // Add various transaction types
    let tx_types = vec![
        TransactionType::Transfer,
        TransactionType::Trade,
        TransactionType::Swap,
        TransactionType::Deposit,
        TransactionType::Withdrawal,
    ];

    for (i, tx_type) in tx_types.iter().enumerate() {
        let tx = Transaction::new(
            format!("tx{}", i),
            *tx_type,
            "sender".to_string(),
            "receiver".to_string(),
            "ETH".to_string(),
            100,
            1,
        );

        manager.add_transaction(tx).unwrap();
    }

    let batch = manager.get_batch("batch1").unwrap();
    let stats = batch.get_statistics();

    assert_eq!(stats.total_transactions, 5);
    assert_eq!(stats.total_fees, 5);
    assert_eq!(stats.transaction_types.len(), 5);
}

// ============================================================================
// IBC Protocol Tests
// ============================================================================

#[test]
fn test_ibc_full_connection_flow() {
    let manager = IBCManager::new("chain1".to_string());

    // Create light client
    assert!(manager
        .create_client("client1".to_string(), "chain2".to_string())
        .is_ok());

    // Update client with consensus state
    let state = ConsensusState {
        height: 1,
        timestamp: 1000,
        root: vec![1, 2, 3],
        next_validators_hash: vec![4, 5, 6],
    };

    assert!(manager.update_client("client1", 1, state).is_ok());

    // Create connection
    assert!(manager
        .create_connection(
            "conn1".to_string(),
            "client1".to_string(),
            "client2".to_string()
        )
        .is_ok());

    // Open connection
    assert!(manager
        .open_connection("conn1", "conn2".to_string())
        .is_ok());

    // Create channel
    assert!(manager
        .create_channel(
            "channel1".to_string(),
            "conn1".to_string(),
            "transfer".to_string(),
            PacketOrdering::Unordered
        )
        .is_ok());

    // Open channel
    assert!(manager
        .open_channel("channel1", "channel2".to_string())
        .is_ok());

    let stats = manager.get_statistics();
    assert_eq!(stats.total_clients, 1);
    assert_eq!(stats.open_connections, 1);
    assert_eq!(stats.open_channels, 1);
}

#[test]
fn test_ibc_packet_transmission() {
    let manager = IBCManager::new("chain1".to_string());

    // Setup
    manager
        .create_client("client1".to_string(), "chain2".to_string())
        .unwrap();

    let state = ConsensusState {
        height: 1,
        timestamp: 1000,
        root: vec![1, 2, 3],
        next_validators_hash: vec![4, 5, 6],
    };
    manager.update_client("client1", 1, state).unwrap();

    manager
        .create_connection(
            "conn1".to_string(),
            "client1".to_string(),
            "client2".to_string(),
        )
        .unwrap();
    manager
        .open_connection("conn1", "conn2".to_string())
        .unwrap();

    manager
        .create_channel(
            "channel1".to_string(),
            "conn1".to_string(),
            "transfer".to_string(),
            PacketOrdering::Unordered,
        )
        .unwrap();
    manager
        .open_channel("channel1", "channel2".to_string())
        .unwrap();

    // Send packet
    let packet = manager
        .send_packet("channel1", vec![1, 2, 3, 4, 5], 1000, 5000)
        .unwrap();

    assert_eq!(packet.sequence, 1);
    assert_eq!(packet.source_channel, "channel1");
    assert_eq!(packet.destination_channel, "channel2");
}

#[test]
fn test_ibc_token_transfer() {
    let manager = IBCManager::new("cosmos-hub".to_string());

    // Setup
    manager
        .create_client("client1".to_string(), "osmosis".to_string())
        .unwrap();

    let state = ConsensusState {
        height: 100,
        timestamp: 1000,
        root: vec![1, 2, 3],
        next_validators_hash: vec![4, 5, 6],
    };
    manager.update_client("client1", 100, state).unwrap();

    manager
        .create_connection(
            "conn1".to_string(),
            "client1".to_string(),
            "client2".to_string(),
        )
        .unwrap();
    manager
        .open_connection("conn1", "conn2".to_string())
        .unwrap();

    manager
        .create_channel(
            "channel1".to_string(),
            "conn1".to_string(),
            "transfer".to_string(),
            PacketOrdering::Unordered,
        )
        .unwrap();
    manager
        .open_channel("channel1", "channel2".to_string())
        .unwrap();

    // Transfer tokens
    let packet = manager
        .transfer(
            "channel1",
            "uatom".to_string(),
            1000000,
            "cosmos1sender".to_string(),
            "osmo1receiver".to_string(),
            1000,
            5000,
        )
        .unwrap();

    assert_eq!(packet.sequence, 1);
    assert!(!packet.data.is_empty());

    // Decode transfer data
    let transfer_data = TransferData::decode(&packet.data).unwrap();
    assert_eq!(transfer_data.denom, "uatom");
    assert_eq!(transfer_data.amount, 1000000);
}

#[test]
fn test_ibc_multiple_channels() {
    let manager = IBCManager::new("chain1".to_string());

    // Setup client and connection
    manager
        .create_client("client1".to_string(), "chain2".to_string())
        .unwrap();

    let state = ConsensusState {
        height: 1,
        timestamp: 1000,
        root: vec![1, 2, 3],
        next_validators_hash: vec![4, 5, 6],
    };
    manager.update_client("client1", 1, state).unwrap();

    manager
        .create_connection(
            "conn1".to_string(),
            "client1".to_string(),
            "client2".to_string(),
        )
        .unwrap();
    manager
        .open_connection("conn1", "conn2".to_string())
        .unwrap();

    // Create multiple channels
    for i in 0..5 {
        let channel_id = format!("channel{}", i);
        manager
            .create_channel(
                channel_id.clone(),
                "conn1".to_string(),
                "transfer".to_string(),
                PacketOrdering::Unordered,
            )
            .unwrap();
        manager
            .open_channel(&channel_id, format!("channel{}_counterparty", i))
            .unwrap();
    }

    let stats = manager.get_statistics();
    assert_eq!(stats.total_channels, 5);
    assert_eq!(stats.open_channels, 5);
}

#[test]
fn test_ibc_ordered_packets() {
    let manager = IBCManager::new("chain1".to_string());

    // Setup with ordered channel
    manager
        .create_client("client1".to_string(), "chain2".to_string())
        .unwrap();

    let state = ConsensusState {
        height: 1,
        timestamp: 1000,
        root: vec![1, 2, 3],
        next_validators_hash: vec![4, 5, 6],
    };
    manager.update_client("client1", 1, state).unwrap();

    manager
        .create_connection(
            "conn1".to_string(),
            "client1".to_string(),
            "client2".to_string(),
        )
        .unwrap();
    manager
        .open_connection("conn1", "conn2".to_string())
        .unwrap();

    manager
        .create_channel(
            "channel1".to_string(),
            "conn1".to_string(),
            "transfer".to_string(),
            PacketOrdering::Ordered,
        )
        .unwrap();
    manager
        .open_channel("channel1", "channel2".to_string())
        .unwrap();

    // Send multiple packets
    for i in 0..10 {
        let packet = manager
            .send_packet("channel1", vec![i], 1000, 5000)
            .unwrap();

        assert_eq!(packet.sequence, (i + 1) as u64);
    }
}

// ============================================================================
// Integration Tests - Cross-Module
// ============================================================================

#[test]
fn test_state_channels_with_batch_settlement() {
    // Scenario: Close state channel and batch settle final balances

    let channel_config = ChannelConfig::default();
    let channel_manager = StateChannelManager::new(channel_config);

    let batch_config = BatchConfig::default();
    let batch_manager = BatchSettlementManager::new(batch_config);

    // Setup state channel
    let participants = vec![
        Participant {
            address: "alice".to_string(),
            public_key: vec![1],
        },
        Participant {
            address: "bob".to_string(),
            public_key: vec![2],
        },
    ];

    channel_manager
        .open_channel("channel1".to_string(), participants)
        .unwrap();
    channel_manager.deposit("channel1", "alice", 10000).unwrap();
    channel_manager.deposit("channel1", "bob", 10000).unwrap();
    channel_manager.activate_channel("channel1").unwrap();

    // Close channel
    channel_manager.close_channel("channel1").unwrap();

    // Create batch for settlement
    batch_manager.create_batch("settlement_batch".to_string()).unwrap();

    // Add settlement transactions
    let tx1 = Transaction::new(
        "settlement_alice".to_string(),
        TransactionType::Withdrawal,
        "channel1".to_string(),
        "alice".to_string(),
        "ETH".to_string(),
        9500,
        0,
    );

    let tx2 = Transaction::new(
        "settlement_bob".to_string(),
        TransactionType::Withdrawal,
        "channel1".to_string(),
        "bob".to_string(),
        "ETH".to_string(),
        10500,
        0,
    );

    batch_manager.add_transaction(tx1).unwrap();
    batch_manager.add_transaction(tx2).unwrap();

    // Settle batch
    batch_manager
        .settle_batch("settlement_batch", "0xsettlement".to_string())
        .unwrap();

    let batch = batch_manager.get_batch("settlement_batch").unwrap();
    assert_eq!(batch.transactions.len(), 2);
}

#[test]
fn test_ibc_with_batch_settlement() {
    // Scenario: Cross-chain transfer via IBC, then batch settle on destination

    let ibc_manager = IBCManager::new("source_chain".to_string());
    let batch_manager = BatchSettlementManager::new(BatchConfig::default());

    // Setup IBC
    ibc_manager
        .create_client("client1".to_string(), "dest_chain".to_string())
        .unwrap();

    let state = ConsensusState {
        height: 1,
        timestamp: 1000,
        root: vec![1, 2, 3],
        next_validators_hash: vec![4, 5, 6],
    };
    ibc_manager.update_client("client1", 1, state).unwrap();

    ibc_manager
        .create_connection(
            "conn1".to_string(),
            "client1".to_string(),
            "client2".to_string(),
        )
        .unwrap();
    ibc_manager
        .open_connection("conn1", "conn2".to_string())
        .unwrap();

    ibc_manager
        .create_channel(
            "channel1".to_string(),
            "conn1".to_string(),
            "transfer".to_string(),
            PacketOrdering::Unordered,
        )
        .unwrap();
    ibc_manager
        .open_channel("channel1", "channel2".to_string())
        .unwrap();

    // Send IBC transfer
    let _packet = ibc_manager
        .transfer(
            "channel1",
            "utoken".to_string(),
            5000,
            "source_addr".to_string(),
            "dest_addr".to_string(),
            1000,
            5000,
        )
        .unwrap();

    // On destination chain, batch settle the received tokens
    batch_manager.create_batch("ibc_batch".to_string()).unwrap();

    let tx = Transaction::new(
        "ibc_settlement".to_string(),
        TransactionType::Deposit,
        "ibc_escrow".to_string(),
        "dest_addr".to_string(),
        "ibc/utoken".to_string(),
        5000,
        0,
    );

    batch_manager.add_transaction(tx).unwrap();
    batch_manager
        .settle_batch("ibc_batch", "0xibc_settlement".to_string())
        .unwrap();

    let batch = batch_manager.get_batch("ibc_batch").unwrap();
    assert_eq!(batch.transactions.len(), 1);
}

#[test]
fn test_performance_high_throughput() {
    // Test high-throughput scenario with all three systems

    let channel_manager = StateChannelManager::new(ChannelConfig::default());
    let batch_manager = BatchSettlementManager::new(BatchConfig::default());
    let ibc_manager = IBCManager::new("chain1".to_string());

    // Create 100 state channels
    for i in 0..100 {
        let participants = vec![
            Participant {
                address: format!("user{}", i * 2),
                public_key: vec![i as u8],
            },
            Participant {
                address: format!("user{}", i * 2 + 1),
                public_key: vec![(i + 1) as u8],
            },
        ];

        channel_manager
            .open_channel(format!("channel{}", i), participants)
            .unwrap();
    }

    // Create 1000 batch transactions
    for i in 0..1000 {
        let tx = Transaction::new(
            format!("tx{}", i),
            TransactionType::Transfer,
            format!("sender{}", i % 100),
            format!("receiver{}", (i + 1) % 100),
            "ETH".to_string(),
            100,
            1,
        );

        let _ = batch_manager.add_transaction(tx);
    }

    // Create 10 IBC connections
    for i in 0..10 {
        ibc_manager
            .create_client(format!("client{}", i), format!("chain{}", i))
            .unwrap();
    }

    let channel_stats = channel_manager.get_statistics();
    let batch_stats = batch_manager.get_statistics();
    let ibc_stats = ibc_manager.get_statistics();

    assert_eq!(channel_stats.total_channels, 100);
    assert_eq!(batch_stats.total_transactions, 1000);
    assert_eq!(ibc_stats.total_clients, 10);
}
