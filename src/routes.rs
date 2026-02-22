use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use platform_challenge_sdk_wasm::{WasmRouteDefinition, WasmRouteRequest, WasmRouteResponse};

use crate::api::handlers;

pub fn get_route_definitions() -> Vec<WasmRouteDefinition> {
    vec![
        WasmRouteDefinition {
            method: String::from("GET"),
            path: String::from("/leaderboard"),
            description: String::from("Returns current leaderboard with scores and rankings"),
            requires_auth: false,
        },
        WasmRouteDefinition {
            method: String::from("GET"),
            path: String::from("/stats"),
            description: String::from("Challenge statistics: total bounties, active miners"),
            requires_auth: false,
        },
        WasmRouteDefinition {
            method: String::from("GET"),
            path: String::from("/status/:hotkey"),
            description: String::from("Get status for a specific hotkey"),
            requires_auth: false,
        },
        WasmRouteDefinition {
            method: String::from("POST"),
            path: String::from("/register"),
            description: String::from("Register GitHub username with hotkey (requires auth)"),
            requires_auth: true,
        },
        WasmRouteDefinition {
            method: String::from("POST"),
            path: String::from("/claim"),
            description: String::from("Claim bounty for resolved issues (requires auth)"),
            requires_auth: true,
        },
        WasmRouteDefinition {
            method: String::from("GET"),
            path: String::from("/issues"),
            description: String::from("List all synced issues"),
            requires_auth: false,
        },
        WasmRouteDefinition {
            method: String::from("GET"),
            path: String::from("/issues/pending"),
            description: String::from("List pending issues"),
            requires_auth: false,
        },
        WasmRouteDefinition {
            method: String::from("GET"),
            path: String::from("/hotkey/:hotkey"),
            description: String::from("Detailed hotkey information"),
            requires_auth: false,
        },
        WasmRouteDefinition {
            method: String::from("POST"),
            path: String::from("/invalid"),
            description: String::from("Record an invalid issue (requires auth)"),
            requires_auth: true,
        },
        WasmRouteDefinition {
            method: String::from("POST"),
            path: String::from("/sync/propose"),
            description: String::from("Propose synced issue data for consensus (requires auth)"),
            requires_auth: true,
        },
        WasmRouteDefinition {
            method: String::from("GET"),
            path: String::from("/sync/consensus"),
            description: String::from("Check sync consensus status"),
            requires_auth: false,
        },
        WasmRouteDefinition {
            method: String::from("POST"),
            path: String::from("/issue/propose"),
            description: String::from("Propose issue validity for consensus (requires auth)"),
            requires_auth: true,
        },
        WasmRouteDefinition {
            method: String::from("POST"),
            path: String::from("/issue/consensus"),
            description: String::from("Check issue validity consensus"),
            requires_auth: false,
        },
        WasmRouteDefinition {
            method: String::from("GET"),
            path: String::from("/config/timeout"),
            description: String::from("Returns current timeout configuration"),
            requires_auth: false,
        },
        WasmRouteDefinition {
            method: String::from("POST"),
            path: String::from("/config/timeout"),
            description: String::from("Updates timeout configuration (requires auth)"),
            requires_auth: true,
        },
        WasmRouteDefinition {
            method: String::from("GET"),
            path: String::from("/get_weights"),
            description: String::from("Returns normalized weight assignments for all miners"),
            requires_auth: false,
        },
    ]
}

pub fn handle_route_request(request: &WasmRouteRequest) -> WasmRouteResponse {
    let path = request.path.as_str();
    let method = request.method.as_str();

    match (method, path) {
        ("GET", "/leaderboard") => handlers::handle_leaderboard(request),
        ("GET", "/stats") => handlers::handle_stats(request),
        ("POST", "/register") => handlers::handle_register(request),
        ("POST", "/claim") => handlers::handle_claim(request),
        ("GET", "/issues") => handlers::handle_issues(request),
        ("GET", "/issues/pending") => handlers::handle_issues_pending(request),
        ("POST", "/invalid") => handlers::handle_invalid(request),
        ("POST", "/sync/propose") => handlers::handle_sync_propose(request),
        ("GET", "/sync/consensus") => handlers::handle_sync_consensus(request),
        ("POST", "/issue/propose") => handlers::handle_issue_propose(request),
        ("POST", "/issue/consensus") => handlers::handle_issue_consensus(request),
        ("GET", "/config/timeout") => handlers::handle_get_timeout_config(request),
        ("POST", "/config/timeout") => handlers::handle_set_timeout_config(request),
        ("GET", "/get_weights") => handlers::handle_get_weights(request),
        _ => {
            if method == "GET" {
                if path.starts_with("/status/") {
                    return handlers::handle_status(request);
                }
                if path.starts_with("/hotkey/") {
                    return handlers::handle_hotkey_details(request);
                }
            }
            WasmRouteResponse {
                status: 404,
                body: Vec::new(),
            }
        }
    }
}
