# API Reference

Complete API documentation for Bounty Challenge.

## Base URLs

### Bridge API (Recommended)

All requests should go through the platform bridge:

```
https://chain.platform.network/api/v1/bridge/bounty-challenge/
```

### Direct Server (Development)

For local development or direct server access:

```
http://localhost:8080/
```

---

## Authentication

### Signature-Based Authentication

Requests that modify state require sr25519 signatures:

```
message = "{action}:{data}:{timestamp}"
signature = sr25519_sign(message, secret_key)
```

### Timestamp Validation

- Timestamps must be within **5 minutes** of server time
- Uses Unix timestamps (seconds since epoch)
- Prevents replay attacks

---

## Endpoints

### Health Check

Check if the server is healthy.

**GET** `/health`

**Response:**
```json
{
  "healthy": true,
  "load": 0.0,
  "pending": 0,
  "uptime_secs": 3600,
  "version": "0.1.0",
  "challenge_id": "bounty-challenge"
}
```

---

### Register

Register a GitHub username with a hotkey.

**POST** `/register`

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

**Success Response (200):**
```json
{
  "success": true,
  "message": "Successfully registered @johndoe with your hotkey."
}
```

**Error Response (400):**
```json
{
  "success": false,
  "error": "Invalid signature. Make sure you're using the correct key."
}
```

**Possible Errors:**
| Error | Cause |
|-------|-------|
| `Timestamp expired` | Request older than 5 minutes |
| `Invalid signature` | Signature doesn't match hotkey |
| `Registration failed` | Database error |

---

### Status

Get status for a specific hotkey.

**GET** `/status/{hotkey}`

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
  "weight": 0.05
}
```

**Not Registered Response:**
```json
{
  "registered": false,
  "github_username": null,
  "valid_issues_count": null,
  "weight": null
}
```

---

### Leaderboard

Get current standings.

**GET** `/leaderboard`

**Query Parameters:**
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `limit` | integer | 20 | Number of entries to return |

**Response:**
```json
{
  "leaderboard": [
    {
      "github_username": "alice",
      "hotkey": "5GrwvaEF...",
      "issues_resolved_24h": 12,
      "weight": 0.12
    },
    {
      "github_username": "bob",
      "hotkey": "5FHneW46...",
      "issues_resolved_24h": 8,
      "weight": 0.08
    }
  ]
}
```

---

### Stats

Get challenge statistics.

**GET** `/stats`

**Response:**
```json
{
  "total_bounties": 150,
  "total_miners": 25,
  "challenge_id": "bounty-challenge",
  "version": "0.1.0"
}
```

---

### Get Weights

Get current weight calculations for all miners.

**GET** `/get_weights`

**Query Parameters:**
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `epoch` | integer | current | Epoch number |

**Response:**
```json
{
  "weights": [
    {
      "miner_hotkey": "5GrwvaEF...",
      "weight": 0.35
    },
    {
      "miner_hotkey": "5FHneW46...",
      "weight": 0.25
    }
  ],
  "epoch": 12345,
  "challenge_id": "bounty-challenge",
  "total_miners": 15
}
```

---

### Config

Get challenge configuration.

**GET** `/config`

**Response:**
```json
{
  "challenge_id": "bounty-challenge",
  "name": "Bounty Challenge",
  "version": "0.1.0",
  "config_schema": {
    "type": "object",
    "properties": {
      "action": {
        "type": "string",
        "enum": ["register", "claim", "leaderboard"]
      },
      "github_username": {
        "type": "string"
      }
    }
  },
  "features": ["github-verification", "anti-abuse"],
  "limits": {
    "max_submission_size": 10240,
    "max_evaluation_time": 60
  }
}
```

---

## Error Handling

### Error Response Format

```json
{
  "success": false,
  "error": "Error message here"
}
```

### HTTP Status Codes

| Code | Meaning |
|------|---------|
| 200 | Success |
| 400 | Bad Request (validation error) |
| 401 | Unauthorized (invalid signature) |
| 404 | Not Found |
| 500 | Internal Server Error |

### Common Errors

| Error Message | Cause | Solution |
|--------------|-------|----------|
| `Timestamp expired` | Request too old | Use current timestamp |
| `Invalid signature` | Wrong key used | Verify secret key |
| `Registration failed` | DB error | Retry or contact support |
| `Hotkey not found` | Not registered | Register first |

---

## Rate Limits

| Endpoint | Limit |
|----------|-------|
| `/register` | 10/minute |
| `/status/*` | 60/minute |
| `/leaderboard` | 30/minute |
| `/stats` | 30/minute |

---

## Code Examples

### Python

```python
import requests
import time
from substrateinterface import Keypair

# Create keypair from seed
keypair = Keypair.create_from_mnemonic("your mnemonic here")

# Prepare registration
timestamp = int(time.time())
message = f"register_github:johndoe:{timestamp}"
signature = keypair.sign(message.encode()).hex()

# Register
response = requests.post(
    "https://chain.platform.network/api/v1/bridge/bounty-challenge/register",
    json={
        "hotkey": keypair.ss58_address,
        "github_username": "johndoe",
        "signature": f"0x{signature}",
        "timestamp": timestamp
    }
)

print(response.json())
```

### Rust

```rust
use sp_core::{sr25519, Pair};
use reqwest::Client;
use serde_json::json;

async fn register(secret_key: &str, github_username: &str) -> Result<(), Box<dyn std::error::Error>> {
    let pair = sr25519::Pair::from_string(secret_key, None)?;
    let timestamp = chrono::Utc::now().timestamp();
    
    let message = format!("register_github:{}:{}", github_username.to_lowercase(), timestamp);
    let signature = pair.sign(message.as_bytes());
    
    let client = Client::new();
    let response = client
        .post("https://chain.platform.network/api/v1/bridge/bounty-challenge/register")
        .json(&json!({
            "hotkey": encode_ss58(&pair.public().0),
            "github_username": github_username,
            "signature": hex::encode(signature.0),
            "timestamp": timestamp
        }))
        .send()
        .await?;
    
    println!("{}", response.text().await?);
    Ok(())
}
```

### JavaScript

```javascript
const { Keyring } = require('@polkadot/keyring');
const { u8aToHex } = require('@polkadot/util');

async function register(mnemonic, githubUsername) {
    const keyring = new Keyring({ type: 'sr25519' });
    const pair = keyring.addFromMnemonic(mnemonic);
    
    const timestamp = Math.floor(Date.now() / 1000);
    const message = `register_github:${githubUsername.toLowerCase()}:${timestamp}`;
    const signature = pair.sign(message);
    
    const response = await fetch(
        'https://chain.platform.network/api/v1/bridge/bounty-challenge/register',
        {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                hotkey: pair.address,
                github_username: githubUsername,
                signature: u8aToHex(signature),
                timestamp: timestamp
            })
        }
    );
    
    console.log(await response.json());
}
```

---

## WebSocket API

*Coming soon: Real-time updates via WebSocket*

```
wss://chain.platform.network/api/v1/ws/bounty-challenge
```

Events:
- `issue_validated` - New issue validated
- `weight_updated` - Weights recalculated
- `leaderboard_changed` - Rankings changed
