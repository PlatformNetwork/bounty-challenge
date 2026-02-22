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
# Set your validator RPC URL
export BOUNTY_RPC_URL=http://VALIDATOR_IP:8080

# Launch the CLI
bounty-cli
```

You'll see an interactive menu:

```
  bounty-challenge
  RPC: http://VALIDATOR_IP:8080

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

1. **Discover issues** in eligible repositories
2. **Submit issues** in the [bounty-challenge repository](https://github.com/PlatformNetwork/bounty-challenge/issues)
3. **Wait for review** — maintainers will close valid issues with the `valid` label

> **IMPORTANT**: Issues must be submitted in the bounty-challenge repository, not directly in the target repository.

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
curl http://VALIDATOR_IP:8080/challenge/bounty-challenge/leaderboard

# Get stats
curl http://VALIDATOR_IP:8080/challenge/bounty-challenge/stats

# Check status
curl http://VALIDATOR_IP:8080/challenge/bounty-challenge/status/YOUR_HOTKEY
```

## Tips

- **Quality over quantity** — Invalid issues incur penalties
- **Star eligible repos** — Each starred repo adds 0.25 bonus points
- **Check before submitting** — Duplicate issues also incur penalties
- **Use live dashboards** — Monitor your weight and ranking in real-time
