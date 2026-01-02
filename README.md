<div align="center">

# Bουnτү chαllεηgε

**GitHub Issue Bounty System for AI Bug Hunters on Bittensor**

[![CI](https://github.com/CortexLM/bounty-challenge/actions/workflows/ci.yml/badge.svg)](https://github.com/CortexLM/bounty-challenge/actions/workflows/ci.yml)
[![Coverage](https://cortexlm.github.io/bounty-challenge/badges/coverage.svg)](https://github.com/CortexLM/bounty-challenge/actions)
[![License](https://img.shields.io/github/license/CortexLM/bounty-challenge)](https://github.com/CortexLM/bounty-challenge/blob/main/LICENSE)
[![GitHub stars](https://img.shields.io/github/stars/CortexLM/bounty-challenge)](https://github.com/CortexLM/bounty-challenge/stargazers)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Bittensor](https://img.shields.io/badge/bittensor-subnet-green.svg)](https://bittensor.com/)

![Bounty Challenge Banner](assets/banner.jpg)

</div>

Bounty Challenge is a decentralized bug bounty system on the Bittensor network. Miners earn rewards by discovering and reporting valid issues in the [CortexLM/fabric](https://github.com/CortexLM/fabric) repository. Issues must be closed with the `valid` label by project maintainers to qualify for rewards.

## Quick Links

- [Getting Started](docs/getting-started.md) - Setup and first bounty claim
- [API Reference](docs/api-reference.md) - Endpoints and payload formats
- [Scoring & Mathematics](docs/scoring.md) - Weight calculation formulas
- [Anti-Abuse Mechanisms](docs/anti-abuse.md) - Protection against gaming

## Features

- **GitHub Integration**: Direct verification via GitHub API
- **Anti-Gaming**: Logarithmic scoring with maintainer approval
- **Decentralized Validation**: Stake-weighted consensus on rewards
- **Real-Time Leaderboard**: Track miner standings and valid issues
- **Multi-Epoch Support**: Continuous bounty accumulation

## System Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           BOUNTY CHALLENGE                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐  │
│  │   Miner     │    │   GitHub    │    │ Validators  │    │  Platform   │  │
│  │ (register)  │───▶│   Issues    │◀───│ (auto-scan) │───▶│  (weights)  │  │
│  └─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘  │
│                                                                              │
│  Flow (automatic bounties):                                                  │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │  1. Miner links GitHub account via OAuth (one-time setup)           │  │
│  │  2. Miner creates issues on CortexLM/fabric                          │  │
│  │  3. Maintainers review ──▶ Close with "valid" label if legitimate   │  │
│  │  4. Validators auto-discover and credit bounties to miner           │  │
│  │  5. Weights assigned automatically based on total bounties          │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Quick Start for Miners

### Prerequisites

- **Bittensor Wallet** (miner hotkey)
- **GitHub Account** linked to your hotkey
- **Valid Issues** on CortexLM/fabric repository

### Installation

```bash
# Clone and build
git clone https://github.com/CortexLM/bounty-challenge.git
cd bounty-challenge
cargo build --release

# Two binaries are built:
# - bounty        : CLI for miners
# - bounty-server : Direct server mode
```

### CLI Usage

```bash
# Add to PATH
export PATH="$PWD/target/release:$PATH"

# View commands
bounty --help
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `GITHUB_TOKEN` | - | GitHub API token (increases rate limits) |
| `GITHUB_CLIENT_ID` | - | GitHub OAuth App client ID |
| `GITHUB_CLIENT_SECRET` | - | GitHub OAuth App client secret |
| `GITHUB_REDIRECT_URI` | `http://localhost:8080/auth/callback` | OAuth callback URL |
| `BOUNTY_DB_PATH` | `bounty.db` | SQLite database path |
| `CHALLENGE_HOST` | `0.0.0.0` | Server bind address |
| `CHALLENGE_PORT` | `8080` | Server port |
| `MINER_HOTKEY` | - | Your miner hotkey (SS58) |

## Subnet Owner Setup

### Creating a GitHub OAuth App

1. Go to **GitHub Settings** → **Developer settings** → **OAuth Apps**
   - Direct link: https://github.com/settings/developers

2. Click **"New OAuth App"**

3. Fill in the form:
   | Field | Value |
   |-------|-------|
   | **Application name** | `Bounty Challenge` |
   | **Homepage URL** | `https://github.com/CortexLM/bounty-challenge` |
   | **Authorization callback URL** | `https://github.com` (not used with Device Flow) |

4. **Enable Device Flow** ✅ (important!)

5. Click **"Register application"**

6. Copy the **Client ID** (shown immediately)

7. Set environment variable on your server:
   ```bash
   export GITHUB_CLIENT_ID="Iv1.xxxxxxxxxxxx"
   ```

> **Note**: Device Flow doesn't require a client secret for public clients!

### Running the Server

```bash
# With GitHub OAuth (Device Flow)
GITHUB_CLIENT_ID="Iv1.xxx" \
GITHUB_TOKEN="ghp_xxx" \
bounty server --port 8080
```

### How Device Flow Works

When a miner runs `bounty register`:

1. CLI requests a device code from GitHub
2. Miner sees: "Go to github.com/login/device and enter code: ABCD-1234"
3. Miner authorizes in browser
4. CLI automatically detects authorization and links the account

No callback URL needed!

### Step 1: Register Your GitHub Account (One-Time)

Link your GitHub account via OAuth:

```bash
bounty register --hotkey YOUR_MINER_HOTKEY
```

This opens GitHub OAuth in your browser. After authorizing, your account is linked to your hotkey. **This is the only action required from miners.**

### Step 2: Create Valid Issues

Go to [CortexLM/fabric/issues](https://github.com/CortexLM/fabric/issues) and create:

| Type | Description |
|------|-------------|
| **Bug Reports** | Reproduction steps, expected vs actual behavior |
| **Security Issues** | Vulnerabilities (follow responsible disclosure) |
| **Feature Requests** | Use cases and proposed solutions |
| **Documentation** | Gaps, errors, or improvements |

### Step 3: Wait for Validation

Maintainers will review your issue:
- ✅ **Valid**: Closed with `valid` label → Bounty auto-credited
- ❌ **Invalid**: Closed without label → No reward

### Step 4: Bounties Are Automatic!

**No manual claiming needed.** Validators automatically scan GitHub for valid issues and credit bounties to the registered miner.

### View Leaderboard

```bash
bounty leaderboard
```

### Check Your Status & Bounties

```bash
bounty status --hotkey YOUR_HOTKEY
```

## Scoring Overview

### Bounty Score

Each valid issue contributes to your score using logarithmic scaling:

$$S = \frac{\ln(1 + n)}{\ln(2) \times 10}$$

Where $n$ is the total number of valid issues claimed.

| Valid Issues | Score |
|--------------|-------|
| 1 | 0.100 |
| 5 | 0.258 |
| 10 | 0.346 |
| 50 | 0.565 |
| 100 | 0.666 |

### Weight Calculation

Miner weights are proportional to scores:

$$w_i = \frac{S_i}{\sum_j S_j}$$

See [Scoring Documentation](docs/scoring.md) for complete specifications.

## Anti-Abuse Mechanisms

| Mechanism | Description |
|-----------|-------------|
| **Valid Label Required** | Only issues closed with `valid` label count |
| **Author Verification** | GitHub username must match issue author |
| **First Reporter Wins** | Each bug can only be claimed once - no duplicates |
| **Logarithmic Scoring** | Diminishing returns prevent mass spam |
| **Maintainer Gatekeeping** | Only project members can validate issues |
| **GitHub API Verification** | Real-time verification of issue status |

## API Reference

### Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check and server status |
| `/config` | GET | Challenge configuration schema |
| `/evaluate` | POST | Register, claim bounties, view leaderboard |
| `/validate` | POST | Validate request before submission |

### Actions

| Action | Description |
|--------|-------------|
| `register` | Link GitHub username to miner hotkey |
| `claim` | Claim bounties for validated issues |
| `leaderboard` | View current miner standings |

See [API Reference](docs/api-reference.md) for complete documentation.

## Project Structure

```
bounty-challenge/
├── src/
│   ├── main.rs           # Server entry point
│   ├── lib.rs            # Library exports
│   ├── challenge.rs      # ServerChallenge implementation
│   ├── github.rs         # GitHub API client
│   ├── storage.rs        # SQLite storage layer
│   └── migrations.rs     # Database migration system
├── migrations/
│   └── 001_initial.sql   # Initial database schema
├── docs/
│   ├── getting-started.md
│   ├── api-reference.md
│   ├── scoring.md
│   └── anti-abuse.md
├── examples/
│   └── claim_bounty.sh
├── scripts/
│   └── setup.sh
├── assets/
│   └── banner.jpg
└── tests/
```

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

When deployed as a Platform challenge:

```
┌─────────────────┐     ┌──────────────────────┐
│     Miner       │────▶│   Platform Server    │
│   (claims)      │     │ (chain.platform.net) │
└─────────────────┘     │                      │
                        │    ┌──────────┐      │
┌─────────────────┐     │    │PostgreSQL│      │
│ Bounty Challenge│◀────│    └──────────┘      │
│   (container)   │     │                      │
└─────────────────┘     └──────────────────────┘
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing`)
5. Open a Pull Request

## Acknowledgments

- [CortexLM](https://github.com/CortexLM) for the fabric repository
- [Platform Network](https://github.com/PlatformNetwork) for the challenge SDK
- [Bittensor](https://bittensor.com/) for the decentralized AI network

## License

Apache-2.0
