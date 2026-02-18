# Bounty Challenge - Development Guide

This document provides guidelines for developers and AI agents working on the Bounty Challenge codebase.

## Project Overview

Bounty Challenge is a decentralized issue reward system on the Bittensor network. Miners earn TAO rewards by discovering and reporting valid issues.

### Key Components

| Component | Path | Description |
|-----------|------|-------------|
| **Server** | `src/main.rs` | HTTP server entry point |
| **Challenge** | `src/challenge.rs` | Core challenge implementation |
| **Storage** | `src/pg_storage.rs` | PostgreSQL data layer |
| **Auth** | `src/auth.rs` | SS58/sr25519 signature verification |
| **GitHub API** | `src/github.rs` | GitHub API client |
| **GitHub CLI** | `src/gh_cli.rs` | `gh` CLI wrapper for reliable sync |
| **GitHub OAuth** | `src/github_oauth.rs` | GitHub Device Flow OAuth |
| **Config** | `src/config.rs` | Configuration loading |
| **Server Routes** | `src/server.rs` | HTTP routes and handlers |
| **Metagraph** | `src/metagraph.rs` | Metagraph caching |
| **CLI** | `src/bin/bounty/` | Command-line interface |
| **Health Server** | `src/bin/bounty-health-server.rs` | Standalone health check server |

## Coding Guidelines

### Rust Best Practices

1. **Error Handling**
   - Use `Result` types and the `?` operator for propagation
   - Use `unwrap_or_else` with fallbacks, never bare `unwrap()` in production code
   - Log errors with `tracing` before returning them

2. **Async Code**
   - All database and network operations are async
   - Use `tokio` runtime with `async-trait` for trait implementations
   - Respect timeouts (30s default for DB queries)

3. **Security**
   - Never hardcode secrets - use environment variables
   - All user inputs must be validated before use
   - Use parameterized SQL queries only

### Project Structure

```
bounty-challenge/
├── src/
│   ├── main.rs              # Server entry point
│   ├── lib.rs               # Library exports
│   ├── auth.rs              # Signature verification
│   ├── challenge.rs         # Challenge implementation
│   ├── config.rs            # Configuration loading
│   ├── server.rs            # HTTP routes and handlers
│   ├── pg_storage.rs        # PostgreSQL storage
│   ├── github.rs            # GitHub API client
│   ├── gh_cli.rs            # GitHub CLI (gh) wrapper
│   ├── github_oauth.rs      # GitHub Device Flow OAuth
│   ├── metagraph.rs         # Metagraph caching
│   └── bin/
│       ├── bounty/          # CLI application
│       │   ├── main.rs      # CLI entry point
│       │   ├── client.rs    # Bridge API client
│       │   ├── style.rs     # Terminal styling
│       │   ├── wizard/      # Registration wizard
│       │   └── commands/    # CLI commands (config, info, leaderboard, server, status, validate)
│       └── bounty-health-server.rs  # Health check server
├── migrations/              # SQL migrations (001–018)
├── scripts/
│   └── setup.sh             # Setup script
├── docs/
│   ├── anti-abuse.md        # Anti-abuse documentation
│   ├── miner/               # Miner guides
│   ├── reference/           # API & scoring references
│   └── validator/           # Validator guides
├── examples/                # Example code
├── config.toml              # Configuration file
└── Dockerfile               # Container build
```

### Configuration

Configuration is loaded from `config.toml`:
- `[github]` - OAuth client ID and target repositories
- `[server]` - Host and port bindings
- `[database]` - PostgreSQL settings (uses `DATABASE_URL` env var)
- `[rewards]` - Points system parameters (`max_points_for_full_weight`, `weight_per_point`, `valid_label`)

Environment variables take precedence:
- `DATABASE_URL` - PostgreSQL connection string
- `GITHUB_TOKEN` - API authentication
- `PLATFORM_URL` - Platform server URL
- `GITHUB_CLIENT_ID` - OAuth client ID override

### Testing

```bash
# Run all tests
cargo test

# Run with logging
RUST_LOG=debug cargo test

# Check code quality
cargo clippy
cargo fmt --check
```

### Building

```bash
# Development build
cargo build

# Release build
cargo build --release

# Docker build
docker build -t bounty-challenge .
```

## API Endpoints

### Platform SDK Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check |
| GET | `/config` | Challenge configuration |
| POST | `/evaluate` | Evaluate miner submissions |
| POST | `/validate` | Validate submissions |
| GET | `/leaderboard` | Current standings |
| GET | `/get_weights` | Platform-compatible weight calculation |

### Direct Access Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/register` | Register GitHub account with hotkey |
| GET | `/status/:hotkey` | Get miner status |
| GET | `/stats` | Challenge statistics |
| POST | `/invalid` | Record an invalid issue |
| GET | `/issues` | List issues (with `state`, `label`, `limit`, `offset` query params) |
| GET | `/issues/pending` | List pending issues |
| GET | `/issues/stats` | Issue statistics |
| GET | `/hotkey/:hotkey` | Detailed hotkey info (also accepts GitHub username) |
| GET | `/github/:username` | GitHub user details |
| GET | `/sync/status` | Sync status for repos |
| POST | `/sync/trigger` | Trigger manual sync (authenticated) |

All direct access endpoints are also available under `/api/v1/` prefix for platform bridge compatibility.

### Weight Calculation

Points are calculated as:
- 1 point per valid issue
- 0.25 points per starred repository (no cap)
- Separate penalties: `max(0, invalid_count - valid_count) + max(0, duplicate_count - valid_count)`

Weight formula: `net_points × 0.02` (raw weight, no cap per user). Weights are normalized to sum to 1.0 at the API level when served via `/get_weights`.

## Database Schema

Key tables:
- `github_registrations` - Hotkey ↔ GitHub username mappings
- `resolved_issues` - Valid issues credited to miners
- `invalid_issues` - Invalid issues (for penalty tracking)
- `duplicate_issues` - Duplicate issue tracking (for penalty)
- `target_repos` - Repositories to monitor
- `github_issues` - Cached GitHub issues
- `github_sync_state` - Sync state per repository
- `github_stars` - User star tracking
- `star_target_repos` - Repos tracked for star bonuses
- `admin_bonuses` - Admin-granted bonus points
- `project_tags` - Project tag metadata
- `reward_snapshots` - Historical weight snapshots
- `daily_stats` - Aggregated daily statistics
- `schema_migrations` - Migration version tracking

## CLI Commands

The `bounty` binary supports these subcommands:

| Command | Aliases | Description |
|---------|---------|-------------|
| `wizard` | `w`, `register`, `r` | Interactive registration wizard (default) |
| `server` | `s` | Run as server (for subnet operators) |
| `validate` | `v` | Run as validator (auto-discovers bounties) |
| `leaderboard` | `lb` | View the leaderboard |
| `status` | `st` | Check your status and bounties |
| `config` | — | Show challenge configuration |
| `info` | `i` | Display system information for bug reports |

## Common Tasks

### Adding a New Endpoint

1. Define handler function in `src/server.rs`
2. Add route in `create_router()`
3. Update API documentation in `docs/reference/api-reference.md`

### Modifying Storage

1. Create new migration in `migrations/` (next sequential number)
2. Update `PgStorage` methods in `src/pg_storage.rs`
3. Add migration check in `run_migrations()`

### Updating CLI

1. Add command variant to `Commands` enum in `src/bin/bounty/main.rs`
2. Implement handler in `src/bin/bounty/commands/`
3. Register module in `src/bin/bounty/commands/mod.rs`
4. Update CLI documentation in README

## Security Considerations

- All signatures use sr25519 (Substrate standard)
- Timestamps must be within 5 minutes (replay protection)
- Each GitHub username can only link to one hotkey
- Each hotkey can only link to one GitHub username
- Only maintainers can add the `valid` label
