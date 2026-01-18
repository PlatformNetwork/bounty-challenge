//! Bounty Challenge Server
//!
//! HTTP server for challenge endpoints.

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use tracing::{error, info};

use crate::challenge::BountyChallenge;
use crate::storage::BountyStorage;
use platform_challenge_sdk::server::{
    EvaluationRequest, EvaluationResponse, HealthResponse, ServerChallenge, ValidationRequest,
    ValidationResponse,
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
        .route("/get_weights", get(get_weights_handler))
        // Bridge API endpoints (for CLI registration)
        .route("/register", post(register_handler))
        .route("/status/:hotkey", get(status_handler))
        .route("/stats", get(stats_handler))
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

// ============================================================================
// GET /get_weights - Platform-compatible weight calculation
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct GetWeightsQuery {
    pub epoch: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct MinerWeight {
    pub miner_hotkey: String,
    pub weight: f64,
}

#[derive(Debug, Serialize)]
pub struct GetWeightsResponse {
    pub weights: Vec<MinerWeight>,
    pub epoch: u64,
    pub challenge_id: String,
    pub total_miners: usize,
}

/// Calculate score from valid issues count (logarithmic scaling)
fn calculate_weight(valid_issues: u32) -> f64 {
    // Logarithmic scoring: score = log2(1 + valid_issues) / 10
    ((1.0 + valid_issues as f64).ln() / std::f64::consts::LN_2) / 10.0
}

async fn get_weights_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetWeightsQuery>,
) -> Json<GetWeightsResponse> {
    // Get current epoch (use provided or estimate from time)
    let epoch = query.epoch.unwrap_or_else(|| {
        // Estimate epoch from current time (12 second blocks on Bittensor)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now / 12
    });

    // Get all miner scores from storage
    let scores = match state.storage.get_all_scores() {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to get scores: {}", e);
            return Json(GetWeightsResponse {
                weights: vec![],
                epoch,
                challenge_id: "bounty-challenge".to_string(),
                total_miners: 0,
            });
        }
    };

    // Calculate weights for each miner
    let mut weights: Vec<MinerWeight> = scores
        .iter()
        .map(|s| MinerWeight {
            miner_hotkey: s.miner_hotkey.clone(),
            weight: calculate_weight(s.valid_issues_count),
        })
        .collect();

    // Normalize weights to sum to 1.0
    let total_weight: f64 = weights.iter().map(|w| w.weight).sum();
    if total_weight > 0.0 {
        for w in &mut weights {
            w.weight /= total_weight;
        }
    }

    // Sort by weight descending
    weights.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());

    let total_miners = weights.len();

    info!(
        "Returning weights for {} miners at epoch {}",
        total_miners, epoch
    );

    Json(GetWeightsResponse {
        weights,
        epoch,
        challenge_id: "bounty-challenge".to_string(),
        total_miners,
    })
}

// ============================================================================
// POST /register - Register GitHub username with hotkey (Bridge API)
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub hotkey: String,
    pub github_username: String,
    pub signature: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub success: bool,
    pub message: Option<String>,
    pub error: Option<String>,
}

async fn register_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RegisterRequest>,
) -> Json<RegisterResponse> {
    // Validate timestamp (must be within 5 minutes)
    let now = chrono::Utc::now().timestamp();
    if (now - request.timestamp).abs() > 300 {
        return Json(RegisterResponse {
            success: false,
            message: None,
            error: Some("Timestamp expired. Please try again.".to_string()),
        });
    }

    // Verify signature
    let message = format!(
        "register_github:{}:{}",
        request.github_username.to_lowercase(),
        request.timestamp
    );

    if !crate::auth::verify_signature(&request.hotkey, &message, &request.signature) {
        return Json(RegisterResponse {
            success: false,
            message: None,
            error: Some("Invalid signature. Make sure you're using the correct key.".to_string()),
        });
    }

    // Register in storage
    match state
        .storage
        .register_miner(&request.hotkey, &request.github_username)
    {
        Ok(()) => {
            info!(
                "Registered GitHub user @{} with hotkey {}",
                request.github_username,
                &request.hotkey[..16.min(request.hotkey.len())]
            );
            Json(RegisterResponse {
                success: true,
                message: Some(format!(
                    "Successfully registered @{} with your hotkey.",
                    request.github_username
                )),
                error: None,
            })
        }
        Err(e) => {
            error!("Registration failed: {}", e);
            Json(RegisterResponse {
                success: false,
                message: None,
                error: Some(format!("Registration failed: {}", e)),
            })
        }
    }
}

// ============================================================================
// GET /status/:hotkey - Get status for a hotkey
// ============================================================================

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub registered: bool,
    pub github_username: Option<String>,
    pub valid_issues_count: Option<u32>,
    pub weight: Option<f64>,
}

async fn status_handler(
    State(state): State<Arc<AppState>>,
    Path(hotkey): Path<String>,
) -> Json<StatusResponse> {
    // Check if registered
    let github_username = state.storage.get_github_username(&hotkey).ok().flatten();

    if github_username.is_none() {
        return Json(StatusResponse {
            registered: false,
            github_username: None,
            valid_issues_count: None,
            weight: None,
        });
    }

    // Get bounties for this miner
    let bounties = state.storage.get_miner_bounties(&hotkey).unwrap_or_default();
    let valid_count = bounties.len() as u32;
    let weight = calculate_weight(valid_count);

    Json(StatusResponse {
        registered: true,
        github_username,
        valid_issues_count: Some(valid_count),
        weight: Some(weight),
    })
}

// ============================================================================
// GET /stats - Get challenge statistics
// ============================================================================

#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub total_bounties: u32,
    pub total_miners: usize,
    pub challenge_id: String,
    pub version: String,
}

async fn stats_handler(State(state): State<Arc<AppState>>) -> Json<StatsResponse> {
    let total_bounties = state.storage.get_total_bounties().unwrap_or(0);
    let scores = state.storage.get_all_scores().unwrap_or_default();

    Json(StatsResponse {
        total_bounties,
        total_miners: scores.len(),
        challenge_id: "bounty-challenge".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
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
