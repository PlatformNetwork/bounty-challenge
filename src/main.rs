//! Bounty Challenge Server
//!
//! Rewards miners for valid GitHub issues

use std::sync::Arc;

use bounty_challenge::{BountyChallenge, PgStorage};
use platform_challenge_sdk::server::{ChallengeServer, ServerConfig};
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
    let challenge = BountyChallenge::new_with_storage(storage.clone());

    // Build and run server
    let config = ServerConfig::from_env();
    info!("Server will listen on {}:{}", config.host, config.port);

    let server = ChallengeServer::builder(challenge).config(config).build();

    server.run().await?;

    Ok(())
}
