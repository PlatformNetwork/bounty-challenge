use alloc::vec::Vec;
use bincode::Options;
use platform_challenge_sdk_wasm::host_functions::host_consensus_get_submission_count;
use platform_challenge_sdk_wasm::{WasmRouteRequest, WasmRouteResponse};

use crate::types::{
    BountySubmission, IssueRecord, RegisterRequest, StatsResponse, StatusResponse, UserBalance,
};
use crate::{scoring, storage, validation};

const MAX_ROUTE_BODY_SIZE: usize = 1_048_576;

fn bincode_options_route_body() -> impl Options {
    bincode::DefaultOptions::new()
        .with_limit(MAX_ROUTE_BODY_SIZE as u64)
        .with_fixint_encoding()
        .allow_trailing_bytes()
}

fn ok_response(body: Vec<u8>) -> WasmRouteResponse {
    WasmRouteResponse { status: 200, body }
}

fn unauthorized_response() -> WasmRouteResponse {
    WasmRouteResponse {
        status: 401,
        body: bincode::serialize(&false).unwrap_or_default(),
    }
}

fn bad_request_response() -> WasmRouteResponse {
    WasmRouteResponse {
        status: 400,
        body: bincode::serialize(&false).unwrap_or_default(),
    }
}

fn not_found_response() -> WasmRouteResponse {
    WasmRouteResponse {
        status: 404,
        body: Vec::new(),
    }
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
    let entries = storage::get_leaderboard();
    ok_response(bincode::serialize(&entries).unwrap_or_default())
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
    ok_response(bincode::serialize(&stats).unwrap_or_default())
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
                github_username: None,
                valid_issues_count: 0,
                invalid_issues_count: 0,
                balance: UserBalance::default(),
                weight: 0.0,
            };
            return ok_response(bincode::serialize(&status).unwrap_or_default());
        }
    };

    let balance = storage::get_user_balance(hotkey);
    let weight = if balance.is_penalized {
        0.0
    } else {
        scoring::calculate_weight_from_points(balance.valid_count, balance.star_count)
    };

    let status = StatusResponse {
        registered: true,
        github_username: Some(reg.github_username),
        valid_issues_count: balance.valid_count,
        invalid_issues_count: balance.invalid_count,
        balance,
        weight,
    };
    ok_response(bincode::serialize(&status).unwrap_or_default())
}

pub fn handle_register(request: &WasmRouteRequest) -> WasmRouteResponse {
    if !is_authenticated(request) {
        return unauthorized_response();
    }
    if request.body.len() > MAX_ROUTE_BODY_SIZE {
        return bad_request_response();
    }

    let reg: RegisterRequest = match bincode_options_route_body().deserialize(&request.body) {
        Ok(r) => r,
        Err(_) => return bad_request_response(),
    };

    let hotkey = request.auth_hotkey.as_deref().unwrap_or(&reg.hotkey);

    let result = storage::register_user(&reg.github_username, hotkey);
    if result {
        storage::ensure_hotkey_tracked(hotkey);
    }
    ok_response(bincode::serialize(&result).unwrap_or_default())
}

pub fn handle_claim(request: &WasmRouteRequest) -> WasmRouteResponse {
    if !is_authenticated(request) {
        return unauthorized_response();
    }
    if request.body.len() > MAX_ROUTE_BODY_SIZE {
        return bad_request_response();
    }

    let submission: BountySubmission = match bincode_options_route_body().deserialize(&request.body)
    {
        Ok(s) => s,
        Err(_) => return bad_request_response(),
    };

    if !validation::validate_submission(&submission) {
        return bad_request_response();
    }

    let synced_issues = storage::get_synced_issues();
    let result = validation::process_claims(&submission, &synced_issues);

    if !result.claimed.is_empty() {
        scoring::rebuild_leaderboard();
    }

    ok_response(bincode::serialize(&result).unwrap_or_default())
}

pub fn handle_issues(_request: &WasmRouteRequest) -> WasmRouteResponse {
    let issues = storage::get_synced_issues();
    ok_response(bincode::serialize(&issues).unwrap_or_default())
}

pub fn handle_issues_pending(_request: &WasmRouteRequest) -> WasmRouteResponse {
    let issues = storage::get_pending_issues();
    ok_response(bincode::serialize(&issues).unwrap_or_default())
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
    let weight = if balance.is_penalized {
        0.0
    } else {
        scoring::calculate_weight_from_points(balance.valid_count, balance.star_count)
    };

    let status = StatusResponse {
        registered: true,
        github_username: Some(reg.github_username),
        valid_issues_count: balance.valid_count,
        invalid_issues_count: balance.invalid_count,
        balance,
        weight,
    };
    ok_response(bincode::serialize(&status).unwrap_or_default())
}

/// Sync issues data - writes go through platform-v2 consensus via host_storage_set
pub fn handle_issues_sync(request: &WasmRouteRequest) -> WasmRouteResponse {
    if !is_authenticated(request) {
        return unauthorized_response();
    }
    if request.body.len() > MAX_ROUTE_BODY_SIZE {
        return bad_request_response();
    }

    if let Ok(issues) = bincode_options_route_body().deserialize::<Vec<IssueRecord>>(&request.body)
    {
        // This write goes through platform-v2 StorageProposal consensus
        let result = storage::store_issue_data(&issues);
        ok_response(bincode::serialize(&result).unwrap_or_default())
    } else {
        bad_request_response()
    }
}

pub fn handle_get_weights(_request: &WasmRouteRequest) -> WasmRouteResponse {
    let entries = storage::get_leaderboard();
    let weights = scoring::calculate_weights_from_leaderboard(&entries);
    ok_response(bincode::serialize(&weights).unwrap_or_default())
}
