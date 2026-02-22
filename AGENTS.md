# Bounty Challenge - Development Guide

This document provides guidelines for developers and AI agents working on the Bounty Challenge codebase.

## Project Overview

Bounty Challenge is a WASM evaluation module for Platform Network. It runs inside the validator's WASM runtime (`wasm32-unknown-unknown`) and handles evaluation, validation, routing, and weight calculation on-chain. Miners earn rewards by discovering and reporting valid GitHub issues.

### Architecture

This is a `no_std` WASM crate that implements the `Challenge` trait from `platform-challenge-sdk-wasm`. The platform-v2 validator loads the compiled `.wasm` binary and invokes exported functions. All state is stored via host-provided key/value storage.

### Key Components

| File | Description |
|------|-------------|
| `src/lib.rs` | `BountyChallengeWasm` – `Challenge` trait implementation |
| `src/types.rs` | Domain types: `BountySubmission`, `IssueRecord`, `UserRegistration`, `LeaderboardEntry`, etc. |
| `src/storage/bounty_storage.rs` | On-chain key/value storage via `host_storage_get/set` |
| `src/scoring.rs` | Weight calculation (`WEIGHT_PER_POINT = 0.02`, `STAR_BONUS_PER_REPO = 0.25`) |
| `src/consensus.rs` | Validator consensus for issue validity and sync data |
| `src/validation.rs` | Issue validation and claim processing |
| `src/api/handlers.rs` | Route handlers (leaderboard, stats, register, claim, etc.) |
| `src/routes.rs` | Route definitions and request dispatch |

### Storage Key Patterns

`user:<hotkey>`, `github:<username>`, `issue:<owner>/<repo>:<number>`, `balance:<hotkey>`, `leaderboard`, `registered_hotkeys`, `synced_issues`, `timeout_config`

## Building

```bash
# Build WASM module
cargo build --release --target wasm32-unknown-unknown

# Output: target/wasm32-unknown-unknown/release/bounty_challenge.wasm
```

## Coding Guidelines

### Rust / WASM Constraints

1. **`#![no_std]`** – No standard library. Use `alloc` for `String`, `Vec`, etc.
2. **No I/O** – All storage and network access goes through `platform-challenge-sdk-wasm` host functions.
3. **Serialization** – Use `bincode` with size limits (`bincode::DefaultOptions::new().with_limit(...)`) for all deserialization.
4. **Error Handling** – Return `EvaluationOutput::failure(msg)` or empty `Vec` on errors. Never panic.
5. **No `unwrap()` in production paths** – Use `unwrap_or_default()`, `unwrap_or_else()`, or match.

### Platform SDK Integration

The crate depends on `platform-challenge-sdk-wasm` (local path: `../platform-v2/crates/challenge-sdk-wasm`).

Key traits and types:
- `Challenge` trait: `name()`, `version()`, `evaluate()`, `validate()`, `routes()`, `handle_route()`, `get_weights()`
- `register_challenge!` macro: exports WASM ABI functions
- Host functions: `host_storage_get()`, `host_storage_set()`, `host_consensus_get_epoch()`, `host_consensus_get_submission_count()`

### Project Structure

```
bounty-challenge/
├── Cargo.toml               # WASM crate config (cdylib + rlib)
├── Cargo.lock               # Dependency lockfile
├── src/
│   ├── lib.rs               # Challenge trait impl + register_challenge! macro
│   ├── types.rs             # Domain types
│   ├── scoring.rs           # Weight calculation
│   ├── consensus.rs         # Validator consensus
│   ├── validation.rs        # Issue validation and claim processing
│   ├── routes.rs            # Route definitions and dispatch
│   ├── api/
│   │   ├── mod.rs           # API module
│   │   └── handlers.rs      # Route handlers
│   └── storage/
│       ├── mod.rs           # Storage module
│       └── bounty_storage.rs # Key/value storage operations
├── .github/
│   └── workflows/
│       ├── ci.yml           # CI: build + clippy for wasm32
│       ├── protect-valid-label.yml  # Label protection
│       └── version-check.yml       # Issue version check
├── docs/                    # Documentation
├── assets/                  # Static assets
├── LICENSE
└── README.md
```

### API Routes

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/leaderboard` | No | Current standings |
| GET | `/stats` | No | Challenge statistics |
| GET | `/status/:hotkey` | No | Hotkey status |
| POST | `/register` | Yes | Register GitHub username |
| POST | `/claim` | Yes | Claim bounty for issues |
| GET | `/issues` | No | List synced issues |
| GET | `/issues/pending` | No | List pending issues |
| GET | `/hotkey/:hotkey` | No | Detailed hotkey info |
| POST | `/invalid` | Yes | Record invalid issue |
| POST | `/sync/propose` | Yes | Propose sync data |
| GET | `/sync/consensus` | No | Check sync consensus |
| POST | `/issue/propose` | Yes | Propose issue validity |
| POST | `/issue/consensus` | No | Check issue consensus |
| GET | `/config/timeout` | No | Get timeout config |
| POST | `/config/timeout` | Yes | Set timeout config |
| GET | `/get_weights` | No | Weight assignments |

### Weight Calculation

- 1 point per valid issue
- 0.25 points per starred repository
- Weight = `net_points × 0.02`
- Weights are normalized to sum to 1.0

### Code Quality

```bash
# Format
cargo fmt

# Lint (must target wasm32)
cargo clippy --target wasm32-unknown-unknown

# Check
cargo check --target wasm32-unknown-unknown
```
