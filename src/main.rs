//! Bounty Challenge Server
//!
//! Rewards miners for valid GitHub issues

use std::sync::Arc;
use std::time::Duration;

use bounty_challenge::{BountyChallenge, PgStorage};
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

const SYNC_INTERVAL_SECS: u64 = 600; // 10 minutes
const STAR_SYNC_INTERVAL_SECS: u64 = 1800; // 30 minutes

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

    // Start background GitHub sync task (every 5 minutes)
    let sync_storage = storage.clone();
    tokio::spawn(async move {
        // Initial sync after 10 seconds
        tokio::time::sleep(Duration::from_secs(10)).await;
        
        let mut interval = tokio::time::interval(Duration::from_secs(SYNC_INTERVAL_SECS));
        loop {
            interval.tick().await;
            if let Err(e) = sync_all_repos(&sync_storage).await {
                error!("GitHub sync failed: {}", e);
            }
        }
    });
    info!("Background GitHub sync started (every {} seconds)", SYNC_INTERVAL_SECS);

    // Start background star sync task (every 10 minutes)
    let star_storage = storage.clone();
    tokio::spawn(async move {
        // Initial sync after 30 seconds
        tokio::time::sleep(Duration::from_secs(30)).await;
        
        let mut interval = tokio::time::interval(Duration::from_secs(STAR_SYNC_INTERVAL_SECS));
        loop {
            interval.tick().await;
            if let Err(e) = sync_all_stars(&star_storage).await {
                error!("Star sync failed: {}", e);
            }
        }
    });
    info!("Background star sync started (every {} seconds)", STAR_SYNC_INTERVAL_SECS);

    // Run our custom server with all endpoints
    bounty_challenge::server::run_server(&host, port, challenge, storage).await?;

    Ok(())
}

/// Sync all target repos from GitHub
async fn sync_all_repos(storage: &PgStorage) -> anyhow::Result<()> {
    let repos = storage.get_active_repos().await?;
    
    if repos.is_empty() {
        warn!("No target repos configured for sync");
        return Ok(());
    }

    info!("Starting GitHub sync for {} repos", repos.len());
    
    let mut total_synced = 0;
    for repo in repos {
        match bounty_challenge::server::sync_repo(storage, &repo.owner, &repo.repo).await {
            Ok(count) => {
                if count > 0 {
                    info!("Synced {} issues from {}/{}", count, repo.owner, repo.repo);
                }
                total_synced += count;
            }
            Err(e) => {
                error!("Failed to sync {}/{}: {}", repo.owner, repo.repo, e);
            }
        }
    }
    
    if total_synced > 0 {
        info!("GitHub sync complete: {} new/updated issues", total_synced);
    }
    
    Ok(())
}

/// Sync stars from all target repos
async fn sync_all_stars(storage: &PgStorage) -> anyhow::Result<()> {
    let repos = storage.get_star_target_repos().await?;
    
    if repos.is_empty() {
        warn!("No star target repos configured for sync");
        return Ok(());
    }

    info!("Starting star sync for {} repos", repos.len());
    
    let mut total_stars = 0;
    let mut new_stars = 0;
    
    for repo in repos {
        match bounty_challenge::github::get_stargazers(&repo.owner, &repo.repo).await {
            Ok(stargazers) => {
                total_stars += stargazers.len();
                
                for username in stargazers {
                    if let Ok(is_new) = storage.upsert_star(&username, &repo.owner, &repo.repo).await {
                        if is_new {
                            new_stars += 1;
                            info!("New star: @{} starred {}/{}", username, repo.owner, repo.repo);
                        }
                    }
                }
                
                if let Err(e) = storage.update_star_sync(&repo.owner, &repo.repo).await {
                    warn!("Failed to update star sync timestamp for {}/{}: {}", repo.owner, repo.repo, e);
                }
            }
            Err(e) => {
                error!("Failed to fetch stargazers for {}/{}: {}", repo.owner, repo.repo, e);
            }
        }
    }
    
    if new_stars > 0 {
        info!("Star sync complete: {} new stars (total {} stars tracked)", new_stars, total_stars);
    }
    
    Ok(())
}
