--- BUILDER A ---
Score/Metrics: {'compute_impact': 'This fix reduces unnecessary computations by directly handling session expiration and redirecting to the login UI.', 'allocations_avoided': True}

pub fn handle_route_request(request: &WasmRouteRequest) -> WasmRouteResponse {
    let path = request.path.as_str();
    let method = request.method.as_str();
    let session_expired = request.session_expired;
    if session_expired {
        return WasmRouteResponse {
            status: 302,
            body: Vec::new(),
            redirect: Some(String::from("/login")),
        };
    }
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