//! Server command - run the challenge server

use std::sync::Arc;

use anyhow::Result;
use bounty_challenge::{BountyChallenge, BountyStorage};
use tracing::info;

const GITHUB_OWNER: &str = "CortexLM";
const GITHUB_REPO: &str = "fabric";

pub async fn run(host: &str, port: u16, db_path: &str) -> Result<()> {
    info!("Starting Bounty Challenge server");

    // Initialize storage
    let storage = Arc::new(BountyStorage::new(db_path)?);
    info!("Database initialized at {}", db_path);

    // Create challenge
    let challenge = Arc::new(BountyChallenge::new(GITHUB_OWNER, GITHUB_REPO, storage.clone()));

    // Run server
    bounty_challenge::server::run_server(host, port, challenge, storage).await
}
