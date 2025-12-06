//! Integration tests for CQRS command/query separation.
//!
//! Validates Priority 3 feature:
//! "Distributed Systems,Distributed Systems,Distributed Systems,CQRS,Command/Query Separation,Medium"

use dex_core::cqrs::{CqrsCommand, CqrsEngine, CqrsError, OrderStatus};
use dex_core::types::{OrderSide, TradingPair};
use std::sync::Arc;
use std::thread;

fn sample_pair() -> TradingPair {
    TradingPair {
        base: "SOL".to_string(),
        quote: "USDC".to_string(),
    }
}

#[test]
fn cqrs_handles_parallel_commands_and_queries() {
    let engine = Arc::new(CqrsEngine::new());
    let pair = sample_pair();

    engine
        .execute_command(CqrsCommand::RegisterAccount {
            account_id: "eve".to_string(),
        })
        .unwrap();

    let mut handles = vec![];
    for order_id in 0u64..5 {
        let engine_clone = engine.clone();
        let pair_clone = pair.clone();
        handles.push(thread::spawn(move || {
            engine_clone
                .execute_command(CqrsCommand::PlaceOrder {
                    order_id,
                    account_id: "eve".to_string(),
                    pair: pair_clone.clone(),
                    side: OrderSide::Buy,
                    quantity: 5,
                })
                .unwrap();
            engine_clone
                .execute_command(CqrsCommand::RecordFill {
                    order_id,
                    quantity: 2,
                    price: 1000 + order_id,
                })
                .unwrap();
        }));
    }

    for handle in handles {
        handle.join().expect("thread should complete");
    }

    // Five open orders remain with 3 units each after partial fills.
    let open_orders = engine.query_open_orders("eve");
    assert_eq!(open_orders.len(), 5);
    let total_remaining: u64 = open_orders.iter().map(|o| o.open_quantity).sum();
    assert_eq!(total_remaining, 15);
    assert_eq!(engine.query_volume_for_pair(&pair), 10);

    // Rebuild the projection and confirm data remains consistent.
    engine.rebuild_read_model_from_events();
    let rebuilt = engine.query_open_orders("eve");
    let rebuilt_remaining: u64 = rebuilt.iter().map(|o| o.open_quantity).sum();
    assert_eq!(rebuilt_remaining, 15);
}

#[test]
fn cqrs_event_stream_is_sequenced_and_replayable() {
    let engine = CqrsEngine::new();
    let pair = sample_pair();

    engine
        .execute_command(CqrsCommand::RegisterAccount {
            account_id: "frank".to_string(),
        })
        .unwrap();
    engine
        .execute_command(CqrsCommand::PlaceOrder {
            order_id: 88,
            account_id: "frank".to_string(),
            pair: pair.clone(),
            side: OrderSide::Sell,
            quantity: 6,
        })
        .unwrap();
    engine
        .execute_command(CqrsCommand::RecordFill {
            order_id: 88,
            quantity: 3,
            price: 25_000,
        })
        .unwrap();

    let stream = engine.event_stream();
    let versions: Vec<u64> = stream.iter().map(|e| e.version).collect();
    assert_eq!(versions, vec![1, 2, 3]);

    // Force a rebuild and validate the order view remains accurate.
    engine.rebuild_read_model_from_events();
    let order = engine.query_order(88).expect("order should exist");
    assert_eq!(order.status, OrderStatus::Open);
    assert_eq!(order.open_quantity, 3);
    assert_eq!(order.filled_quantity, 3);
    assert_eq!(engine.query_volume_for_pair(&pair), 3);
    assert_eq!(engine.current_version(), 3);

    // Attempting an invalid fill after the rebuild still fails validation.
    let err = engine
        .execute_command(CqrsCommand::RecordFill {
            order_id: 88,
            quantity: 10,
            price: 26_000,
        })
        .unwrap_err();
    assert!(matches!(err, CqrsError::InvalidFill { .. }));
}
