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
| **Config** | `src/config.rs` | Configuration loading |
| **Server Routes** | `src/server.rs` | HTTP routes and handlers |
| **Metagraph** | `src/metagraph.rs` | Metagraph caching |
| **CLI** | `src/bin/bounty/` | Command-line interface |

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
│       │   └── commands/    # CLI commands
│       └── bounty-health-server.rs  # Health check server
├── migrations/              # SQL migrations
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
- `[rewards]` - Points system parameters

Environment variables take precedence:
- `DATABASE_URL` - PostgreSQL connection string
- `GITHUB_TOKEN` - API authentication
- `PLATFORM_URL` - Platform server URL

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

### Public Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check |
| GET | `/config` | Challenge configuration |
| POST | `/register` | Register GitHub account |
| GET | `/status/:hotkey` | Get miner status |
| GET | `/leaderboard` | Current standings |
| GET | `/stats` | Challenge statistics |

### Weight Calculation

Points are calculated as:
- 1 point per valid issue
- 0.25 points per starred repository (max 5 repos)
- -0.5 penalty per invalid issue

Weight formula: `min(points × 0.02, 1.0)`

## Database Schema

Key tables:
- `github_registrations` - Hotkey ↔ GitHub username mappings
- `resolved_issues` - Valid issues credited to miners
- `invalid_issues` - Invalid issues (for penalty tracking)
- `target_repos` - Repositories to monitor

## Common Tasks

### Adding a New Endpoint

1. Define handler function in `src/server.rs`
2. Add route in `create_router()`
3. Update API documentation in `docs/reference/api-reference.md`

### Modifying Storage

1. Create new migration in `migrations/`
2. Update `PgStorage` methods in `src/pg_storage.rs`
3. Add migration check in `run_migrations()`

### Updating CLI

1. Add command variant to `Commands` enum in `src/bin/bounty/main.rs`
2. Implement handler in `src/bin/bounty/commands/`
3. Update CLI documentation in README

## Security Considerations

- All signatures use sr25519 (Substrate standard)
- Timestamps must be within 5 minutes (replay protection)
- Each GitHub username can only link to one hotkey
- Only maintainers can add the `valid` label
