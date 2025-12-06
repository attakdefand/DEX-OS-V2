//! Tests for the real-time updates functionality

use dex_api::{
    auth::AuthManager,
    challenge::ChallengeStore,
    routes, 
    ApiState, 
    Config,
};
use dex_core::{
    governance::GlobalDAO,
    orderbook::OrderBook,
};
use dex_db::DatabaseManager;
use secrecy::SecretString;
use std::sync::{atomic::AtomicU64, Arc};
use tokio::sync::{broadcast, RwLock};

const TEST_DB_URL: &str = "postgres://user:password@localhost/test";
const TEST_SECRET: &str = "super-secret-signing-key";

#[tokio::test]
async fn test_websocket_depth_endpoint() {
    let state = test_state();
    let routes = routes(state);

    // Test that the WebSocket endpoint exists and can be upgraded
    let response = warp::test::request()
        .method("GET")
        .path("/ws/depth")
        .header("connection", "upgrade")
        .header("upgrade", "websocket")
        .header("sec-websocket-version", "13")
        .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
        .reply(&routes)
        .await;

    // The WebSocket upgrade should be accepted
    assert_eq!(response.status(), warp::http::StatusCode::SWITCHING_PROTOCOLS);
}

#[tokio::test]
async fn test_websocket_depth_with_levels_parameter() {
    let state = test_state();
    let routes = routes(state);

    // Test that the WebSocket endpoint accepts levels parameter
    let response = warp::test::request()
        .method("GET")
        .path("/ws/depth?levels=5")
        .header("connection", "upgrade")
        .header("upgrade", "websocket")
        .header("sec-websocket-version", "13")
        .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
        .reply(&routes)
        .await;

    // The WebSocket upgrade should be accepted
    assert_eq!(response.status(), warp::http::StatusCode::SWITCHING_PROTOCOLS);
}

#[tokio::test]
async fn test_rest_depth_endpoint() {
    let state = test_state();
    let routes = routes(state);

    // Test the REST depth endpoint
    let response = warp::test::request()
        .method("GET")
        .path("/orderbook/depth")
        .reply(&routes)
        .await;

    assert_eq!(response.status(), warp::http::StatusCode::OK);
    
    let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
    assert!(body.get("bids").is_some());
    assert!(body.get("asks").is_some());
    assert!(body.get("timestamp").is_some());
}

#[tokio::test]
async fn test_rest_depth_with_levels_parameter() {
    let state = test_state();
    let routes = routes(state);

    // Test the REST depth endpoint with levels parameter
    let response = warp::test::request()
        .method("GET")
        .path("/orderbook/depth?levels=5")
        .reply(&routes)
        .await;

    assert_eq!(response.status(), warp::http::StatusCode::OK);
    
    let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
    assert!(body.get("bids").is_some());
    assert!(body.get("asks").is_some());
    assert!(body.get("timestamp").is_some());
}

fn test_state() -> ApiState {
    let secret = SecretString::from(TEST_SECRET.to_string());
    let auth = Arc::new(AuthManager::new(
        &secret,
        "test-issuer",
        Vec::new(),
        false,
    ));
    let config = Config {
        database_url: SecretString::from(TEST_DB_URL.to_string()),
        jwt_secret: secret.clone(),
        jwt_issuer: "test-issuer".into(),
        jwt_default_ttl_seconds: 900,
        jwt_max_ttl_seconds: 3600,
        wallet_challenge_ttl_seconds: 300,
        trader_secrets: std::collections::HashMap::new(),
        server_port: 3030,
        cors_allowed_origins: Vec::new(),
        jwt_require_audience: false,
        jwt_allowed_audiences: Vec::new(),
        dao_members: Vec::new(),
    };
    let database = Arc::new(
        DatabaseManager::connect_lazy(TEST_DB_URL).expect("lazy db pool"),
    );
    let (market_tx, _) = broadcast::channel(16);
    let dao = Arc::new(RwLock::new(GlobalDAO::new()));

    ApiState {
        orderbook: Arc::new(RwLock::new(OrderBook::new())),
        order_id_counter: Arc::new(AtomicU64::new(1)),
        trade_id_counter: Arc::new(AtomicU64::new(1)),
        database,
        auth,
        config,
        wallet_challenges: Arc::new(ChallengeStore::new(300)),
        market_tx,
        dao,
    }
}