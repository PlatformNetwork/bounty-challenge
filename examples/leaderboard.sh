#!/bin/bash
# View the Bounty Challenge leaderboard
#
# Usage: ./leaderboard.sh

set -e

CHALLENGE_URL="${CHALLENGE_URL:-http://localhost:8080}"

echo "=== Bounty Challenge Leaderboard ==="
echo ""

RESPONSE=$(curl -s -X POST "$CHALLENGE_URL/evaluate" \
    -H "Content-Type: application/json" \
    -d '{
        "request_id": "lb-'$(date +%s)'",
        "submission_id": "sub-'$(date +%s)'",
        "participant_id": "viewer",
        "epoch": 1,
        "data": {"action": "leaderboard"}
    }')

# Check if successful
SUCCESS=$(echo "$RESPONSE" | jq -r '.success')
if [ "$SUCCESS" != "true" ]; then
    echo "Error fetching leaderboard:"
    echo "$RESPONSE" | jq -r '.error'
    exit 1
fi

# Extract and format leaderboard
ENTRIES=$(echo "$RESPONSE" | jq -r '.results.leaderboard | length')

if [ "$ENTRIES" = "0" ]; then
    echo "No miners with valid bounties yet."
    exit 0
fi

echo "Rank | GitHub User      | Valid Issues | Score  | Hotkey"
echo "-----|------------------|--------------|--------|------------------"

echo "$RESPONSE" | jq -r '.results.leaderboard | to_entries | .[] | 
    "\(.key + 1)    | \(.value.github_username | .[0:16] | . + " " * (16 - length)) | \(.value.valid_issues | tostring | . + " " * (12 - length)) | \(.value.score | tostring | .[0:6]) | \(.value.miner_hotkey | .[0:16])..."'

echo ""
echo "Total miners: $ENTRIES"
