# Getting Started

This guide walks you through setting up Bounty Challenge and claiming your first bounty.

## Prerequisites

- **Rust** 1.70 or later
- **SQLite** (bundled with the project)
- **GitHub Account** linked to your miner hotkey

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/CortexLM/bounty-challenge.git
cd bounty-challenge

# Build in release mode
cargo build --release

# The binary is at ./target/release/bounty-server
```

### Docker

```bash
docker build -t bounty-challenge .
docker run -p 8080:8080 -v ./data:/data bounty-challenge
```

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `GITHUB_TOKEN` | - | GitHub personal access token for API calls |
| `BOUNTY_DB_PATH` | `bounty.db` | Path to SQLite database |
| `CHALLENGE_HOST` | `0.0.0.0` | Server bind address |
| `CHALLENGE_PORT` | `8080` | Server port |
| `MAX_CONCURRENT` | `4` | Max concurrent evaluations |

### GitHub Token

While optional, a GitHub token significantly increases API rate limits:

1. Go to [GitHub Settings > Developer settings > Personal access tokens](https://github.com/settings/tokens)
2. Generate a new token (classic) with `public_repo` scope
3. Set the environment variable:

```bash
export GITHUB_TOKEN="ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
```

## Starting the Server

```bash
# Basic start
./target/release/bounty-server

# With environment variables
GITHUB_TOKEN="ghp_xxx" CHALLENGE_PORT=9000 ./target/release/bounty-server

# With logging
RUST_LOG=info ./target/release/bounty-server
```

## Your First Bounty

### Step 1: Register Your GitHub Username

Link your GitHub account to your miner hotkey:

```bash
curl -X POST http://localhost:8080/evaluate \
  -H "Content-Type: application/json" \
  -d '{
    "request_id": "register-1",
    "submission_id": "sub-1",
    "participant_id": "YOUR_MINER_HOTKEY",
    "epoch": 1,
    "data": {
      "action": "register",
      "github_username": "your-github-username"
    }
  }'
```

Response:
```json
{
  "request_id": "register-1",
  "success": true,
  "score": 1.0,
  "results": {
    "registered": true,
    "github_username": "your-github-username"
  }
}
```

### Step 2: Create a Valid Issue

1. Go to [CortexLM/fabric/issues/new](https://github.com/CortexLM/fabric/issues/new)
2. Create a detailed issue:
   - **Bug reports**: Include reproduction steps, expected vs actual behavior
   - **Feature requests**: Describe the use case and proposed solution
   - **Documentation**: Point out gaps or errors
3. Wait for maintainers to review

### Step 3: Get Your Issue Validated

Maintainers will review your issue:
- If valid, they close it with the `valid` label
- If invalid, they may close it without the label or request more info

### Step 4: Claim Your Bounty

Once your issue has the `valid` label:

```bash
curl -X POST http://localhost:8080/evaluate \
  -H "Content-Type: application/json" \
  -d '{
    "request_id": "claim-1",
    "submission_id": "sub-2",
    "participant_id": "YOUR_MINER_HOTKEY",
    "epoch": 1,
    "data": {
      "action": "claim",
      "github_username": "your-github-username",
      "issue_numbers": [42]
    }
  }'
```

Response:
```json
{
  "request_id": "claim-1",
  "success": true,
  "score": 0.1,
  "results": {
    "claimed": [{"issue_number": 42, "issue_url": "https://..."}],
    "rejected": [],
    "total_valid": 1,
    "score": 0.1
  }
}
```

## Checking Your Standing

View the leaderboard:

```bash
curl -X POST http://localhost:8080/evaluate \
  -H "Content-Type: application/json" \
  -d '{
    "request_id": "lb-1",
    "submission_id": "sub-3",
    "participant_id": "anyone",
    "epoch": 1,
    "data": {
      "action": "leaderboard"
    }
  }'
```

## Next Steps

- Read the [API Reference](api-reference.md) for all endpoints
- Understand [Scoring](scoring.md) to optimize your strategy
- Learn about [Anti-Abuse](anti-abuse.md) measures
