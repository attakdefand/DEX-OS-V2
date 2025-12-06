//! Main entry point for the DEX-OS API server

use dex_api::{auth::AuthManager, challenge::ChallengeStore, routes, ApiState, Config};
use dex_core::{
    governance::GlobalDAO,
    orderbook::OrderBook,
    rate_limiting::{RateLimiter, RateLimitConfig},
};
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

    // Create a mock database manager for testing purposes
    let database = Arc::new(create_mock_database_manager());

    let auth = Arc::new(AuthManager::new(
        &config.jwt_secret,
        config.jwt_issuer.clone(),
        config.jwt_allowed_audiences.clone(),
        config.jwt_require_audience,
    ));
    let wallet_challenges = Arc::new(ChallengeStore::new(config.wallet_challenge_ttl_seconds));
    let (market_tx, _) = broadcast::channel(64);
    let dao = Arc::new(RwLock::new(GlobalDAO::new()));
    {
        let mut dao_lock = dao.write().await;
        for member in &config.dao_members {
            dao_lock.add_member(
                member.trader_id.clone(),
                member.voting_power,
                member.is_council_member,
            );
            if member.is_council_member {
                dao_lock.add_emergency_council_member(member.trader_id.clone());
            }
        }
        if dao_lock.member_count() == 0 {
            let default_trader = "alice".to_string();
            dao_lock.add_member(default_trader.clone(), 1_000, true);
            dao_lock.add_emergency_council_member(default_trader);
        }
    }

    let state = ApiState {
        orderbook: Arc::new(RwLock::new(OrderBook::new())),
        order_id_counter: Arc::new(AtomicU64::new(1)),
        trade_id_counter: Arc::new(AtomicU64::new(1)),
        database,
        auth,
        config: config.clone(),
        wallet_challenges,
        market_tx,
        dao,
    };

    let routes = routes(state);

    println!("Starting DEX-OS API server on port {}", config.server_port);
    warp::serve(routes)
        .run(([0, 0, 0, 0], config.server_port))
        .await;

    Ok(())
}

/// Create a mock database manager for testing purposes
fn create_mock_database_manager() -> DatabaseManager {
    // Create a mock database manager using the connect_lazy function
    // This won't actually connect to a database but will create a valid DatabaseManager instance
    DatabaseManager::connect_lazy("postgres://user:password@localhost:5432/dummy")
        .expect("Failed to create mock database manager")
}