//! Bounty Challenge Server
//!
//! HTTP server for challenge endpoints.

use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::{error, info};

use crate::auth::is_valid_ss58_hotkey;
use crate::challenge::BountyChallenge;
use crate::pg_storage::PgStorage;
use platform_challenge_sdk::server::{
    EvaluationRequest, EvaluationResponse, HealthResponse, ServerChallenge, ValidationRequest,
    ValidationResponse,
};

pub struct WeightsCache {
    pub weights: Vec<crate::pg_storage::CurrentWeight>,
    pub updated_at: std::time::Instant,
}

pub struct AppState {
    pub challenge: Arc<BountyChallenge>,
    pub storage: Arc<PgStorage>,
    pub started_at: std::time::Instant,
    pub weights_cache: RwLock<WeightsCache>,
}

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Root-level endpoints (direct access + bridge root-level)
        .route("/health", get(health_handler))
        .route("/config", get(config_handler))
        .route("/evaluate", post(evaluate_handler))
        .route("/validate", post(validate_handler))
        .route("/leaderboard", get(leaderboard_handler))
        .route("/get_weights", get(get_weights_handler))
        // Direct access endpoints
        .route("/register", post(register_handler))
        .route("/status/:hotkey", get(status_handler))
        .route("/stats", get(stats_handler))
        .route("/invalid", post(invalid_handler))
        .route("/issues", get(issues_handler))
        .route("/issues/pending", get(pending_issues_handler))
        .route("/issues/stats", get(issues_stats_handler))
        .route("/hotkey/:hotkey", get(hotkey_details_handler))
        .route("/github/:username", get(github_user_handler))
        .route("/sync/status", get(sync_status_handler))
        .route("/sync/trigger", post(trigger_sync_handler))
        // Bridge API endpoints (/api/v1/... routes for platform bridge)
        .route("/api/v1/register", post(register_handler))
        .route("/api/v1/status/:hotkey", get(status_handler))
        .route("/api/v1/stats", get(stats_handler))
        .route("/api/v1/invalid", post(invalid_handler))
        .route("/api/v1/issues", get(issues_handler))
        .route("/api/v1/issues/pending", get(pending_issues_handler))
        .route("/api/v1/issues/stats", get(issues_stats_handler))
        .route("/api/v1/hotkey/:hotkey", get(hotkey_details_handler))
        .route("/api/v1/github/:username", get(github_user_handler))
        .route("/api/v1/sync/status", get(sync_status_handler))
        .route("/api/v1/sync/trigger", post(trigger_sync_handler))
        // CORS: Permissive policy is intentional - this is a public API accessed by
        // browser-based tools, CLI clients, and third-party integrations from any origin
        .layer(CorsLayer::permissive())
        .with_state(state)
}

pub async fn refresh_weights_cache(state: &Arc<AppState>) {
    match state.storage.get_current_weights().await {
        Ok(weights) => {
            let mut cache = state.weights_cache.write().await;
            cache.weights = weights;
            cache.updated_at = std::time::Instant::now();
            tracing::debug!("Weights cache refreshed ({} entries)", cache.weights.len());
        }
        Err(e) => {
            error!("Failed to refresh weights cache: {}", e);
        }
    }
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
    match serde_json::to_value(state.challenge.config()) {
        Ok(value) => Json(value),
        Err(e) => {
            error!("Failed to serialize config: {}", e);
            Json(serde_json::json!({ "error": "Failed to load configuration" }))
        }
    }
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
    match state.challenge.get_leaderboard().await {
        Ok(lb) => Json(serde_json::json!({ "leaderboard": lb })),
        Err(e) => {
            error!("Failed to get leaderboard: {}", e);
            Json(serde_json::json!({ "error": "Failed to load leaderboard" }))
        }
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
pub struct WeightEntry {
    pub hotkey: String,
    pub weight: f64,
}

#[derive(Debug, Serialize)]
pub struct GetWeightsResponse {
    pub epoch: u64,
    pub weights: Vec<WeightEntry>,
}

// Weight calculation moved to pg_storage::calculate_weight_from_points()
// Uses point system: 1 point per issue + 0.25 points per starred repo
// 100 points = 100% weight

async fn get_weights_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetWeightsQuery>,
) -> Json<GetWeightsResponse> {
    // Get current epoch (use provided or estimate from time)
    let epoch = query.epoch.unwrap_or_else(|| {
        // Estimate epoch from current time (12 second blocks on Bittensor)
        // Fall back to 0 if system time is somehow before Unix epoch
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        now / 12
    });

    // Always serve from cache (refreshed every 5 minutes by background task)
    let current_weights = state.weights_cache.read().await.weights.clone();

    // Convert to WeightEntry with normalization
    // Each user's raw weight = their points * 0.02 (NO CAP - proportional to points)
    // Points: 1 per issue + 0.25 per starred repo
    // This ensures users with more points always have proportionally higher weight
    let mut weights: Vec<WeightEntry> = current_weights
        .iter()
        .filter(|w| w.weight > 0.0 && !w.is_penalized) // Exclude zero weight and penalized users
        .map(|w| WeightEntry {
            hotkey: w.hotkey.clone(),
            weight: w.weight, // Raw weight before normalization
        })
        .collect();

    // Normalize weights so they sum to exactly 1.0
    // This ensures proper distribution even when total weights exceed 1.0
    let total_weight: f64 = weights.iter().map(|w| w.weight).sum();
    if total_weight > 0.0 {
        for w in &mut weights {
            w.weight /= total_weight;
        }
    }

    // Sort by weight descending
    weights.sort_by(|a, b| {
        b.weight
            .partial_cmp(&a.weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    info!(
        "Returning weights for {} miners at epoch {}",
        weights.len(),
        epoch
    );

    Json(GetWeightsResponse { epoch, weights })
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
    // Validate timestamp (must be within 5 minutes, only past timestamps allowed)
    let now = chrono::Utc::now().timestamp();
    if request.timestamp > now || (now - request.timestamp) > 300 {
        return Json(RegisterResponse {
            success: false,
            message: None,
            error: Some("Timestamp expired or invalid. Please try again.".to_string()),
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

    // Check if GitHub username is already registered with a DIFFERENT hotkey
    if let Ok(Some(existing_hotkey)) = state
        .storage
        .get_hotkey_by_github(&request.github_username)
        .await
    {
        if existing_hotkey != request.hotkey {
            return Json(RegisterResponse {
                success: false,
                message: None,
                error: Some(format!(
                    "GitHub username @{} is already registered with a different hotkey. Each username can only be linked to one hotkey.",
                    request.github_username
                )),
            });
        }
    }

    // Check if hotkey is already registered with a DIFFERENT username
    if let Ok(Some(existing_username)) = state.storage.get_github_by_hotkey(&request.hotkey).await {
        if existing_username.to_lowercase() != request.github_username.to_lowercase() {
            return Json(RegisterResponse {
                success: false,
                message: None,
                error: Some(format!(
                    "This hotkey is already registered with @{}. Each hotkey can only be linked to one GitHub account.",
                    existing_username
                )),
            });
        }
    }

    // Register in storage (async)
    match state
        .storage
        .register_user(&request.github_username, &request.hotkey)
        .await
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
                error: Some("Registration failed. Please try again later.".to_string()),
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
    pub valid_issues_count: Option<u64>,
    pub invalid_issues_count: Option<u64>,
    pub balance: Option<i64>,
    pub is_penalized: bool,
    pub weight: Option<f64>,
}

async fn status_handler(
    State(state): State<Arc<AppState>>,
    Path(hotkey): Path<String>,
) -> Json<StatusResponse> {
    // Check if registered
    let github_username = state
        .storage
        .get_github_by_hotkey(&hotkey)
        .await
        .ok()
        .flatten();

    if github_username.is_none() {
        return Json(StatusResponse {
            registered: false,
            github_username: None,
            valid_issues_count: None,
            invalid_issues_count: None,
            balance: None,
            is_penalized: false,
            weight: None,
        });
    }

    // Get user balance (valid - invalid) from PostgreSQL
    let user_balance = state.storage.get_user_balance(&hotkey).await.ok().flatten();

    let (valid_count, invalid_count, balance, is_penalized, weight) = match user_balance {
        Some(b) => {
            let weight = if b.is_penalized {
                0.0
            } else {
                state
                    .storage
                    .calculate_user_weight(&hotkey)
                    .await
                    .unwrap_or(0.0)
            };
            (
                b.valid_count as u64,
                b.invalid_count as u64,
                b.balance,
                b.is_penalized,
                weight,
            )
        }
        None => (0, 0, 0, false, 0.0),
    };

    Json(StatusResponse {
        registered: true,
        github_username,
        valid_issues_count: Some(valid_count),
        invalid_issues_count: Some(invalid_count),
        balance: Some(balance),
        is_penalized,
        weight: Some(weight),
    })
}

// ============================================================================
// POST /invalid - Record an invalid issue (maintainers only)
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct InvalidIssueRequest {
    pub issue_id: i64,
    pub repo_owner: String,
    pub repo_name: String,
    pub github_username: String,
    pub issue_url: String,
    pub issue_title: Option<String>,
    pub reason: Option<String>,
    pub hotkey: String,
    pub signature: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize)]
pub struct InvalidIssueResponse {
    pub success: bool,
    pub message: Option<String>,
    pub error: Option<String>,
}

async fn invalid_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<InvalidIssueRequest>,
) -> Json<InvalidIssueResponse> {
    // Validate timestamp (must be within 5 minutes, only past timestamps allowed)
    let now = chrono::Utc::now().timestamp();
    if request.timestamp > now || (now - request.timestamp) > 300 {
        return Json(InvalidIssueResponse {
            success: false,
            message: None,
            error: Some("Timestamp expired or invalid. Please try again.".to_string()),
        });
    }

    // Verify signature
    let message = format!(
        "invalid_issue:{}:{}:{}",
        request.issue_id,
        request.github_username.to_lowercase(),
        request.timestamp
    );

    if !crate::auth::verify_signature(&request.hotkey, &message, &request.signature) {
        return Json(InvalidIssueResponse {
            success: false,
            message: None,
            error: Some("Invalid signature. Make sure you're using the correct key.".to_string()),
        });
    }

    // Record the invalid issue (async)
    match state
        .storage
        .record_invalid_issue(
            request.issue_id,
            &request.repo_owner,
            &request.repo_name,
            &request.github_username,
            &request.issue_url,
            request.issue_title.as_deref(),
            request.reason.as_deref(),
        )
        .await
    {
        Ok(()) => {
            info!(
                "Recorded invalid issue #{} by @{}",
                request.issue_id, request.github_username
            );
            Json(InvalidIssueResponse {
                success: true,
                message: Some(format!(
                    "Recorded invalid issue #{} by @{}",
                    request.issue_id, request.github_username
                )),
                error: None,
            })
        }
        Err(e) => {
            error!("Failed to record invalid issue: {}", e);
            Json(InvalidIssueResponse {
                success: false,
                message: None,
                error: Some("Failed to record invalid issue".to_string()),
            })
        }
    }
}

// ============================================================================
// GET /stats - Get challenge statistics
// ============================================================================

#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub total_bounties: u32,
    pub total_miners: usize,
    pub total_invalid: u32,
    pub penalized_miners: u32,
    pub challenge_id: String,
    pub version: String,
}

async fn stats_handler(State(state): State<Arc<AppState>>) -> Json<StatsResponse> {
    // Get stats from PostgreSQL - ALL STATS ARE 24H ONLY
    let stats = state.storage.get_stats_24h().await.ok();
    let current_weights = state.weights_cache.read().await.weights.clone();

    // Count miners with activity in last 24h (weight > 0)
    let active_miners = current_weights.iter().filter(|w| w.weight > 0.0).count();
    // Count penalized miners (from current_weights which is 24h based)
    let penalized_count = current_weights.iter().filter(|w| w.is_penalized).count();

    Json(StatsResponse {
        total_bounties: stats.map(|s| s.total_issues_resolved as u32).unwrap_or(0),
        total_miners: active_miners, // Only count miners active in 24h
        total_invalid: penalized_count as u32,
        penalized_miners: penalized_count as u32,
        challenge_id: "bounty-challenge".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

// ============================================================================
// ISSUES API (cached from GitHub)
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct IssuesQuery {
    pub state: Option<String>,
    pub label: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

async fn issues_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<IssuesQuery>,
) -> Json<serde_json::Value> {
    let limit = query.limit.unwrap_or(100).min(1000); // Allow viewing up to 1000 issues
    let offset = query.offset.unwrap_or(0);

    match state
        .storage
        .get_issues(
            query.state.as_deref(),
            query.label.as_deref(),
            limit,
            offset,
        )
        .await
    {
        Ok(issues) => Json(serde_json::json!({
            "issues": issues,
            "count": issues.len(),
            "limit": limit,
            "offset": offset
        })),
        Err(e) => {
            error!("Failed to get issues: {}", e);
            Json(serde_json::json!({ "error": "Failed to retrieve issues" }))
        }
    }
}

async fn pending_issues_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<IssuesQuery>,
) -> Json<serde_json::Value> {
    let limit = query.limit.unwrap_or(100).min(1000); // Allow viewing up to 1000 issues
    let offset = query.offset.unwrap_or(0);

    match state.storage.get_pending_issues(limit, offset).await {
        Ok(issues) => Json(serde_json::json!({
            "issues": issues,
            "count": issues.len(),
            "limit": limit,
            "offset": offset
        })),
        Err(e) => {
            error!("Failed to get pending issues: {}", e);
            Json(serde_json::json!({ "error": "Failed to retrieve pending issues" }))
        }
    }
}

async fn issues_stats_handler(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    match state.storage.get_issues_stats().await {
        Ok(stats) => match serde_json::to_value(stats) {
            Ok(value) => Json(value),
            Err(e) => {
                error!("Failed to serialize issues stats: {}", e);
                Json(serde_json::json!({ "error": "Failed to retrieve issues statistics" }))
            }
        },
        Err(e) => {
            error!("Failed to get issues stats: {}", e);
            Json(serde_json::json!({ "error": "Failed to retrieve issues statistics" }))
        }
    }
}

async fn hotkey_details_handler(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
) -> Json<serde_json::Value> {
    // Resolve identifier to hotkey: either it's already a valid SS58 hotkey,
    // or it's a GitHub username that we need to look up
    let hotkey = if is_valid_ss58_hotkey(&identifier) {
        identifier
    } else {
        // Try to look up as GitHub username
        match state.storage.get_hotkey_by_github(&identifier).await {
            Ok(Some(resolved_hotkey)) => resolved_hotkey,
            Ok(None) => {
                return Json(serde_json::json!({ "error": "Hotkey not found" }));
            }
            Err(e) => {
                error!("Failed to look up GitHub username '{}': {}", identifier, e);
                return Json(serde_json::json!({ "error": "Failed to retrieve hotkey details" }));
            }
        }
    };

    match state.storage.get_hotkey_details(&hotkey).await {
        Ok(Some(details)) => match serde_json::to_value(details) {
            Ok(value) => Json(value),
            Err(e) => {
                error!("Failed to serialize hotkey details: {}", e);
                Json(serde_json::json!({ "error": "Failed to retrieve hotkey details" }))
            }
        },
        Ok(None) => Json(serde_json::json!({ "error": "Hotkey not found" })),
        Err(e) => {
            error!("Failed to get hotkey details: {}", e);
            Json(serde_json::json!({ "error": "Failed to retrieve hotkey details" }))
        }
    }
}

async fn github_user_handler(
    State(state): State<Arc<AppState>>,
    Path(username): Path<String>,
) -> Json<serde_json::Value> {
    match state.storage.get_github_user_details(&username).await {
        Ok(Some(details)) => match serde_json::to_value(details) {
            Ok(value) => Json(value),
            Err(e) => {
                error!("Failed to serialize GitHub user details: {}", e);
                Json(serde_json::json!({ "error": "Failed to retrieve GitHub user details" }))
            }
        },
        Ok(None) => Json(serde_json::json!({ "error": "GitHub user not found" })),
        Err(e) => {
            error!("Failed to get GitHub user details: {}", e);
            Json(serde_json::json!({ "error": "Failed to retrieve GitHub user details" }))
        }
    }
}

async fn sync_status_handler(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    match state.storage.get_sync_status().await {
        Ok(status) => Json(serde_json::json!({
            "repos": status,
            "issues_stats": state.storage.get_issues_stats().await.ok()
        })),
        Err(e) => {
            error!("Failed to get sync status: {}", e);
            Json(serde_json::json!({ "error": "Failed to retrieve sync status" }))
        }
    }
}

/// Request body for sync trigger (requires authentication)
#[derive(Debug, Deserialize)]
pub struct SyncTriggerRequest {
    pub hotkey: String,
    pub signature: String,
    pub timestamp: i64,
}

async fn trigger_sync_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SyncTriggerRequest>,
) -> Json<serde_json::Value> {
    // Validate timestamp (must be within 5 minutes, only past timestamps allowed)
    let now = chrono::Utc::now().timestamp();
    if request.timestamp > now || (now - request.timestamp) > 300 {
        return Json(
            serde_json::json!({ "error": "Timestamp expired or invalid. Please try again." }),
        );
    }

    // Verify signature
    let message = format!("sync_trigger:{}", request.timestamp);

    if !crate::auth::verify_signature(&request.hotkey, &message, &request.signature) {
        return Json(
            serde_json::json!({ "error": "Invalid signature. Make sure you're using the correct key." }),
        );
    }

    // Get target repos
    let repos = match state.storage.get_active_repos().await {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to get active repos: {}", e);
            return Json(serde_json::json!({ "error": "Failed to get repositories" }));
        }
    };

    let mut synced = 0;
    let mut errors = Vec::new();

    for repo in repos {
        match sync_repo(&state.storage, &repo.owner, &repo.repo).await {
            Ok(count) => {
                synced += count;
                info!("Synced {} issues from {}/{}", count, repo.owner, repo.repo);
            }
            Err(e) => {
                error!("Failed to sync {}/{}: {}", repo.owner, repo.repo, e);
                errors.push(format!("{}/{}", repo.owner, repo.repo));
            }
        }
    }

    Json(serde_json::json!({
        "success": errors.is_empty(),
        "issues_synced": synced,
        "errors": errors
    }))
}

/// Sync issues from a single repo
/// Fetches ALL issues from GitHub and marks missing ones as deleted
pub async fn sync_repo(storage: &PgStorage, owner: &str, repo: &str) -> anyhow::Result<i32> {
    let github = crate::github::GitHubClient::new(owner, repo);

    info!("Syncing all issues from {}/{}", owner, repo);

    // Fetch ALL issues from GitHub (both open and closed)
    let issues = github.get_all_issues().await?;
    let count = issues.len() as i32;

    // Collect issue IDs that we see from GitHub
    let seen_issue_ids: Vec<i64> = issues.iter().map(|i| i.number as i64).collect();

    // Upsert each issue (this also clears deleted_at if issue reappears)
    for issue in &issues {
        storage.upsert_issue(issue, owner, repo).await?;
    }

    // Mark issues not returned by GitHub as deleted (transferred/removed)
    let deleted = storage
        .mark_deleted_issues(owner, repo, &seen_issue_ids)
        .await?;
    if deleted > 0 {
        info!(
            "Marked {} stale issues as deleted in {}/{}",
            deleted, owner, repo
        );
    }

    // Update sync state
    storage.update_sync_state(owner, repo, count).await?;

    info!(
        "Sync complete for {}/{}: {} issues from GitHub, {} marked deleted",
        owner, repo, count, deleted
    );

    Ok(count)
}

/// Run the server
pub async fn run_server(
    host: &str,
    port: u16,
    challenge: Arc<BountyChallenge>,
    storage: Arc<PgStorage>,
) -> anyhow::Result<()> {
    let state = Arc::new(AppState {
        challenge,
        storage,
        started_at: std::time::Instant::now(),
        weights_cache: RwLock::new(WeightsCache {
            weights: vec![],
            updated_at: std::time::Instant::now(),
        }),
    });

    refresh_weights_cache(&state).await;

    let cache_state = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(300));
        interval.tick().await;
        loop {
            interval.tick().await;
            refresh_weights_cache(&cache_state).await;
        }
    });
    info!("Weights cache background refresh started (every 300 seconds)");

    let app = create_router(state);
    let addr = format!("{}:{}", host, port);

    info!("Starting Bounty Challenge server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
