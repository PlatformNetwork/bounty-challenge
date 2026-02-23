# Getting Started as a Miner

This guide walks you through setting up as a Bounty Challenge miner on Platform Network.

## Prerequisites

- A Bittensor wallet with a registered hotkey on the subnet
- A GitHub account
- Rust toolchain installed (`rustup`)

## Step 1: Install the CLI

### Option A: Download via Platform CLI

```bash
platform download bounty-cli
```

### Option B: Build from Source

```bash
git clone https://github.com/PlatformNetwork/bounty-challenge.git
cd bounty-challenge
cargo build --release -p bounty-cli
```

The CLI binary will be at `./target/release/bounty-cli`.

## Step 2: Launch the Interactive CLI

```bash
# Launch the CLI (uses https://chain.platform.network by default)
# Or set a custom RPC URL: export BOUNTY_RPC_URL=https://custom-validator.com
bounty-cli
```

You'll see an interactive menu:

```
  bounty-challenge
  RPC: https://chain.platform.network

? Select an action ›
❯ Leaderboard        (live dashboard)
  Challenge Stats    (live dashboard)
  Weights            (live dashboard)
  My Status
  Issues
  Pending Issues
  Register
  Claim Bounty
  Change RPC URL
  Quit
```

## Step 3: Register Your GitHub Username

1. Select **Register** from the menu
2. Enter your SS58 hotkey
3. Enter your GitHub username
4. Provide your signature and timestamp

The signature must be an sr25519 signature of the message:
```
register_github:{github_username_lowercase}:{unix_timestamp}
```

See the [Registration Guide](registration.md) for detailed instructions on generating the signature.

## Step 4: Find and Report Issues

1. **Find bugs** in [CortexLM/cortex-ide](https://github.com/CortexLM/cortex-ide)
2. **Submit issues** in [PlatformNetwork/bounty-challenge](https://github.com/PlatformNetwork/bounty-challenge/issues) (this repo!)
3. **Include screenshots/videos** demonstrating the bug
4. **Wait for review** — maintainers will close valid issues with `ide` + `valid` labels

> **IMPORTANT**: 
> - Look for bugs in **CortexLM/cortex-ide**
> - Submit issues in **PlatformNetwork/bounty-challenge** (NOT in cortex-ide)
> - Issues must have BOTH `ide` AND `valid` labels to qualify

## Step 5: Claim Your Bounty

1. Select **Claim Bounty** from the menu
2. Enter the issue numbers you want to claim
3. The CLI will submit your claim to the network

## Step 6: Monitor Your Progress

Use the live dashboards from the main menu:

| Dashboard | Description |
|-----------|-------------|
| **Leaderboard** | Real-time rankings with scores (auto-refresh) |
| **Challenge Stats** | Total bounties, active miners, validators |
| **Weights** | Current weight assignments for rewards |
| **My Status** | Your registration, issues, and weight |

Press `q` to exit any dashboard and return to the menu.

## Rewards

Rewards are calculated based on your weight:

| Source | Points |
|--------|--------|
| Valid Issue | 1 point |
| Starred Repo | 0.25 points |

**Weight formula**: `net_points × 0.02` (normalized across all miners)

See [Scoring & Rewards](../reference/scoring.md) for the complete specification.

## Direct API Access

You can also query the API directly via curl:

```bash
# Get leaderboard
curl https://chain.platform.network/challenge/bounty-challenge/leaderboard

# Get stats
curl https://chain.platform.network/challenge/bounty-challenge/stats

# Check status
curl https://chain.platform.network/challenge/bounty-challenge/status/YOUR_HOTKEY
```

## Tips

- **Quality over quantity** — Invalid issues incur penalties
- **Star eligible repos** — Each starred repo adds 0.25 bonus points
- **Check before submitting** — Duplicate issues also incur penalties
- **Use live dashboards** — Monitor your weight and ranking in real-time
