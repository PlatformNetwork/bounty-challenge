use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use platform_challenge_sdk_wasm::{WasmRouteDefinition, WasmRouteRequest, WasmRouteResponse};

use crate::api::handlers;

/// Route definitions for the bounty challenge.
/// Note: Consensus is handled by platform-v2 via StorageProposal/StorageVote.
/// The WASM module only exposes data access routes.
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
            method: String::from("GET"),
            path: String::from("/issues/stats"),
            description: String::from(
                "Issue statistics: total, open, closed, valid, invalid, pending",
            ),
            requires_auth: false,
        },
        WasmRouteDefinition {
            method: String::from("GET"),
            path: String::from("/github/:username"),
            description: String::from("Get GitHub user details and their issues"),
            requires_auth: false,
        },
        WasmRouteDefinition {
            method: String::from("GET"),
            path: String::from("/get_weights"),
            description: String::from("Returns normalized weight assignments for all miners"),
            requires_auth: false,
        },
        WasmRouteDefinition {
            method: String::from("POST"),
            path: String::from("/sudo/bulk_migrate"),
            description: String::from(
                "Bulk register GitHub users linked to hotkeys (sudo owner only)",
            ),
            requires_auth: true,
        },
        WasmRouteDefinition {
            method: String::from("POST"),
            path: String::from("/sudo/register_user"),
            description: String::from("Register a single user with hotkey (sudo owner only)"),
            requires_auth: true,
        },
        WasmRouteDefinition {
            method: String::from("POST"),
            path: String::from("/sudo/sync_github"),
            description: String::from("Trigger GitHub issue sync manually (sudo owner only)"),
            requires_auth: true,
        },
        WasmRouteDefinition {
            method: String::from("POST"),
            path: String::from("/sudo/recount"),
            description: String::from("Recount all balances from stored issues (sudo owner only)"),
            requires_auth: true,
        },
        WasmRouteDefinition {
            method: String::from("POST"),
            path: String::from("/sudo/ban_user"),
            description: String::from("Ban a user from receiving rewards (sudo owner only)"),
            requires_auth: true,
        },
        WasmRouteDefinition {
            method: String::from("POST"),
            path: String::from("/sudo/unban_user"),
            description: String::from("Unban a user (sudo owner only)"),
            requires_auth: true,
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
        ("GET", "/issues/stats") => handlers::handle_issues_stats(request),
        ("GET", "/get_weights") => handlers::handle_get_weights(request),
        ("POST", "/sudo/bulk_migrate") => handlers::handle_sudo_bulk_migrate(request),
        ("POST", "/sudo/register_user") => handlers::handle_sudo_register_user(request),
        ("POST", "/sudo/sync_github") => handlers::handle_sudo_sync_github(request),
        ("POST", "/sudo/recount") => handlers::handle_sudo_recount(request),
        ("POST", "/sudo/ban_user") => handlers::handle_sudo_ban_user(request),
        ("POST", "/sudo/unban_user") => handlers::handle_sudo_unban_user(request),
        _ => {
            if method == "GET" {
                if path.starts_with("/status/") {
                    return handlers::handle_status(request);
                }
                if path.starts_with("/hotkey/") {
                    return handlers::handle_hotkey_details(request);
                }
                if path.starts_with("/github/") {
                    return handlers::handle_github_user(request);
                }
            }
            WasmRouteResponse {
                status: 404,
                body: Vec::new(),
            }
        }
    }
}
