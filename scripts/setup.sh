#!/bin/bash
# Development setup script for Bounty Challenge
#
# Usage: ./scripts/setup.sh

set -e

echo "=== Bounty Challenge Setup ==="
echo ""

# Check Rust
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    echo "✅ Rust: $RUST_VERSION"
else
    echo "❌ Rust not found. Install from https://rustup.rs/"
    exit 1
fi

# Check Cargo
if command -v cargo &> /dev/null; then
    CARGO_VERSION=$(cargo --version)
    echo "✅ Cargo: $CARGO_VERSION"
else
    echo "❌ Cargo not found"
    exit 1
fi

# Build project
echo ""
echo "Building project..."
cargo build --release

echo ""
echo "✅ Build successful!"
echo ""
echo "Binary location: ./target/release/bounty-server"
echo ""
echo "To run the server:"
echo "  ./target/release/bounty-server"
echo ""
echo "Optional environment variables:"
echo "  GITHUB_TOKEN=ghp_xxx     # GitHub API token (higher rate limits)"
echo "  BOUNTY_DB_PATH=bounty.db # Database path"
echo "  CHALLENGE_PORT=8080      # Server port"
echo ""
echo "Example with all options:"
echo "  GITHUB_TOKEN=ghp_xxx CHALLENGE_PORT=9000 ./target/release/bounty-server"
