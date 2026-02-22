# API Reference

Complete API documentation for Bounty Challenge.

## Base URL

All requests go through the Platform Network validator chain RPC:

**HTTP REST:**
```
http://VALIDATOR_IP:8080/challenge/bounty-challenge/
```

**JSON-RPC:**
```
POST http://VALIDATOR_IP:8080/rpc
```

> **Note**: The WASM module uses bincode serialization internally. The Platform validator handles JSON ↔ bincode translation for external clients.

---

## Authentication

### Signature-Based Authentication

Routes marked as requiring auth need a valid sr25519 signature. The validator verifies signatures before forwarding requests to the WASM module.

```
message = "{action}:{data}:{timestamp}"
signature = sr25519_sign(message, secret_key)
```

### Timestamp Validation

- Timestamps must be within **5 minutes** of server time
- Uses Unix timestamps (seconds since epoch)
- Prevents replay attacks

---

## JSON-RPC Format

All routes can be accessed via the `challenge_call` JSON-RPC method:

```json
{
  "jsonrpc": "2.0",
  "method": "challenge_call",
  "params": {
    "challengeId": "bounty-challenge",
    "method": "GET",
    "path": "/leaderboard",
    "body": null,
    "query": {}
  },
  "id": 1
}
```

**Response format:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "challengeId": "bounty-challenge",
    "status": 200,
    "headers": {},
    "body": { ... }
  },
  "id": 1
}
```

---

## Endpoints

### Register

Register a GitHub username with a hotkey.

**POST** `/register` (requires auth)

**Request Body:**
```json
{
  "hotkey": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
  "github_username": "johndoe",
  "signature": "0x...",
  "timestamp": 1705590000
}
```

**Signature Message Format:**
```
register_github:{github_username_lowercase}:{timestamp}
```

**Response:** `true` on success, `false` on failure.

**Possible Errors:**
| Error | Cause |
|-------|-------|
| 401 | Missing or invalid authentication |
| 400 | Invalid request body |

---

### Status

Get status for a specific hotkey.

**GET** `/status/:hotkey`

**Path Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `hotkey` | string | SS58-encoded hotkey |

**Response:**
```json
{
  "registered": true,
  "github_username": "johndoe",
  "valid_issues_count": 5,
  "invalid_issues_count": 2,
  "balance": {
    "valid_count": 5,
    "invalid_count": 2,
    "duplicate_count": 0,
    "star_count": 3,
    "is_penalized": false
  },
  "weight": 0.13
}
```

**Not Registered Response:**
```json
{
  "registered": false,
  "github_username": null,
  "valid_issues_count": 0,
  "invalid_issues_count": 0,
  "balance": {
    "valid_count": 0,
    "invalid_count": 0,
    "duplicate_count": 0,
    "star_count": 0,
    "is_penalized": false
  },
  "weight": 0.0
}
```

---

### Leaderboard

Get current standings.

**GET** `/leaderboard`

**Response:**
```json
[
  {
    "rank": 1,
    "hotkey": "5GrwvaEF...",
    "github_username": "alice",
    "score": 0.24,
    "valid_issues": 12,
    "invalid_issues": 0,
    "pending_issues": 0,
    "star_count": 3,
    "star_bonus": 0.75,
    "net_points": 12.75,
    "is_penalized": false,
    "last_epoch": 100
  }
]
```

---

### Stats

Get challenge statistics.

**GET** `/stats`

**Response:**
```json
{
  "total_bounties": 150,
  "active_miners": 25,
  "validator_count": 5,
  "total_issues": 200
}
```

---

### Claim

Claim bounty for resolved issues.

**POST** `/claim` (requires auth)

**Request Body:**
```json
{
  "hotkey": "5GrwvaEF...",
  "github_username": "johndoe",
  "issue_numbers": [42, 43, 44],
  "repo_owner": "PlatformNetwork",
  "repo_name": "bounty-challenge",
  "signature": "0x...",
  "timestamp": 1705590000
}
```

**Response:**
```json
{
  "claimed": [
    { "issue_number": 42 },
    { "issue_number": 43 }
  ],
  "rejected": [
    { "issue_number": 44, "reason": "Issue already claimed" }
  ],
  "total_valid": 7,
  "score": 0.14
}
```

---

### List Issues

Get all synced issues.

**GET** `/issues`

**Response:** Array of `IssueRecord` objects:
```json
[
  {
    "issue_number": 42,
    "repo_owner": "PlatformNetwork",
    "repo_name": "bounty-challenge",
    "author": "johndoe",
    "is_closed": true,
    "has_valid_label": true,
    "has_invalid_label": false,
    "claimed_by_hotkey": "5GrwvaEF...",
    "recorded_epoch": 100
  }
]
```

---

### List Pending Issues

Get pending (unclaimed, open) issues.

**GET** `/issues/pending`

**Response:** Array of `IssueRecord` objects (filtered to unclosed, unclaimed issues).

---

### Hotkey Details

Get detailed information for a specific hotkey.

**GET** `/hotkey/:hotkey`

**Path Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `hotkey` | string | SS58-encoded hotkey |

**Response:** Same format as `/status/:hotkey` (returns `StatusResponse`).

**Not Found:** Returns 404 if hotkey is not registered.

---

### Record Invalid Issue

Record an invalid issue.

**POST** `/invalid` (requires auth)

**Request Body:**
```json
{
  "issue_number": 123,
  "repo_owner": "PlatformNetwork",
  "repo_name": "bounty-challenge",
  "github_username": "johndoe",
  "reason": "Not a real bug"
}
```

**Response:** `true` on success, `false` on failure.

---

### Propose Sync Data

Propose synced issue data for validator consensus.

**POST** `/sync/propose` (requires auth)

**Request Body:**
```json
{
  "validator_id": "validator-1",
  "issues": [
    {
      "issue_number": 42,
      "repo_owner": "PlatformNetwork",
      "repo_name": "bounty-challenge",
      "author": "johndoe",
      "is_closed": true,
      "has_valid_label": true,
      "has_invalid_label": false,
      "claimed_by_hotkey": null,
      "recorded_epoch": 100
    }
  ]
}
```

**Response:** `true` if proposal was recorded. If consensus is reached, the synced issues are automatically stored.

---

### Check Sync Consensus

Check the current sync consensus status.

**GET** `/sync/consensus`

**Response:** The consensus result (array of `IssueRecord` if consensus reached, `null` otherwise).

---

### Propose Issue Validity

Propose whether a specific issue is valid or invalid.

**POST** `/issue/propose` (requires auth)

**Request Body:**
```json
{
  "validator_id": "validator-1",
  "issue_number": 42,
  "repo_owner": "PlatformNetwork",
  "repo_name": "bounty-challenge",
  "is_valid": true
}
```

**Response:** `true` if proposal was recorded.

---

### Check Issue Consensus

Check consensus on a specific issue's validity.

**POST** `/issue/consensus`

**Request Body:**
```json
{
  "issue_number": 42,
  "repo_owner": "PlatformNetwork",
  "repo_name": "bounty-challenge"
}
```

**Response:** `true` if consensus says valid, `false` if invalid, `null` if no consensus yet.

---

### Get Timeout Config

Get current timeout configuration.

**GET** `/config/timeout`

**Response:**
```json
{
  "review_timeout_blocks": 1800,
  "sync_timeout_blocks": 300
}
```

---

### Set Timeout Config

Update timeout configuration.

**POST** `/config/timeout` (requires auth)

**Request Body:**
```json
{
  "review_timeout_blocks": 1800,
  "sync_timeout_blocks": 300
}
```

**Response:** `true` on success.

---

### Get Weights

Get normalized weight assignments for all miners.

**GET** `/get_weights`

**Response:**
```json
[
  {
    "hotkey": "5GrwvaEF...",
    "weight": 0.35
  },
  {
    "hotkey": "5FHneW46...",
    "weight": 0.25
  }
]
```

Weights are normalized to sum to 1.0 across all non-penalized miners with positive scores.

---

## Error Handling

### HTTP Status Codes

| Code | Meaning |
|------|---------|
| 200 | Success |
| 400 | Bad Request (invalid body or parameters) |
| 401 | Unauthorized (missing authentication) |
| 404 | Not Found (unknown route or resource) |

---

## Code Examples

### Python

```python
import requests
import time
from substrateinterface import Keypair

VALIDATOR_RPC = "http://VALIDATOR_IP:8080"

# Get leaderboard (HTTP REST)
response = requests.get(f"{VALIDATOR_RPC}/challenge/bounty-challenge/leaderboard")
print(response.json())

# Get leaderboard (JSON-RPC)
response = requests.post(f"{VALIDATOR_RPC}/rpc", json={
    "jsonrpc": "2.0",
    "method": "challenge_call",
    "params": {
        "challengeId": "bounty-challenge",
        "method": "GET",
        "path": "/leaderboard"
    },
    "id": 1
})
print(response.json())

# Register (JSON-RPC)
keypair = Keypair.create_from_mnemonic("your mnemonic here")
timestamp = int(time.time())
message = f"register_github:johndoe:{timestamp}"
signature = keypair.sign(message.encode()).hex()

response = requests.post(f"{VALIDATOR_RPC}/rpc", json={
    "jsonrpc": "2.0",
    "method": "challenge_call",
    "params": {
        "challengeId": "bounty-challenge",
        "method": "POST",
        "path": "/register",
        "body": {
            "hotkey": keypair.ss58_address,
            "github_username": "johndoe",
            "signature": f"0x{signature}",
            "timestamp": timestamp
        }
    },
    "id": 1
})
print(response.json())
```

### JavaScript

```javascript
const { Keyring } = require('@polkadot/keyring');
const { u8aToHex } = require('@polkadot/util');

const VALIDATOR_RPC = 'http://VALIDATOR_IP:8080';

// Get leaderboard (HTTP REST)
const leaderboard = await fetch(
    `${VALIDATOR_RPC}/challenge/bounty-challenge/leaderboard`
);
console.log(await leaderboard.json());

// Register (JSON-RPC)
async function register(mnemonic, githubUsername) {
    const keyring = new Keyring({ type: 'sr25519' });
    const pair = keyring.addFromMnemonic(mnemonic);
    
    const timestamp = Math.floor(Date.now() / 1000);
    const message = `register_github:${githubUsername.toLowerCase()}:${timestamp}`;
    const signature = pair.sign(message);
    
    const response = await fetch(`${VALIDATOR_RPC}/rpc`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            jsonrpc: '2.0',
            method: 'challenge_call',
            params: {
                challengeId: 'bounty-challenge',
                method: 'POST',
                path: '/register',
                body: {
                    hotkey: pair.address,
                    github_username: githubUsername,
                    signature: u8aToHex(signature),
                    timestamp: timestamp
                }
            },
            id: 1
        })
    });
    
    console.log(await response.json());
}
```

### Using the CLI

```bash
# Build
cargo build --release -p bounty-cli

# Leaderboard
./target/release/bounty-cli leaderboard --rpc-url http://VALIDATOR_IP:8080

# Register
./target/release/bounty-cli register \
  --hotkey 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY \
  --github johndoe \
  --signature 0xabc123...def456 \
  --timestamp 1705590000 \
  --rpc-url http://VALIDATOR_IP:8080

# Status
./target/release/bounty-cli status \
  --hotkey 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY \
  --rpc-url http://VALIDATOR_IP:8080

# Stats
./target/release/bounty-cli stats --rpc-url http://VALIDATOR_IP:8080
```
