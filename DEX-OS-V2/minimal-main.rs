//! Minimal main entry point for the DEX-OS API server

use warp::Filter;

#[tokio::main]
async fn main() {
    // Health endpoint
    let health = warp::path("health")
        .and(warp::get())
        .map(|| {
            let payload = serde_json::json!({ "status": "ok" });
            warp::reply::with_status(warp::reply::json(&payload), warp::http::StatusCode::OK)
        });

    println!("Starting minimal DEX-OS API server on port 3030");
    warp::serve(health)
        .run(([0, 0, 0, 0], 3030))
        .await;
}