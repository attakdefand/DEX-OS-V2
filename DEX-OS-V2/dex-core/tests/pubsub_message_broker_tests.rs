//! Tests for the Pub-Sub message broker implementation
//!
//! Validates the Priority 3 feature:
//! - Distributed Systems,Distributed Systems,Distributed Systems,Pub-Sub,Message Brokers,Medium

use dex_core::network::{MessageBroker, PubSubConfig, PubSubError};
use serde_json::json;
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn publish_delivers_to_single_subscriber() {
    let broker = MessageBroker::new(PubSubConfig::default());
    let mut sub = broker
        .subscribe("trades")
        .await
        .expect("should subscribe to trades");

    broker
        .publish("trades", json!({ "order_id": 1, "side": "buy" }))
        .await
        .expect("publish should succeed");

    let message = timeout(Duration::from_millis(200), sub.next())
        .await
        .expect("message should arrive")
        .expect("subscription should be open");

    assert_eq!(message.topic, "trades");
    assert_eq!(message.payload["order_id"], json!(1));
    assert_eq!(message.payload["side"], json!("buy"));
}

#[tokio::test]
async fn publishes_to_multiple_subscribers() {
    let broker = MessageBroker::new(PubSubConfig::default());
    let mut sub_a = broker.subscribe("prices").await.unwrap();
    let mut sub_b = broker.subscribe("prices").await.unwrap();

    broker
        .publish("prices", json!({ "pair": "ETH/USDC", "price": 2500 }))
        .await
        .expect("publish should succeed");

    let msg_a = timeout(Duration::from_millis(200), sub_a.next())
        .await
        .expect("subscriber A should receive")
        .expect("channel should stay open");
    let msg_b = timeout(Duration::from_millis(200), sub_b.next())
        .await
        .expect("subscriber B should receive")
        .expect("channel should stay open");

    assert_eq!(msg_a.payload["price"], json!(2500));
    assert_eq!(msg_b.payload["pair"], json!("ETH/USDC"));
}

#[tokio::test]
async fn stats_capture_publishes_and_drops() {
    let broker = MessageBroker::new(PubSubConfig {
        channel_capacity: 8,
        max_topics: 4,
    });

    let err = broker
        .publish("orphans", json!({ "note": "nobody is listening" }))
        .await;
    assert_eq!(err, Err(PubSubError::NoSubscribers("orphans".to_string())));

    {
        let stats = broker
            .topic_stats("orphans")
            .await
            .expect("stats should exist after publish");
        assert_eq!(stats.dropped, 1);
        assert_eq!(stats.published, 0);
    }

    let mut sub = broker.subscribe("orphans").await.unwrap();
    broker
        .publish("orphans", json!({ "note": "hello subscriber" }))
        .await
        .expect("publish should succeed with subscriber");

    let _ = timeout(Duration::from_millis(200), sub.next())
        .await
        .expect("subscriber should receive message")
        .expect("channel should stay open");

    {
        let stats = broker.topic_stats("orphans").await.unwrap();
        assert_eq!(stats.published, 1);
        assert_eq!(stats.dropped, 1);
        assert_eq!(stats.subscribers, 1);
    }

    drop(sub);
    let stats = broker.topic_stats("orphans").await.unwrap();
    assert_eq!(stats.subscribers, 0);
}
