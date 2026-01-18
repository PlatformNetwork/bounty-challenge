//! Server command - run the challenge server

use std::sync::Arc;

use anyhow::Result;
use bounty_challenge::{BountyChallenge, PgStorage};
use tracing::info;

pub async fn run(host: &str, port: u16, database_url: &str) -> Result<()> {
    info!("Starting Bounty Challenge server");

    // Initialize PostgreSQL storage
    let storage = Arc::new(PgStorage::new(database_url).await?);
    info!("PostgreSQL database initialized");

    // Create challenge
    let challenge = Arc::new(BountyChallenge::new_with_storage(storage.clone()));

    // Run server
    bounty_challenge::server::run_server(host, port, challenge, storage).await
}
