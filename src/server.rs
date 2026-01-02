//! Bounty Challenge Server
//!
//! HTTP server for challenge endpoints.

use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use tower_http::cors::CorsLayer;
use tracing::{error, info};

use crate::challenge::BountyChallenge;
use crate::storage::BountyStorage;
use platform_challenge_sdk::server::{
    EvaluationRequest, EvaluationResponse, HealthResponse, ValidationRequest, ValidationResponse,
    ServerChallenge,
};

pub struct AppState {
    pub challenge: Arc<BountyChallenge>,
    pub storage: Arc<BountyStorage>,
    pub started_at: std::time::Instant,
}

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/config", get(config_handler))
        .route("/evaluate", post(evaluate_handler))
        .route("/validate", post(validate_handler))
        .route("/leaderboard", get(leaderboard_handler))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn health_handler(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    Json(HealthResponse {
        healthy: true,
        load: 0.0,
        pending: 0,
        uptime_secs: state.started_at.elapsed().as_secs(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        challenge_id: "bounty-challenge".to_string(),
    })
}

async fn config_handler(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(serde_json::to_value(state.challenge.config()).unwrap())
}

async fn evaluate_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<EvaluationRequest>,
) -> (StatusCode, Json<EvaluationResponse>) {
    let request_id = request.request_id.clone();
    let start = std::time::Instant::now();

    match state.challenge.evaluate(request).await {
        Ok(mut response) => {
            response.execution_time_ms = start.elapsed().as_millis() as i64;
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            error!("Evaluation error: {}", e);
            let response = EvaluationResponse::error(&request_id, e.to_string())
                .with_time(start.elapsed().as_millis() as i64);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}

async fn validate_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ValidationRequest>,
) -> Json<ValidationResponse> {
    match state.challenge.validate(request).await {
        Ok(response) => Json(response),
        Err(e) => Json(ValidationResponse {
            valid: false,
            errors: vec![e.to_string()],
            warnings: vec![],
        }),
    }
}

async fn leaderboard_handler(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    match state.challenge.get_leaderboard() {
        Ok(lb) => Json(serde_json::json!({ "leaderboard": lb })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

/// Run the server
pub async fn run_server(
    host: &str,
    port: u16,
    challenge: Arc<BountyChallenge>,
    storage: Arc<BountyStorage>,
) -> anyhow::Result<()> {
    let state = Arc::new(AppState {
        challenge,
        storage,
        started_at: std::time::Instant::now(),
    });

    let app = create_router(state);
    let addr = format!("{}:{}", host, port);
    
    info!("Starting Bounty Challenge server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
