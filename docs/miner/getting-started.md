# Getting Started

This guide will help you set up and start earning rewards with Bounty Challenge.

## Prerequisites

- **Bittensor Wallet**: You need a miner hotkey with its secret key
- **GitHub Account**: Any GitHub account
- **Rust 1.70+**: Required to build the CLI (or download pre-built binary)

## Installation

### Option 1: Build from Source

```bash
# Clone the repository
git clone https://github.com/PlatformNetwork/bounty-challenge.git
cd bounty-challenge

# Build in release mode
cargo build --release

# Add to PATH
export PATH="$PWD/target/release:$PATH"

# Verify installation
bounty --version
```

### Option 2: Download Pre-built Binary

```bash
# Download latest release (example for Linux x86_64)
curl -LO https://github.com/PlatformNetwork/bounty-challenge/releases/latest/download/bounty-linux-x86_64

# Make executable
chmod +x bounty-linux-x86_64

# Move to PATH
sudo mv bounty-linux-x86_64 /usr/local/bin/bounty

# Verify
bounty --version
```

## Quick Start

### 1. Register Your GitHub Account

Run the registration wizard:

```bash
bounty
```

You'll be prompted to enter:
1. Your miner **secret key** (64-char hex or 12-word mnemonic)
2. Your **GitHub username**

The wizard will sign the registration and submit it to the platform.

### 2. Create Issues in bounty-challenge

> **IMPORTANT**: Issues must be submitted to this repository to receive rewards!

Go to the bounty-challenge repository:

- **PlatformNetwork/bounty-challenge**: https://github.com/PlatformNetwork/bounty-challenge/issues

Create quality issues (bug reports, feature requests, security issues, documentation improvements). Each valid issue earns you **1 point** (50 points = 100% weight).

### 3. Wait for Validation

Maintainers will review your issue:
- ✅ Valid issue → Closed with `valid` label → **+1 point**
- ❌ Invalid issue → Marked with `invalid` label → **-2 points**
- ⏳ Closed without labels → No reward or penalty

### 4. Check Your Status

```bash
bounty status --hotkey YOUR_HOTKEY
```

### 5. View Leaderboard

```bash
bounty leaderboard
```

## Next Steps

- Read the [Registration Guide](registration.md) for detailed registration info
- Check the [Scoring Documentation](../reference/scoring.md) to understand rewards
- Review the [API Reference](../reference/api-reference.md) for programmatic access

## Common Issues

### "Connection refused" Error

Make sure you can reach the platform server:
```bash
curl https://chain.platform.network/health
```

### Invalid Secret Key

The CLI accepts:
- **64-char hex**: `a1b2c3d4...` (32 bytes as hex)
- **12+ word mnemonic**: `word1 word2 word3 ...`
- **SURI**: `//Alice` (for testing only)

### Registration Failed

- Check your internet connection
- Verify your secret key is correct
- Ensure your GitHub username is valid
