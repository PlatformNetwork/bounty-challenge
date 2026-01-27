//! Bounty Challenge Server
//!
//! Rewards miners for valid GitHub issues

use std::sync::Arc;
use std::time::Duration;

use bounty_challenge::{BountyChallenge, GitHubClient, PgStorage};
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

const SYNC_INTERVAL_SECS: u64 = 300; // 5 minutes
const STAR_SYNC_INTERVAL_SECS: u64 = 300; // 5 minutes
const HEARTBEAT_INTERVAL_SECS: u64 = 60; // 1 minute - check GitHub API health

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

    // Start background star sync task (every 5 minutes)
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

    // Start GitHub API heartbeat task (every 1 minute)
    tokio::spawn(async move {
        // Initial check after 5 seconds
        tokio::time::sleep(Duration::from_secs(5)).await;
        
        let mut interval = tokio::time::interval(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));
        loop {
            interval.tick().await;
            check_github_health().await;
        }
    });
    info!("GitHub API heartbeat started (every {} seconds)", HEARTBEAT_INTERVAL_SECS);

    // Run our custom server with all endpoints
    bounty_challenge::server::run_server(&host, port, challenge, storage).await?;

    Ok(())
}

/// Sync all target repos from GitHub
/// Uses gh CLI if available (more reliable), falls back to REST API
async fn sync_all_repos(storage: &PgStorage) -> anyhow::Result<()> {
    let repos = storage.get_active_repos().await?;
    
    if repos.is_empty() {
        warn!("No target repos configured for sync");
        return Ok(());
    }

    // Check if gh CLI is available
    let use_gh_cli = bounty_challenge::GhCli::is_available();
    if use_gh_cli {
        info!("Using gh CLI for GitHub sync (more reliable)");
    } else {
        warn!("gh CLI not available, falling back to REST API");
    }

    info!("Starting GitHub sync for {} repos", repos.len());
    
    let mut total_synced = 0;
    let mut total_valid = 0;
    let mut total_invalid = 0;
    
    for repo in repos {
        if use_gh_cli {
            // Use gh CLI (preferred)
            match bounty_challenge::sync_repo_with_gh(storage, &repo.owner, &repo.repo).await {
                Ok(result) => {
                    info!(
                        "Synced {}/{}: {} issues, {} valid, {} invalid, {} deleted",
                        repo.owner, repo.repo, 
                        result.total_fetched, result.became_valid, 
                        result.became_invalid, result.marked_deleted
                    );
                    total_synced += result.total_fetched;
                    total_valid += result.became_valid;
                    total_invalid += result.became_invalid;
                }
                Err(e) => {
                    error!("Failed to sync {}/{} with gh CLI: {}", repo.owner, repo.repo, e);
                    // Fallback to REST API on error
                    if let Ok(count) = bounty_challenge::server::sync_repo(storage, &repo.owner, &repo.repo).await {
                        info!("Fallback REST sync: {} issues from {}/{}", count, repo.owner, repo.repo);
                        total_synced += count as usize;
                    }
                }
            }
        } else {
            // Fallback to REST API
            match bounty_challenge::server::sync_repo(storage, &repo.owner, &repo.repo).await {
                Ok(count) => {
                    if count > 0 {
                        info!("Synced {} issues from {}/{}", count, repo.owner, repo.repo);
                    }
                    total_synced += count as usize;
                }
                Err(e) => {
                    error!("Failed to sync {}/{}: {}", repo.owner, repo.repo, e);
                }
            }
        }
    }
    
    // Always show totals after sync
    match storage.get_issues_stats().await {
        Ok(stats) => {
            info!(
                "GitHub sync complete: {} total issues (valid: {}, invalid: {}, pending: {}, open: {})",
                stats.total, stats.valid, stats.invalid, stats.pending, stats.open
            );
        }
        Err(e) => {
            warn!("Could not get stats: {}", e);
        }
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

/// Check GitHub API health and rate limit status (heartbeat)
async fn check_github_health() {
    // Use a dummy client just for rate limit check (owner/repo don't matter for /rate_limit)
    let client = GitHubClient::new("platformnetwork", "bounty-challenge");
    
    match client.check_rate_limit().await {
        Ok(rate_info) => {
            let status = if rate_info.is_low() { "LOW" } else { "OK" };
            info!(
                "GitHub API heartbeat: {} - {}/{} requests remaining (resets in {} seconds)",
                status,
                rate_info.remaining,
                rate_info.limit,
                rate_info.seconds_until_reset()
            );
            
            if !client.is_authenticated() {
                warn!("GitHub API: Running WITHOUT authentication - set EXTRA_GITHUB_TOKEN or GITHUB_TOKEN for higher limits");
            }
            
            if rate_info.is_low() {
                warn!(
                    "GitHub rate limit critically low! {} remaining. Syncs may fail until reset.",
                    rate_info.remaining
                );
            }
        }
        Err(e) => {
            error!("GitHub API heartbeat FAILED: {} - API may be unavailable", e);
        }
    }
}
