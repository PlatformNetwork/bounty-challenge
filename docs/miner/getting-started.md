# Getting Started as a Miner

This guide walks you through setting up as a Bounty Challenge miner on Platform Network.

## Prerequisites

- A Bittensor wallet with a registered hotkey on the subnet
- A GitHub account
- Rust toolchain installed (`rustup`)

## Step 1: Build the CLI

```bash
git clone https://github.com/PlatformNetwork/bounty-challenge.git
cd bounty-challenge
cargo build --release -p bounty-cli
```

The CLI binary will be at `./target/release/bounty-cli`.

## Step 2: Register Your GitHub Username

Register your GitHub username with your Bittensor hotkey. This links your on-chain identity to your GitHub account.

```bash
./target/release/bounty-cli register \
  --hotkey YOUR_SS58_HOTKEY \
  --github YOUR_GITHUB_USERNAME \
  --signature YOUR_HEX_SIGNATURE \
  --timestamp UNIX_TIMESTAMP \
  --rpc-url http://VALIDATOR_IP:8080
```

The signature must be an sr25519 signature of the message:
```
register_github:{github_username_lowercase}:{unix_timestamp}
```

The timestamp must be within 5 minutes of the validator's server time.

See the [Registration Guide](registration.md) for detailed instructions on generating the signature.

## Step 3: Find and Report Issues

1. **Discover issues** in eligible repositories
2. **Submit issues** in the [bounty-challenge repository](https://github.com/PlatformNetwork/bounty-challenge/issues)
3. **Wait for review** — maintainers will close valid issues with the `valid` label

> **IMPORTANT**: Issues must be submitted in the bounty-challenge repository, not directly in the target repository.

## Step 4: Monitor Your Progress

```bash
# Check your status
./target/release/bounty-cli status \
  --hotkey YOUR_SS58_HOTKEY \
  --rpc-url http://VALIDATOR_IP:8080

# View the leaderboard
./target/release/bounty-cli leaderboard \
  --rpc-url http://VALIDATOR_IP:8080

# View challenge statistics
./target/release/bounty-cli stats \
  --rpc-url http://VALIDATOR_IP:8080
```

## Step 5: Earn Rewards

Rewards are calculated based on your weight:

| Source | Points |
|--------|--------|
| Valid Issue | 1 point |
| Starred Repo | 0.25 points |

**Weight formula**: `net_points × 0.02` (normalized across all miners)

See [Scoring & Rewards](../reference/scoring.md) for the complete specification.

## Chain RPC Access

The CLI communicates with Platform Network validators via JSON-RPC. You can also query the API directly:

### Using curl (HTTP REST)

```bash
# Get leaderboard
curl http://VALIDATOR_IP:8080/challenge/bounty-challenge/leaderboard

# Get stats
curl http://VALIDATOR_IP:8080/challenge/bounty-challenge/stats

# Check status
curl http://VALIDATOR_IP:8080/challenge/bounty-challenge/status/YOUR_HOTKEY
```

### Using JSON-RPC

```bash
curl -X POST http://VALIDATOR_IP:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "challenge_call",
    "params": {
      "challengeId": "bounty-challenge",
      "method": "GET",
      "path": "/leaderboard"
    },
    "id": 1
  }'
```

## Tips

- **Quality over quantity** — Invalid issues incur penalties
- **Star eligible repos** — Each starred repo adds 0.25 bonus points
- **Check before submitting** — Duplicate issues also incur penalties
- **Monitor your status** — Use the CLI to track your weight and ranking
