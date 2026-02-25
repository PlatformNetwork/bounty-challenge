use alloc::string::ToString;
use alloc::vec::Vec;
use bincode::Options;
use platform_challenge_sdk_wasm::host_functions::host_consensus_get_submission_count;
use platform_challenge_sdk_wasm::{WasmRouteRequest, WasmRouteResponse};
use serde::{Deserialize, Serialize};

use crate::ss58;
use crate::types::{
    BountySubmission, BulkMigrationRequest, ClaimRequest, GitHubUserDetailsResponse,
    HotkeyDetailsResponse, IssueRecord, IssueShort, IssuesStatsResponse, RegisterRequest,
    StatsResponse, StatusResponse, UserBalance,
};

fn to_ss58(hotkey: &str) -> alloc::string::String {
    ss58::normalize_hotkey(hotkey).unwrap_or_else(|| hotkey.to_string())
}
use crate::{scoring, storage, validation};

const MAX_ROUTE_BODY_SIZE: usize = 1_048_576;

fn bincode_options_route_body() -> impl Options {
    bincode::DefaultOptions::new()
        .with_limit(MAX_ROUTE_BODY_SIZE as u64)
        .with_fixint_encoding()
        .allow_trailing_bytes()
}

#[allow(dead_code)]
fn ok_response(body: Vec<u8>) -> WasmRouteResponse {
    WasmRouteResponse { status: 200, body }
}

fn json_response<T: Serialize>(data: &T) -> WasmRouteResponse {
    WasmRouteResponse {
        status: 200,
        body: serde_json::to_vec(data).unwrap_or_default(),
    }
}

fn json_error(status: u16, error: &str, message: &str) -> WasmRouteResponse {
    WasmRouteResponse {
        status,
        body: serde_json::to_vec(&serde_json::json!({
            "error": error,
            "message": message
        }))
        .unwrap_or_default(),
    }
}

fn unauthorized_response() -> WasmRouteResponse {
    json_error(401, "unauthorized", "Authentication required")
}

fn bad_request_response() -> WasmRouteResponse {
    json_error(400, "bad_request", "Invalid request")
}

fn not_found_response() -> WasmRouteResponse {
    json_error(404, "not_found", "Resource not found")
}

fn is_authenticated(request: &WasmRouteRequest) -> bool {
    request
        .auth_hotkey
        .as_ref()
        .map(|k| !k.is_empty())
        .unwrap_or(false)
}

fn get_param<'a>(request: &'a WasmRouteRequest, name: &str) -> Option<&'a str> {
    request
        .params
        .iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.as_str())
}

pub fn handle_leaderboard(_request: &WasmRouteRequest) -> WasmRouteResponse {
    // Rebuild leaderboard dynamically from registered hotkeys
    scoring::rebuild_leaderboard();
    let entries = storage::get_leaderboard();
    json_response(&entries)
}

pub fn handle_stats(_request: &WasmRouteRequest) -> WasmRouteResponse {
    let total_submissions = host_consensus_get_submission_count() as u64;
    let active_miners = storage::get_active_miner_count();
    let validator_count = storage::get_validator_count();
    let issues = storage::get_synced_issues();

    let stats = StatsResponse {
        total_bounties: total_submissions,
        active_miners,
        validator_count,
        total_issues: issues.len() as u64,
    };
    json_response(&stats)
}

pub fn handle_status(request: &WasmRouteRequest) -> WasmRouteResponse {
    let hotkey = match get_param(request, "hotkey") {
        Some(h) => h,
        None => return bad_request_response(),
    };

    let reg = match storage::get_user_by_hotkey(hotkey) {
        Some(r) => r,
        None => {
            let status = StatusResponse {
                registered: false,
                hotkey: to_ss58(hotkey),
                github_username: None,
                valid_issues_count: 0,
                invalid_issues_count: 0,
                balance: UserBalance::default(),
                weight: 0.0,
            };
            return json_response(&status);
        }
    };

    let balance = storage::get_user_balance(hotkey);
    let net = scoring::calculate_net_points(
        balance.valid_count,
        balance.invalid_count,
        balance.duplicate_count,
        balance.malicious_count,
        balance.star_count,
    );
    let weight = if storage::is_banned(hotkey) || net <= 0.0 {
        0.0
    } else {
        scoring::calculate_weight_from_points(balance.valid_count, balance.star_count)
    };

    let status = StatusResponse {
        registered: true,
        hotkey: to_ss58(hotkey),
        github_username: Some(reg.github_username),
        valid_issues_count: balance.valid_count,
        invalid_issues_count: balance.invalid_count,
        balance,
        weight,
    };
    json_response(&status)
}

pub fn handle_register(request: &WasmRouteRequest) -> WasmRouteResponse {
    if !is_authenticated(request) {
        return unauthorized_response();
    }
    if request.body.len() > MAX_ROUTE_BODY_SIZE {
        return bad_request_response();
    }

    // Try JSON first, then bincode for backwards compatibility
    let reg: RegisterRequest = match serde_json::from_slice(&request.body) {
        Ok(r) => r,
        Err(_) => match bincode_options_route_body().deserialize(&request.body) {
            Ok(r) => r,
            Err(_) => return bad_request_response(),
        },
    };

    if reg.github_username.trim().is_empty() {
        return bad_request_response();
    }

    // Use authenticated hotkey from headers, or fall back to body hotkey
    let hotkey = request.auth_hotkey.as_deref().unwrap_or(&reg.hotkey);

    // Check specific error conditions for better error messages
    let existing_hotkey_for_github = storage::get_hotkey_by_github(&reg.github_username);
    let existing_github_for_hotkey = storage::get_github_by_hotkey(hotkey);

    if let Some(ref existing) = existing_hotkey_for_github {
        if existing != hotkey {
            return json_error(
                400,
                "github_already_registered",
                &alloc::format!(
                    "GitHub '{}' is already registered to hotkey {}",
                    reg.github_username,
                    existing
                ),
            );
        }
    }

    if let Some(ref existing) = existing_github_for_hotkey {
        if existing.to_lowercase() != reg.github_username.to_lowercase() {
            return json_error(
                400,
                "hotkey_already_registered",
                &alloc::format!("Hotkey is already registered to GitHub '{}'", existing),
            );
        }
    }

    let result = storage::register_user(&reg.github_username, hotkey);
    if result {
        storage::ensure_hotkey_tracked(hotkey);
        json_response(&serde_json::json!({
            "success": true,
            "message": "Registration successful",
            "hotkey": to_ss58(hotkey),
            "github_username": reg.github_username
        }))
    } else {
        json_error(
            400,
            "registration_failed",
            "Registration failed - unknown error",
        )
    }
}

pub fn handle_claim(request: &WasmRouteRequest) -> WasmRouteResponse {
    if !is_authenticated(request) {
        return unauthorized_response();
    }
    if request.body.len() > MAX_ROUTE_BODY_SIZE {
        return bad_request_response();
    }

    // Get authenticated hotkey from headers
    let auth_hotkey = match &request.auth_hotkey {
        Some(h) if !h.is_empty() => h.clone(),
        _ => return unauthorized_response(),
    };

    // Try new JSON format first (ClaimRequest with issue_url)
    if let Ok(claim_req) = serde_json::from_slice::<ClaimRequest>(&request.body) {
        // Parse issue URL: https://github.com/{owner}/{repo}/issues/{number}
        let parts: Vec<&str> = claim_req.issue_url.split('/').collect();
        if parts.len() < 7 {
            return bad_request_response();
        }

        let repo_owner = parts[3].to_string();
        let repo_name = parts[4].to_string();
        let issue_number: u32 = match parts[6].parse() {
            Ok(n) => n,
            Err(_) => return bad_request_response(),
        };

        // Get user's github username
        let github_username = match storage::get_user_by_hotkey(&auth_hotkey) {
            Some(reg) => reg.github_username,
            None => return unauthorized_response(),
        };

        // Create submission from authenticated request
        let submission = BountySubmission {
            hotkey: auth_hotkey,
            github_username,
            issue_numbers: alloc::vec![issue_number],
            repo_owner,
            repo_name,
            signature: alloc::vec![],
            timestamp: 0,
        };

        let synced_issues = storage::get_synced_issues();
        let result = validation::process_claims(&submission, &synced_issues);

        if !result.claimed.is_empty() {
            scoring::rebuild_leaderboard();
        }

        return json_response(&result);
    }

    // Fallback: try legacy bincode format (BountySubmission)
    let mut submission: BountySubmission =
        match bincode_options_route_body().deserialize(&request.body) {
            Ok(s) => s,
            Err(_) => return bad_request_response(),
        };

    // Override body-provided identity with authenticated hotkey to prevent impersonation
    submission.hotkey = auth_hotkey.clone();
    if let Some(reg) = storage::get_user_by_hotkey(&auth_hotkey) {
        submission.github_username = reg.github_username;
    } else {
        return unauthorized_response();
    }

    if !validation::validate_submission(&submission) {
        return bad_request_response();
    }

    let synced_issues = storage::get_synced_issues();
    let result = validation::process_claims(&submission, &synced_issues);

    if !result.claimed.is_empty() {
        scoring::rebuild_leaderboard();
    }

    json_response(&result)
}

pub fn handle_issues(_request: &WasmRouteRequest) -> WasmRouteResponse {
    let issues = storage::get_synced_issues();
    json_response(&issues)
}

pub fn handle_issues_pending(_request: &WasmRouteRequest) -> WasmRouteResponse {
    let issues = storage::get_pending_issues();
    json_response(&issues)
}

pub fn handle_hotkey_details(request: &WasmRouteRequest) -> WasmRouteResponse {
    let hotkey = match get_param(request, "hotkey") {
        Some(h) => h,
        None => return bad_request_response(),
    };

    let reg = match storage::get_user_by_hotkey(hotkey) {
        Some(r) => r,
        None => return not_found_response(),
    };

    let balance = storage::get_user_balance(hotkey);
    let net = scoring::calculate_net_points(
        balance.valid_count,
        balance.invalid_count,
        balance.duplicate_count,
        balance.malicious_count,
        balance.star_count,
    );
    let weight = if storage::is_banned(hotkey) || net <= 0.0 {
        0.0
    } else {
        scoring::calculate_weight_from_points(balance.valid_count, balance.star_count)
    };

    let total_points = balance.valid_count as f64;
    let penalty_points = (balance.invalid_count + balance.duplicate_count) as f64;

    // Build issues list for this miner's github username
    let issues = storage::get_synced_issues();
    let username_lower = reg.github_username.to_lowercase();
    let mut recent: Vec<IssueShort> = issues
        .iter()
        .filter(|i| i.author.to_lowercase() == username_lower)
        .map(|i| {
            let mut labels = Vec::new();
            if i.has_valid_label {
                labels.push(alloc::string::String::from("valid"));
            }
            if i.has_invalid_label {
                labels.push(alloc::string::String::from("invalid"));
            }
            if i.has_duplicate_label {
                labels.push(alloc::string::String::from("duplicate"));
            }
            if i.has_ide_label {
                labels.push(alloc::string::String::from("ide"));
            }
            let state = if i.is_closed {
                alloc::string::String::from("closed")
            } else {
                alloc::string::String::from("open")
            };
            let mut url = alloc::string::String::from("https://github.com/");
            url.push_str(&i.repo_owner);
            url.push('/');
            url.push_str(&i.repo_name);
            url.push_str("/issues/");
            let _ = core::fmt::Write::write_fmt(&mut url, format_args!("{}", i.issue_number));

            IssueShort {
                issue_id: i.issue_number,
                repo_owner: i.repo_owner.clone(),
                repo_name: i.repo_name.clone(),
                title: alloc::string::String::new(),
                state,
                labels,
                updated_at: alloc::string::String::new(),
                issue_url: url,
            }
        })
        .collect();
    recent.sort_by(|a, b| b.issue_id.cmp(&a.issue_id));

    let mut registered_at = alloc::string::String::new();
    let _ =
        core::fmt::Write::write_fmt(&mut registered_at, format_args!("{}", reg.registered_epoch));

    let is_penalized = balance.is_penalized;
    let details = HotkeyDetailsResponse {
        hotkey: to_ss58(hotkey),
        github_username: reg.github_username,
        registered_at,
        valid_issues: balance.valid_count,
        invalid_issues: balance.invalid_count,
        duplicate_issues: balance.duplicate_count,
        total_points,
        penalty_points,
        net_points: net,
        balance,
        is_penalized,
        weight,
        recent_issues: recent,
    };
    json_response(&details)
}

pub fn handle_issues_stats(_request: &WasmRouteRequest) -> WasmRouteResponse {
    let issues = storage::get_synced_issues();

    let total = issues.len() as u64;
    let mut open = 0u64;
    let mut closed = 0u64;
    let mut valid = 0u64;
    let mut invalid = 0u64;
    let mut pending = 0u64;

    for issue in &issues {
        if issue.is_closed {
            closed += 1;
        } else {
            open += 1;
        }
        if issue.has_valid_label {
            valid += 1;
        }
        if issue.has_invalid_label {
            invalid += 1;
        }
        if issue.is_closed && !issue.has_valid_label && !issue.has_invalid_label {
            pending += 1;
        }
    }

    let stats = IssuesStatsResponse {
        total,
        open,
        closed,
        valid,
        invalid,
        pending,
    };
    json_response(&stats)
}

pub fn handle_github_user(request: &WasmRouteRequest) -> WasmRouteResponse {
    let username = match get_param(request, "username") {
        Some(u) => u,
        None => return bad_request_response(),
    };

    let hotkey = storage::get_hotkey_by_github(username);
    let issues = storage::get_synced_issues();

    let user_issues: Vec<&IssueRecord> = issues
        .iter()
        .filter(|i| i.author.to_lowercase() == username.to_lowercase())
        .collect();

    let total_issues = user_issues.len() as u64;
    let valid_issues = user_issues.iter().filter(|i| i.has_valid_label).count() as u64;
    let invalid_issues = user_issues.iter().filter(|i| i.has_invalid_label).count() as u64;
    let open_issues = user_issues.iter().filter(|i| !i.is_closed).count() as u64;

    let mut recent: Vec<IssueShort> = user_issues
        .iter()
        .map(|i| {
            let mut labels = Vec::new();
            if i.has_valid_label {
                labels.push(alloc::string::String::from("valid"));
            }
            if i.has_invalid_label {
                labels.push(alloc::string::String::from("invalid"));
            }
            if i.has_ide_label {
                labels.push(alloc::string::String::from("ide"));
            }
            let state = if i.is_closed {
                alloc::string::String::from("closed")
            } else {
                alloc::string::String::from("open")
            };
            let mut url = alloc::string::String::from("https://github.com/");
            url.push_str(&i.repo_owner);
            url.push('/');
            url.push_str(&i.repo_name);
            url.push_str("/issues/");
            let _ = core::fmt::Write::write_fmt(&mut url, format_args!("{}", i.issue_number));

            IssueShort {
                issue_id: i.issue_number,
                repo_owner: i.repo_owner.clone(),
                repo_name: i.repo_name.clone(),
                title: alloc::string::String::new(),
                state,
                labels,
                updated_at: alloc::string::String::new(),
                issue_url: url,
            }
        })
        .collect();
    recent.truncate(20);

    let registered_at = hotkey.as_ref().and_then(|hk| {
        storage::get_user_by_hotkey(hk).map(|reg| {
            let mut s = alloc::string::String::new();
            let _ = core::fmt::Write::write_fmt(&mut s, format_args!("{}", reg.registered_epoch));
            s
        })
    });

    let details = GitHubUserDetailsResponse {
        github_username: alloc::string::String::from(username),
        hotkey: hotkey.map(|h| to_ss58(&h)),
        registered_at,
        total_issues,
        valid_issues,
        invalid_issues,
        open_issues,
        recent_issues: recent,
    };
    json_response(&details)
}

pub fn handle_get_weights(_request: &WasmRouteRequest) -> WasmRouteResponse {
    let entries = storage::get_leaderboard();
    let weights = scoring::calculate_weights_from_leaderboard(&entries);
    json_response(&weights)
}

pub fn handle_sudo_bulk_migrate(request: &WasmRouteRequest) -> WasmRouteResponse {
    if !is_authenticated(request) {
        return unauthorized_response();
    }

    let auth_hotkey = match &request.auth_hotkey {
        Some(h) if !h.is_empty() => h.clone(),
        _ => return unauthorized_response(),
    };

    // Auto-set sudo owner on first call if not set
    if storage::get_sudo_owner().is_none() {
        storage::set_sudo_owner(&auth_hotkey);
    }

    if !storage::is_sudo_owner(&auth_hotkey) {
        return json_error(
            403,
            "forbidden",
            "Only the sudo owner can perform bulk migration",
        );
    }

    if request.body.len() > MAX_ROUTE_BODY_SIZE {
        return bad_request_response();
    }

    let migration: BulkMigrationRequest = match serde_json::from_slice(&request.body) {
        Ok(m) => m,
        Err(_) => return json_error(400, "bad_request", "Invalid migration request JSON"),
    };

    if migration.entries.is_empty() {
        return json_error(400, "bad_request", "No entries to migrate");
    }

    if migration.entries.len() > 1000 {
        return json_error(400, "bad_request", "Maximum 1000 entries per batch");
    }

    let pairs: alloc::vec::Vec<(alloc::string::String, alloc::string::String)> = migration
        .entries
        .iter()
        .map(|e| (e.hotkey.clone(), e.github_username.clone()))
        .collect();

    let (success, skipped) = storage::bulk_register_users(&pairs);

    // Rebuild leaderboard after bulk migration
    if success > 0 {
        scoring::rebuild_leaderboard();
    }

    json_response(&serde_json::json!({
        "success": true,
        "registered": success,
        "skipped": skipped,
        "total": migration.entries.len()
    }))
}

#[derive(Debug, Deserialize)]
struct SudoRegisterRequest {
    pub hotkey: alloc::string::String,
    pub github_username: alloc::string::String,
}

pub fn handle_sudo_register_user(request: &WasmRouteRequest) -> WasmRouteResponse {
    if !is_authenticated(request) {
        return unauthorized_response();
    }

    let auth_hotkey = match &request.auth_hotkey {
        Some(h) if !h.is_empty() => h.clone(),
        _ => return unauthorized_response(),
    };

    if !storage::is_sudo_owner(&auth_hotkey) {
        return json_error(403, "forbidden", "Only the sudo owner can register users");
    }

    let req: SudoRegisterRequest = match serde_json::from_slice(&request.body) {
        Ok(r) => r,
        Err(_) => return json_error(400, "bad_request", "Invalid request JSON"),
    };

    if req.hotkey.is_empty() || req.github_username.is_empty() {
        return json_error(400, "bad_request", "hotkey and github_username required");
    }

    if storage::register_user(&req.github_username, &req.hotkey) {
        storage::ensure_hotkey_tracked(&req.hotkey);
        scoring::rebuild_leaderboard();
        json_response(&serde_json::json!({
            "success": true,
            "hotkey": to_ss58(&req.hotkey),
            "github_username": req.github_username
        }))
    } else {
        json_error(
            400,
            "registration_failed",
            "User already registered or conflict",
        )
    }
}

pub fn handle_sudo_sync_github(request: &WasmRouteRequest) -> WasmRouteResponse {
    if !is_authenticated(request) {
        return unauthorized_response();
    }

    let auth_hotkey = match &request.auth_hotkey {
        Some(h) if !h.is_empty() => h.clone(),
        _ => return unauthorized_response(),
    };

    if !storage::is_sudo_owner(&auth_hotkey) {
        return json_error(403, "forbidden", "Only the sudo owner can trigger sync");
    }

    let github_token: Option<alloc::string::String> =
        serde_json::from_slice::<serde_json::Value>(&request.body)
            .ok()
            .and_then(|v| {
                v.get("github_token")
                    .and_then(|t| t.as_str())
                    .map(alloc::string::String::from)
            });

    let stats = crate::github_sync::fetch_and_process_issues_with_token(github_token.as_deref());
    storage::recount_all_balances();
    scoring::rebuild_leaderboard();

    json_response(&serde_json::json!({
        "success": true,
        "fetched": stats.fetched,
        "awarded": stats.awarded,
        "penalized": stats.penalized,
        "error": stats.last_error
    }))
}

pub fn handle_sudo_recount(request: &WasmRouteRequest) -> WasmRouteResponse {
    if !is_authenticated(request) {
        return unauthorized_response();
    }

    let auth_hotkey = match &request.auth_hotkey {
        Some(h) if !h.is_empty() => h.clone(),
        _ => return unauthorized_response(),
    };

    if !storage::is_sudo_owner(&auth_hotkey) {
        return json_error(403, "forbidden", "Only the sudo owner can recount");
    }

    let result = storage::recount_all_balances();
    scoring::rebuild_leaderboard();

    json_response(&result)
}

#[derive(Debug, Deserialize)]
struct BanRequest {
    pub hotkey: alloc::string::String,
}

pub fn handle_sudo_ban_user(request: &WasmRouteRequest) -> WasmRouteResponse {
    if !is_authenticated(request) {
        return unauthorized_response();
    }
    let auth_hotkey = match &request.auth_hotkey {
        Some(h) if !h.is_empty() => h.clone(),
        _ => return unauthorized_response(),
    };
    if !storage::is_sudo_owner(&auth_hotkey) {
        return json_error(403, "forbidden", "Only the sudo owner can ban users");
    }
    let req: BanRequest = match serde_json::from_slice(&request.body) {
        Ok(r) => r,
        Err(_) => return json_error(400, "bad_request", "Invalid request JSON"),
    };
    if req.hotkey.is_empty() {
        return json_error(400, "bad_request", "hotkey required");
    }
    storage::ban_user(&req.hotkey);
    scoring::rebuild_leaderboard();
    json_response(&serde_json::json!({
        "success": true,
        "banned": req.hotkey
    }))
}

pub fn handle_sudo_unban_user(request: &WasmRouteRequest) -> WasmRouteResponse {
    if !is_authenticated(request) {
        return unauthorized_response();
    }
    let auth_hotkey = match &request.auth_hotkey {
        Some(h) if !h.is_empty() => h.clone(),
        _ => return unauthorized_response(),
    };
    if !storage::is_sudo_owner(&auth_hotkey) {
        return json_error(403, "forbidden", "Only the sudo owner can unban users");
    }
    let req: BanRequest = match serde_json::from_slice(&request.body) {
        Ok(r) => r,
        Err(_) => return json_error(400, "bad_request", "Invalid request JSON"),
    };
    if req.hotkey.is_empty() {
        return json_error(400, "bad_request", "hotkey required");
    }
    storage::unban_user(&req.hotkey);
    scoring::rebuild_leaderboard();
    json_response(&serde_json::json!({
        "success": true,
        "unbanned": req.hotkey
    }))
}
