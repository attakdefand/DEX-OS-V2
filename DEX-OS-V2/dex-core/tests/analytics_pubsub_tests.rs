//! Integration test: AnalyticsService emits Pub-Sub events for volume updates.

use dex_core::analytics::{AnalyticsConfig, AnalyticsService};
use dex_core::network::{MessageBroker, PubSubConfig};
use serde_json::json;
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn analytics_emits_pubsub_events() {
    let broker = MessageBroker::new(PubSubConfig::default());
    let mut subscriber = broker
        .subscribe("analytics.volume")
        .await
        .expect("subscriber should attach");

    let service = AnalyticsService::with_pubsub(AnalyticsConfig::default(), broker);
    service
        .record_trade_volume("ETH".into(), "USDC".into(), 42)
        .expect("volume recorded");

    let msg = timeout(Duration::from_millis(250), subscriber.next())
        .await
        .expect("message should arrive")
        .expect("channel should remain open");

    assert_eq!(msg.topic, "analytics.volume");
    assert_eq!(msg.payload["base_token"], json!("ETH"));
    assert_eq!(msg.payload["quote_token"], json!("USDC"));
    assert_eq!(msg.payload["volume"], json!(42));
}
