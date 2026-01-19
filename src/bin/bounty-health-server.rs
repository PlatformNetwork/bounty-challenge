//! Minimal health-only server for validator mode
//! 
//! When DATABASE_URL is not set, this lightweight server provides
//! only /health and /get_weights endpoints for platform orchestration.

use axum::{routing::get, Json, Router};
use serde_json::json;
use std::net::SocketAddr;
use std::time::Instant;
use tokio::sync::OnceCell;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

static START_TIME: OnceCell<Instant> = OnceCell::const_new();

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

    let host = std::env::var("CHALLENGE_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
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
