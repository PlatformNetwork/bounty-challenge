//! Bounty Challenge Server
//!
//! Rewards miners for valid GitHub issues

use std::sync::Arc;

use bounty_challenge::{BountyChallenge, PgStorage};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    info!("Starting Bounty Challenge Server");

    // Initialize PostgreSQL storage (required)
    let database_url = std::env::var("DATABASE_URL").map_err(|_| {
        error!("DATABASE_URL environment variable is required");
        anyhow::anyhow!("DATABASE_URL not set")
    })?;
    
    let storage = Arc::new(PgStorage::new(&database_url).await?);
    info!("PostgreSQL storage initialized");

    // Create challenge
    let challenge = Arc::new(BountyChallenge::new_with_storage(storage.clone()));

    // Get server config from environment
    let host = std::env::var("CHALLENGE_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("CHALLENGE_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    // Run our custom server with all endpoints
    bounty_challenge::server::run_server(&host, port, challenge, storage).await?;

    Ok(())
}
