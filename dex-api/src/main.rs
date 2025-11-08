//! Main entry point for the DEX-OS API server

use dex_api::{auth::AuthManager, challenge::ChallengeStore, routes, ApiState, Config};
use dex_core::orderbook::OrderBook;
use dex_db::DatabaseManager;
use secrecy::ExposeSecret;
use std::sync::{atomic::AtomicU64, Arc};
use tokio::sync::{broadcast, RwLock};

#[tokio::main]
async fn main() {
    if let Err(err) = bootstrap().await {
        eprintln!("Failed to start DEX-OS API server: {}", err);
        std::process::exit(1);
    }
}

async fn bootstrap() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;

    let database = Arc::new(DatabaseManager::connect(config.database_url.expose_secret()).await?);
    database.initialize().await?;

    let auth = Arc::new(AuthManager::new(
        &config.jwt_secret,
        config.jwt_issuer.clone(),
    ));
    let wallet_challenges = Arc::new(ChallengeStore::new(config.wallet_challenge_ttl_seconds));
    let (market_tx, _) = broadcast::channel(64);

    let state = ApiState {
        orderbook: Arc::new(RwLock::new(OrderBook::new())),
        order_id_counter: Arc::new(AtomicU64::new(1)),
        trade_id_counter: Arc::new(AtomicU64::new(1)),
        database,
        auth,
        config: config.clone(),
        wallet_challenges,
        market_tx,
    };

    let routes = routes(state);

    println!("Starting DEX-OS API server on port {}", config.server_port);
    warp::serve(routes)
        .run(([0, 0, 0, 0], config.server_port))
        .await;

    Ok(())
}
