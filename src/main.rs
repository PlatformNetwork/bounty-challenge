//! Bounty Challenge Server
//!
//! Rewards miners for valid GitHub issues in CortexLM/fabric

use std::sync::Arc;

use bounty_challenge::{BountyChallenge, BountyStorage};
use platform_challenge_sdk::server::{ChallengeServer, ServerConfig};
use tracing::info;
use tracing_subscriber::EnvFilter;

const GITHUB_OWNER: &str = "CortexLM";
const GITHUB_REPO: &str = "fabric";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    info!("Starting Bounty Challenge Server");

    // Initialize storage
    let db_path = std::env::var("BOUNTY_DB_PATH").unwrap_or_else(|_| "bounty.db".to_string());
    let storage = Arc::new(BountyStorage::new(&db_path)?);
    info!("Storage initialized at {}", db_path);

    // Create challenge
    let challenge = BountyChallenge::new(GITHUB_OWNER, GITHUB_REPO, storage);

    // Build and run server
    let config = ServerConfig::from_env();
    info!(
        "Server will listen on {}:{}",
        config.host, config.port
    );

    let server = ChallengeServer::builder(challenge)
        .config(config)
        .build();

    server.run().await?;

    Ok(())
}
