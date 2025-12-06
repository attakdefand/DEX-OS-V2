//! Tests for the session management functionality

use dex_api::{
    auth::{AuthManager, Claims},
    challenge::ChallengeStore,
    routes, ApiState, Config,
};
use dex_core::{
    governance::GlobalDAO,
    orderbook::OrderBook,
    rate_limiting::{RateLimiter, RateLimitConfig},
};
use dex_db::DatabaseManager;
use jsonwebtoken;
use secrecy::{ExposeSecret, SecretString};
use serde_json;
use std::sync::{atomic::AtomicU64, Arc};
use tokio::sync::{broadcast, RwLock};
use warp;

const TEST_DB_URL: &str = "postgres://user:password@localhost/test";
const TEST_SECRET: &str = "super-secret-signing-key";

#[tokio::test]
async fn test_create_session_endpoint() {
    let state = test_state();
    let routes = routes(state);

    let create_session_request = serde_json::json!({
        "trader_id": "alice",
        "password": "password123"
    });

    let response = warp::test::request()
        .method("POST")
        .path("/sessions")
        .json(&create_session_request)
        .reply(&routes)
        .await;

    assert_eq!(response.status(), warp::http::StatusCode::CREATED);
    
    let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
    assert!(body.get("session_token").is_some());
    assert!(body.get("expires_at").is_some());
}

#[tokio::test]
async fn test_validate_session_endpoint() {
    let state = test_state();
    let routes = routes(state);

    let validate_session_request = serde_json::json!({
        "session_token": "sess_alice_123456789"
    });

    let response = warp::test::request()
        .method("POST")
        .path("/sessions/validate")
        .json(&validate_session_request)
        .reply(&routes)
        .await;

    assert_eq!(response.status(), warp::http::StatusCode::OK);
    
    let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
    assert!(body.get("valid").is_some());
}

#[tokio::test]
async fn test_invalidate_session_endpoint() {
    let state = test_state();
    let routes = routes(state);

    let invalidate_session_request = serde_json::json!({
        "session_token": "sess_alice_123456789"
    });

    let response = warp::test::request()
        .method("POST")
        .path("/sessions/invalidate")
        .json(&invalidate_session_request)
        .reply(&routes)
        .await;

    assert_eq!(response.status(), warp::http::StatusCode::OK);
    
    let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
    assert_eq!(body.get("success").and_then(|v| v.as_bool()), Some(true));
}

#[tokio::test]
async fn test_list_sessions_endpoint() {
    let state = test_state();
    let routes = routes(state);
    
    // First create a JWT token for authentication
    let secret = SecretString::from(TEST_SECRET.to_string());
    let token = build_token(&secret, 300);

    let response = warp::test::request()
        .method("GET")
        .path("/sessions")
        .header("authorization", format!("Bearer {}", token))
        .reply(&routes)
        .await;

    assert_eq!(response.status(), warp::http::StatusCode::OK);
}

fn test_state() -> ApiState {
    let secret = SecretString::from(TEST_SECRET.to_string());
    let auth = Arc::new(AuthManager::new(
        &secret,
        "test-issuer",
        Vec::new(),
        false,
    ));
    let mut trader_secrets = std::collections::HashMap::new();
    trader_secrets.insert(
        "alice".to_string(),
        SecretString::from("password123".to_string()),
    );
    let config = Config {
        database_url: SecretString::from(TEST_DB_URL.to_string()),
        jwt_secret: secret.clone(),
        jwt_issuer: "test-issuer".into(),
        jwt_default_ttl_seconds: 900,
        jwt_max_ttl_seconds: 3600,
        wallet_challenge_ttl_seconds: 300,
        trader_secrets,
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

fn build_token(secret: &SecretString, offset_seconds: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("time went backwards")
        .as_secs() as i64;
    // Prevent negative exp wrapping to a huge usize
    let exp = (now + offset_seconds).max(0) as u64 as usize;
    let claims = Claims {
        sub: "alice".into(),
        exp,
        aud: None,
        iss: Some("test-issuer".into()),
        iat: Some(now as usize),
    };
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.expose_secret().as_bytes()),
    )
    .expect("token encoding")
}
