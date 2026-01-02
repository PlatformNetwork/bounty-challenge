#!/bin/bash
# Register a GitHub username with a miner hotkey
#
# Usage: ./register.sh <hotkey> <github_username>
# Example: ./register.sh 5GrwvaEF... octocat

set -e

CHALLENGE_URL="${CHALLENGE_URL:-http://localhost:8080}"
HOTKEY="$1"
GITHUB_USER="$2"

if [ -z "$HOTKEY" ] || [ -z "$GITHUB_USER" ]; then
    echo "Usage: $0 <hotkey> <github_username>"
    echo "Example: $0 5GrwvaEF... octocat"
    exit 1
fi

echo "=== Bounty Challenge Registration ==="
echo "Hotkey: $HOTKEY"
echo "GitHub: $GITHUB_USER"
echo ""

RESPONSE=$(curl -s -X POST "$CHALLENGE_URL/evaluate" \
    -H "Content-Type: application/json" \
    -d "{
        \"request_id\": \"reg-$(date +%s)\",
        \"submission_id\": \"sub-$(date +%s)\",
        \"participant_id\": \"$HOTKEY\",
        \"epoch\": 1,
        \"data\": {
            \"action\": \"register\",
            \"github_username\": \"$GITHUB_USER\"
        }
    }")

echo "Response:"
echo "$RESPONSE" | jq .

SUCCESS=$(echo "$RESPONSE" | jq -r '.success')
if [ "$SUCCESS" = "true" ]; then
    echo ""
    echo "✅ Registration successful!"
    echo "You can now claim bounties for issues created by @$GITHUB_USER"
else
    echo ""
    echo "❌ Registration failed"
fi
