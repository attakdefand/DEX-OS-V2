mod config;
mod executor;
mod loader;
mod results;

use axum::{extract::State, http::StatusCode, routing::{get, post}, Json, Router};
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let state = Arc::new(config::AppState::new());

    let app = Router::new()
        .route("/health", get(health))
        .route("/load-csv", post(load_csv))
        .route("/run-tests", post(executor::run_all_tests))
        .with_state(state.clone());

    tracing::info!(
        "MCP gold/diamond server listening on http://localhost:9000 using {}",
        state.reference_root.display()
    );

    axum::Server::bind(&"0.0.0.0:9000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .expect("server failed");
}

async fn health() -> &'static str {
    "ok"
}

async fn load_csv(
    State(state): State<Arc<config::AppState>>,
) -> Result<Json<loader::LoadSummary>, (StatusCode, String)> {
    loader::load_all(state)
        .await
        .map(Json)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}
