#!/bin/bash
# Install git hooks for bounty-challenge

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(dirname "$SCRIPT_DIR")"

echo "Installing git hooks for bounty-challenge..."

# Configure git to use our hooks directory
git -C "$REPO_DIR" config core.hooksPath .githooks

# Make hooks executable
chmod +x "$SCRIPT_DIR/pre-commit"
chmod +x "$SCRIPT_DIR/pre-push"

echo "✅ Git hooks installed!"
echo ""
echo "The following checks will run:"
echo "  pre-commit: cargo fmt"
echo "  pre-push:   cargo check, clippy, test"
echo ""
echo "To bypass hooks (not recommended): git push --no-verify"
