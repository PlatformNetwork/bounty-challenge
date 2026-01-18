# Validator Setup Guide

This guide explains how to run a Bounty Challenge validator.

## Overview

Validators perform two key functions:
1. **Auto-scan**: Periodically scan GitHub for new valid issues
2. **Credit rewards**: Link issues to registered miners and update weights

## Prerequisites

- **Server**: Linux with 1+ CPU, 2GB RAM
- **Network**: Outbound HTTPS access to GitHub API and Platform
- **Database**: PostgreSQL 14+ (or access to Platform's PostgreSQL)
- **Bittensor**: Validator hotkey registered on the subnet

## Installation

### 1. Build from Source

```bash
# Clone repository
git clone https://github.com/CortexLM/bounty-challenge.git
cd bounty-challenge

# Build release
cargo build --release

# Verify
./target/release/bounty-server --version
```

### 2. Set Environment Variables

```bash
# Required
export DATABASE_URL="postgres://user:pass@localhost:5432/bounty"
export GITHUB_TOKEN="ghp_xxxxxxxxxxxx"

# Optional
export CHALLENGE_HOST="0.0.0.0"
export CHALLENGE_PORT="8080"
export PLATFORM_URL="https://chain.platform.network"
export VALIDATOR_HOTKEY="5FHneW46..."
```

### 3. Initialize Database

The server automatically runs migrations on startup. Alternatively:

```bash
psql $DATABASE_URL < migrations/002_rewards_schema.sql
```

## Running the Validator

### Server Mode

Run the full server with auto-scanning:

```bash
./target/release/bounty-server
```

Or with explicit options:

```bash
./target/release/bounty server --host 0.0.0.0 --port 8080
```

### Validate Mode

Run validator-only mode (no HTTP server):

```bash
./target/release/bounty validate \
    --platform https://chain.platform.network \
    --hotkey $VALIDATOR_HOTKEY
```

## Configuration

### config.toml

```toml
[github]
client_id = "Ov23liAkfvnMhA6C68iy"

[[github.repos]]
owner = "CortexLM"
repo = "cortex"

[[github.repos]]
owner = "CortexLM"
repo = "fabric"

[server]
host = "0.0.0.0"
port = 8080

[database]
sqlite_path = "bounty.db"

[rewards]
max_issues_for_full_emission = 250
base_weight_per_issue = 0.01
adaptation_threshold = 100
valid_label = "valid"
```

### Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `DATABASE_URL` | Yes | PostgreSQL connection string |
| `GITHUB_TOKEN` | Yes | GitHub API token (for rate limits) |
| `PLATFORM_URL` | No | Platform server URL |
| `VALIDATOR_HOTKEY` | No | Your validator hotkey |
| `CHALLENGE_HOST` | No | Server bind address |
| `CHALLENGE_PORT` | No | Server port |

## Auto-Scan Process

### Scan Interval

By default, validators scan every **5 minutes**:

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│ Fetch closed │────▶│ Filter with  │────▶│ Credit to    │
│ issues       │     │ 'valid' label│     │ miners       │
└──────────────┘     └──────────────┘     └──────────────┘
```

### What Gets Scanned

1. **Target repos**: All repos in config.toml
2. **Issue state**: Only closed issues
3. **Label filter**: Only issues with `valid` label
4. **Time filter**: Since last scan timestamp

### Crediting Process

For each valid issue:
1. Check if already credited (by issue ID)
2. Look up GitHub username → hotkey mapping
3. Calculate weight at time of resolution
4. Record in `resolved_issues` table

## Monitoring

### Health Check

```bash
curl http://localhost:8080/health
```

Response:
```json
{
  "healthy": true,
  "uptime_secs": 3600,
  "version": "0.1.0"
}
```

### Stats

```bash
curl http://localhost:8080/stats
```

Response:
```json
{
  "total_bounties": 150,
  "total_miners": 25
}
```

### Logs

Run with verbose logging:

```bash
RUST_LOG=info ./target/release/bounty-server
```

Log levels:
- `error`: Critical errors only
- `warn`: Warnings and errors
- `info`: Normal operation (recommended)
- `debug`: Detailed debugging
- `trace`: Very verbose

## Systemd Service

### Create Service File

```bash
sudo nano /etc/systemd/system/bounty-validator.service
```

```ini
[Unit]
Description=Bounty Challenge Validator
After=network.target postgresql.service

[Service]
Type=simple
User=bounty
WorkingDirectory=/opt/bounty-challenge
ExecStart=/opt/bounty-challenge/target/release/bounty-server
Restart=always
RestartSec=10
Environment=DATABASE_URL=postgres://user:pass@localhost:5432/bounty
Environment=GITHUB_TOKEN=ghp_xxxx
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

### Enable and Start

```bash
sudo systemctl daemon-reload
sudo systemctl enable bounty-validator
sudo systemctl start bounty-validator
sudo systemctl status bounty-validator
```

### View Logs

```bash
sudo journalctl -u bounty-validator -f
```

## Docker Deployment

### Dockerfile

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/bounty-server /usr/local/bin/
EXPOSE 8080
CMD ["bounty-server"]
```

### Docker Compose

```yaml
version: '3.8'

services:
  bounty:
    build: .
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgres://bounty:secret@db:5432/bounty
      - GITHUB_TOKEN=${GITHUB_TOKEN}
      - RUST_LOG=info
    depends_on:
      - db

  db:
    image: postgres:14
    environment:
      - POSTGRES_USER=bounty
      - POSTGRES_PASSWORD=secret
      - POSTGRES_DB=bounty
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

### Run

```bash
GITHUB_TOKEN=ghp_xxx docker-compose up -d
```

## Troubleshooting

### "GitHub rate limit exceeded"

- **Cause**: Too many API requests without token
- **Fix**: Set `GITHUB_TOKEN` environment variable

### "Database connection failed"

- **Cause**: Invalid `DATABASE_URL` or PostgreSQL not running
- **Fix**: Verify connection string and database availability

### "No issues found"

- **Cause**: No closed issues with `valid` label
- **Fix**: Verify target repos have valid issues

### High Memory Usage

- **Cause**: Large number of issues being processed
- **Fix**: Increase server RAM or reduce scan batch size

## Security

### GitHub Token

- Use a token with minimal permissions (read-only)
- Store securely (environment variable, not in config)
- Rotate regularly

### Database

- Use strong passwords
- Enable SSL for remote connections
- Restrict network access

### Network

- Use HTTPS for all external communication
- Consider running behind a reverse proxy (nginx)
