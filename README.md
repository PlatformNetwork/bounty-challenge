<div align="center">

# bουηtү chαllεηgε

**GitHub Issue Reward System for Cortex on Bittensor**

[![CI](https://github.com/PlatformNetwork/bounty-challenge/actions/workflows/ci.yml/badge.svg)](https://github.com/PlatformNetwork/bounty-challenge/actions/workflows/ci.yml)
[![License](https://img.shields.io/github/license/PlatformNetwork/bounty-challenge)](https://github.com/PlatformNetwork/bounty-challenge/blob/main/LICENSE)
[![GitHub stars](https://img.shields.io/github/stars/PlatformNetwork/bounty-challenge)](https://github.com/PlatformNetwork/bounty-challenge/stargazers)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Bittensor](https://img.shields.io/badge/bittensor-subnet-green.svg)](https://bittensor.com/)

![Bounty Challenge Banner](assets/banner.jpg)

</div>

Bounty Challenge is a decentralized issue reward system on the Bittensor network. Miners earn TAO rewards by discovering and reporting valid issues. Issues must be closed with the `valid` label by project maintainers to qualify for rewards.

> **IMPORTANT**: To receive rewards, you MUST submit issues in **this repository** ([PlatformNetwork/bounty-challenge](https://github.com/PlatformNetwork/bounty-challenge/issues)). Issues submitted to other repositories (CortexLM/cortex, CortexLM/fabric, etc.) will **NOT** be counted for rewards.

## Quick Links

- [Getting Started](docs/miner/getting-started.md) - Installation and first registration
- [Registration Guide](docs/miner/registration.md) - Link your GitHub account
- [Scoring & Rewards](docs/reference/scoring.md) - Weight calculation formulas
- [API Reference](docs/reference/api-reference.md) - Endpoints and payloads
- [Validator Setup](docs/validator/setup.md) - Run a validator

## Features

- **Centralized Bug Bounty**: All issues tracked in this repository
- **Adaptive Rewards**: Dynamic weight calculation based on daily activity
- **Cryptographic Registration**: sr25519 signature-based hotkey linking
- **Real-Time Leaderboard**: Track miner standings and valid issues
- **PostgreSQL Backend**: Production-ready storage via Platform integration
- **GitHub Label Protection**: Automated label protection via GitHub Actions

## System Overview

### Core Components

```mermaid
flowchart LR
    Miner["🧑‍💻 Miner"] -->|"create issue"| GitHub["📋 GitHub Issues"]
    Validator["✅ Validator"] -->|"scan"| GitHub
    Validator -->|"submit weights"| Platform["🌐 Platform"]
```

### Registration Flow

```mermaid
flowchart LR
    A["1. Run CLI"] --> B["2. Enter key"] --> C["3. GitHub user"] --> D["4. Sign"] --> E["5. Verified"]
```

### Reward Flow

```mermaid
flowchart LR
    A["Create Issue"] --> B["Review"] --> C{Valid?}
    C -->|Yes| D["✅ Reward"]
    C -->|No| E["❌ No reward"]
```

## Reward System

Bounty Challenge uses an **adaptive reward system** that adjusts based on daily activity.

### Emission Rate

Maximum emission is reached at **250 issues per day**:

$$W_{max} = \min\left(\frac{N_{total}}{250}, 1.0\right)$$

| Daily Issues | Max Weight Available |
|--------------|---------------------|
| 50 | 0.20 (20%) |
| 100 | 0.40 (40%) |
| 250 | 1.00 (100%) |
| 500 | 1.00 (capped) |

### Adaptive Per-Issue Weight

Each resolved issue gives **0.01 weight** by default, but this adapts when activity is high:

$$w_{issue} = \begin{cases} 
0.01 & \text{if } N_{total} \leq 100 \\ 
0.01 \times \frac{100}{N_{total}} & \text{if } N_{total} > 100
\end{cases}$$

| Daily Issues | Weight per Issue |
|--------------|-----------------|
| 50 | 0.0100 |
| 100 | 0.0100 |
| 200 | 0.0050 |
| 500 | 0.0020 |

### User Weight Calculation

Your total weight is your issues multiplied by the current per-issue weight:

$$W_{user} = \min(n_{user} \times w_{issue}, W_{max})$$

**Example:** With 200 issues/day globally, if you resolve 10 issues:
- Weight per issue: 0.005
- Your weight: 10 × 0.005 = 0.05 (5%)

See [Scoring Documentation](docs/reference/scoring.md) for complete specifications.

## Quick Start for Miners

### Prerequisites

- **Bittensor Wallet** (miner hotkey with secret key)
- **GitHub Account** 
- **Rust** 1.70+ (to build the CLI)

### Installation

```bash
# Clone and build
git clone https://github.com/PlatformNetwork/bounty-challenge.git
cd bounty-challenge
cargo build --release

# Add to PATH
export PATH="$PWD/target/release:$PATH"

# Verify installation
bounty --version
```

### Register Your GitHub Account

Run the interactive registration wizard:

```bash
bounty
```

Or explicitly:

```bash
bounty wizard
```

The wizard will:
1. Ask for your miner **secret key** (64-char hex or 12+ word mnemonic)
2. Derive your **hotkey** (SS58 format)
3. Ask for your **GitHub username**
4. Sign the registration with sr25519
5. Submit to the platform

### Create Valid Issues

> **WARNING**: Issues must be created in **this repository** to be eligible for rewards!

Go to the bounty-challenge repository and create issues:

| Repository | URL |
|------------|-----|
| **PlatformNetwork/bounty-challenge** | https://github.com/PlatformNetwork/bounty-challenge/issues |

You can report issues about any Cortex project (Cortex CLI, Fabric, etc.) but they must be submitted HERE to count for rewards.

Valid issue types:

| Type | Description |
|------|-------------|
| **Bug Reports** | Reproduction steps, expected vs actual behavior |
| **Security Issues** | Vulnerabilities (follow responsible disclosure) |
| **Feature Requests** | Use cases and proposed solutions |
| **Documentation** | Gaps, errors, or improvements |

### Wait for Validation

Maintainers will review your issue:
- ✅ **Valid**: Closed with `valid` label → Reward auto-credited
- ❌ **Invalid**: Closed without label → No reward

**Note:** Only maintainers can add/remove the `valid` label. This is enforced via GitHub Actions.

### Check Your Status

```bash
bounty status --hotkey YOUR_HOTKEY
```

### View Leaderboard

```bash
bounty leaderboard
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `bounty` | Interactive registration wizard (default) |
| `bounty wizard` | Same as above |
| `bounty status -h <hotkey>` | Check your status and rewards |
| `bounty leaderboard` | View current standings |
| `bounty config` | Show challenge configuration |
| `bounty server` | Run in server mode (subnet operators) |
| `bounty validate` | Run as validator (auto-scan) |

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PLATFORM_URL` | `https://chain.platform.network` | Platform server URL |
| `DATABASE_URL` | - | PostgreSQL connection (server mode) |
| `GITHUB_TOKEN` | - | GitHub API token (increases rate limits) |
| `MINER_HOTKEY` | - | Your miner hotkey (SS58) |

## Where to Submit Issues

> **IMPORTANT**: All issues must be submitted to this repository to receive rewards.

| Repository | URL | Status |
|------------|-----|--------|
| **PlatformNetwork/bounty-challenge** | https://github.com/PlatformNetwork/bounty-challenge/issues | ✅ Rewards eligible |
| CortexLM/cortex | https://github.com/CortexLM/cortex | ❌ Not counted |
| CortexLM/fabric | https://github.com/CortexLM/fabric | ❌ Not counted |

Report bugs, security issues, or feature requests about ANY Cortex project in the bounty-challenge repo.

## Anti-Abuse Mechanisms

| Mechanism | Description |
|-----------|-------------|
| **Valid Label Required** | Only issues closed with `valid` label count |
| **Signature Verification** | sr25519 signature proves hotkey ownership |
| **Author Verification** | GitHub username must match issue author |
| **First Reporter Wins** | Each issue can only be claimed once |
| **Adaptive Weights** | High activity reduces per-issue reward |
| **Maintainer Gatekeeping** | Only project members can validate issues |
| **Label Protection** | GitHub Actions prevent unauthorized label changes |

## API Reference

### Bridge API Endpoints

All requests go through the platform bridge:

```
https://chain.platform.network/api/v1/bridge/bounty-challenge/
```

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/register` | POST | Register GitHub username with hotkey |
| `/status/{hotkey}` | GET | Get miner status and rewards |
| `/leaderboard` | GET | Get current standings |
| `/stats` | GET | Get challenge statistics |

### Direct Server Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check |
| `/config` | GET | Challenge configuration |
| `/get_weights` | GET | Calculate current weights |

See [API Reference](docs/reference/api-reference.md) for complete documentation.

## Project Structure

```
bounty-challenge/
├── src/
│   ├── main.rs              # Server entry point
│   ├── lib.rs               # Library exports
│   ├── challenge.rs         # Challenge implementation
│   ├── github.rs            # GitHub API client
│   ├── pg_storage.rs        # PostgreSQL storage
│   ├── storage.rs           # SQLite storage (CLI)
│   ├── server.rs            # HTTP server & routes
│   ├── discovery.rs         # Auto-scan for valid issues
│   └── bin/bounty/          # CLI application
│       ├── main.rs          # CLI entry point
│       ├── client.rs        # Bridge API client
│       ├── wizard/          # Registration wizard
│       └── commands/        # CLI commands
├── migrations/
│   ├── 001_initial.sql      # SQLite schema
│   └── 002_rewards_schema.sql # PostgreSQL schema
├── docs/
│   ├── miner/               # Miner guides
│   ├── reference/           # API references
│   └── validator/           # Validator guides
├── .github/workflows/
│   └── protect-valid-label.yml # Label protection
├── config.toml              # Configuration
└── assets/
    └── banner.jpg           # Banner image
```

## Documentation

- **For Miners:**
  - [Getting Started](docs/miner/getting-started.md)
  - [Registration Guide](docs/miner/registration.md)

- **For Validators:**
  - [Setup Guide](docs/validator/setup.md)

- **Reference:**
  - [Scoring & Rewards](docs/reference/scoring.md)
  - [API Reference](docs/reference/api-reference.md)

## Development

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=info cargo run
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Check without building
cargo check
```

## Platform Integration

When deployed as a Platform challenge module:

```mermaid
flowchart LR
    subgraph Miners
        Miner["🧑‍💻 Miner<br/>(CLI/wizard)"]
    end
    
    subgraph Platform["Platform Server<br/>chain.platform.network"]
        API["API Gateway"]
        DB[("PostgreSQL")]
        Bounty["Bounty Challenge<br/>(container)"]
        
        API --> DB
        Bounty --> DB
    end
    
    Miner -->|"register/status"| API
    API -->|"route"| Bounty
```

## Acknowledgments

- [Cortex Foundation](https://github.com/CortexLM) for the Cortex ecosystem
- [Platform Network](https://github.com/PlatformNetwork) for the challenge SDK
- [Bittensor](https://bittensor.com/) for the decentralized AI network

## License

Apache-2.0
