use clap::Parser;
use dex_api::{
    auth::{clamp_ttl, AuthManager},
    Config,
};

/// Utility binary that mints JWTs using the same secret/config as the API server.
#[derive(Debug, Parser)]
#[command(author, version, about = "Issue short-lived JWTs for DEX-OS traders", long_about = None)]
struct Args {
    /// Trader ID (JWT subject)
    #[arg(long, required = true)]
    trader_id: String,
    /// Optional TTL override in seconds
    #[arg(long)]
    ttl_seconds: Option<u64>,
    /// Optional audience claim
    #[arg(long)]
    audience: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config = Config::from_env()?;
    let auth = AuthManager::new(
        &config.jwt_secret,
        config.jwt_issuer.clone(),
        config.jwt_allowed_audiences.clone(),
        config.jwt_require_audience,
    );
    let ttl = clamp_ttl(
        args.ttl_seconds,
        config.jwt_default_ttl_seconds,
        config.jwt_max_ttl_seconds,
    );
    let issued = auth.issue_token(args.trader_id, ttl, args.audience)?;
    println!("token={}", issued.token);
    println!("expires_at={}", issued.expires_at);
    Ok(())
}
