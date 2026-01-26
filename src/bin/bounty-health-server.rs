//! Minimal health-only server for validator mode
//!
//! When DATABASE_URL is not set, this lightweight server provides
//! only /health and /get_weights endpoints for platform orchestration.

use axum::{routing::get, Json, Router};
use serde_json::json;
use std::net::{IpAddr, SocketAddr};
use std::time::Instant;
use tokio::sync::OnceCell;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

static START_TIME: OnceCell<Instant> = OnceCell::const_new();

/// Validates that a string is a valid hostname or IP address for server binding.
fn validate_server_host(s: &str) -> Result<String, String> {
    let s = s.trim();

    // Reject URLs
    if s.contains("://") {
        return Err(format!(
            "Invalid host '{}': URLs are not allowed. Use a hostname or IP address (e.g., '0.0.0.0', 'localhost').",
            s
        ));
    }

    // Reject embedded ports (except for valid IPv6)
    if s.contains(':') && !s.parse::<IpAddr>().is_ok() {
        return Err(format!(
            "Invalid host '{}': Ports should be specified via CHALLENGE_PORT. Use just the hostname or IP.",
            s
        ));
    }

    // Try parsing as IP address first
    if s.parse::<IpAddr>().is_ok() {
        return Ok(s.to_string());
    }

    // Validate as hostname per RFC 1123
    if s.len() > 253 {
        return Err(format!(
            "Invalid host '{}': Hostname exceeds maximum length of 253 characters.",
            s
        ));
    }

    if s.is_empty() {
        return Err("Invalid host: Hostname cannot be empty.".to_string());
    }

    for label in s.split('.') {
        if label.is_empty() || label.len() > 63 {
            return Err(format!(
                "Invalid host '{}': Each hostname label must be 1-63 characters.",
                s
            ));
        }
        if label.starts_with('-') || label.ends_with('-') {
            return Err(format!(
                "Invalid host '{}': Hostname labels cannot start or end with a hyphen.",
                s
            ));
        }
        if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(format!(
                "Invalid host '{}': Hostname contains invalid characters. Use only letters, numbers, and hyphens.",
                s
            ));
        }
    }

    Ok(s.to_string())
}

async fn health() -> Json<serde_json::Value> {
    let uptime = START_TIME
        .get()
        .map(|t| t.elapsed().as_secs())
        .unwrap_or(0);

    Json(json!({
        "healthy": true,
        "load": 0.0,
        "pending": 0,
        "uptime_secs": uptime,
        "version": env!("CARGO_PKG_VERSION"),
        "challenge_id": "bounty-challenge",
        "mode": "validator"
    }))
}

async fn get_weights() -> Json<serde_json::Value> {
    // In validator mode without DB, return empty weights in term-challenge format
    // Platform will use existing chain weights
    Json(json!({
        "epoch": 0,
        "weights": []
    }))
}

async fn config() -> Json<serde_json::Value> {
    Json(json!({
        "challenge_id": "bounty-challenge",
        "mode": "validator",
        "database": false
    }))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Record start time
    START_TIME.set(Instant::now()).ok();

    let host_raw = std::env::var("CHALLENGE_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let host = match validate_server_host(&host_raw) {
        Ok(h) => h,
        Err(e) => {
            error!("{}", e);
            std::process::exit(1);
        }
    };
    let port: u16 = std::env::var("CHALLENGE_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let app = Router::new()
        .route("/health", get(health))
        .route("/get_weights", get(get_weights))
        .route("/config", get(config));

    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    info!("Health-only server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
