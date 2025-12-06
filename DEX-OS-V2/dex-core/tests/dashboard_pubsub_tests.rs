//! Integration test: DashboardQueryEngine publishes events to Pub-Sub.

use dex_core::dashboard_queries::{AnalyticsEvent, DashboardQueryEngine};
use dex_core::network::{MessageBroker, PubSubConfig};
use serde_json::json;
use std::collections::HashMap;
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn dashboard_engine_publishes_events() {
    let broker = MessageBroker::new(PubSubConfig::default());
    let mut subscriber = broker
        .subscribe("dashboard.analytics")
        .await
        .expect("subscriber should attach");

    let mut engine = DashboardQueryEngine::new_with_pubsub(10, 60, broker);

    let mut metadata = HashMap::new();
    metadata.insert("pair".to_string(), "ETH/USDC".to_string());

    let event = AnalyticsEvent {
        id: "e1".to_string(),
        event_type: "swap".to_string(),
        tags: vec!["dex".to_string(), "eth".to_string()],
        value: 1.5,
        timestamp: 100,
        metadata,
    };

    engine.record_event(event).expect("event recorded");

    let msg = timeout(Duration::from_millis(250), subscriber.next())
        .await
        .expect("message should arrive")
        .expect("channel should remain open");

    assert_eq!(msg.topic, "dashboard.analytics");
    assert_eq!(msg.payload["event_type"], json!("swap"));
    assert_eq!(msg.payload["metadata"]["pair"], json!("ETH/USDC"));
}
