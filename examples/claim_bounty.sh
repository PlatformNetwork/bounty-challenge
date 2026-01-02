#!/bin/bash
# Example script to claim bounties from Bounty Challenge
#
# Usage: ./claim_bounty.sh <hotkey> <github_username> <issue_numbers...>
# Example: ./claim_bounty.sh 5GrwvaEF... octocat 42 55 78

set -e

CHALLENGE_URL="${CHALLENGE_URL:-http://localhost:8080}"
HOTKEY="$1"
GITHUB_USER="$2"
shift 2
ISSUES="$@"

if [ -z "$HOTKEY" ] || [ -z "$GITHUB_USER" ] || [ -z "$ISSUES" ]; then
    echo "Usage: $0 <hotkey> <github_username> <issue_numbers...>"
    echo "Example: $0 5GrwvaEF... octocat 42 55 78"
    exit 1
fi

# Convert issues to JSON array
ISSUES_JSON=$(echo "$ISSUES" | tr ' ' '\n' | jq -s '.')

echo "=== Bounty Challenge Claim ==="
echo "Hotkey: $HOTKEY"
echo "GitHub: $GITHUB_USER"
echo "Issues: $ISSUES_JSON"
echo ""

# Make the claim request
RESPONSE=$(curl -s -X POST "$CHALLENGE_URL/evaluate" \
    -H "Content-Type: application/json" \
    -d "{
        \"request_id\": \"claim-$(date +%s)\",
        \"submission_id\": \"sub-$(date +%s)\",
        \"participant_id\": \"$HOTKEY\",
        \"epoch\": 1,
        \"data\": {
            \"action\": \"claim\",
            \"github_username\": \"$GITHUB_USER\",
            \"issue_numbers\": $ISSUES_JSON
        }
    }")

echo "Response:"
echo "$RESPONSE" | jq .

# Extract results
SUCCESS=$(echo "$RESPONSE" | jq -r '.success')
SCORE=$(echo "$RESPONSE" | jq -r '.score')
CLAIMED=$(echo "$RESPONSE" | jq -r '.results.claimed | length')
REJECTED=$(echo "$RESPONSE" | jq -r '.results.rejected | length')

echo ""
echo "=== Summary ==="
echo "Success: $SUCCESS"
echo "Score: $SCORE"
echo "Claimed: $CLAIMED issues"
echo "Rejected: $REJECTED issues"

if [ "$REJECTED" -gt 0 ]; then
    echo ""
    echo "Rejected issues:"
    echo "$RESPONSE" | jq -r '.results.rejected[] | "  #\(.issue_number): \(.reason)"'
fi
