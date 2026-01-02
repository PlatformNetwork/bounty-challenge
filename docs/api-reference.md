# API Reference

Complete API documentation for Bounty Challenge.

## Base URL

```
http://localhost:8080
```

## Endpoints

### Health Check

Check if the server is running.

```
GET /health
```

**Response:**
```json
{
  "healthy": true,
  "load": 0.25,
  "pending": 1,
  "uptime_secs": 3600,
  "version": "0.1.0",
  "challenge_id": "bounty-challenge"
}
```

### Configuration

Get challenge configuration and schema.

```
GET /config
```

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
      },
      "issue_numbers": {
        "type": "array",
        "items": {"type": "integer"}
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

### Evaluate

Main endpoint for all operations.

```
POST /evaluate
```

**Request Body:**
```json
{
  "request_id": "string",
  "submission_id": "string",
  "participant_id": "string (miner hotkey)",
  "epoch": 1,
  "data": {
    "action": "register|claim|leaderboard",
    ...
  }
}
```

---

## Actions

### Register

Link a GitHub username to a miner hotkey.

**Request:**
```json
{
  "request_id": "req-1",
  "submission_id": "sub-1",
  "participant_id": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
  "epoch": 1,
  "data": {
    "action": "register",
    "github_username": "octocat"
  }
}
```

**Response:**
```json
{
  "request_id": "req-1",
  "success": true,
  "error": null,
  "score": 1.0,
  "results": {
    "registered": true,
    "github_username": "octocat"
  },
  "execution_time_ms": 5,
  "cost": null
}
```

---

### Claim

Claim bounties for validated issues.

**Request:**
```json
{
  "request_id": "req-2",
  "submission_id": "sub-2",
  "participant_id": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
  "epoch": 1,
  "data": {
    "action": "claim",
    "github_username": "octocat",
    "issue_numbers": [42, 55, 78]
  }
}
```

**Response:**
```json
{
  "request_id": "req-2",
  "success": true,
  "error": null,
  "score": 0.158,
  "results": {
    "claimed": [
      {"issue_number": 42, "issue_url": "https://github.com/CortexLM/fabric/issues/42"},
      {"issue_number": 55, "issue_url": "https://github.com/CortexLM/fabric/issues/55"}
    ],
    "rejected": [
      {"issue_number": 78, "reason": "Issue missing 'valid' label"}
    ],
    "total_valid": 2,
    "score": 0.158
  },
  "execution_time_ms": 1250,
  "cost": null
}
```

**Rejection Reasons:**
- `"Issue already claimed"` - Another miner already claimed this issue
- `"Author mismatch: expected X, got Y"` - GitHub username doesn't match issue author
- `"Issue not closed"` - Issue is still open
- `"Issue missing 'valid' label"` - Issue lacks the required label
- `"Verification failed: ..."` - GitHub API error

---

### Leaderboard

Get current standings.

**Request:**
```json
{
  "request_id": "req-3",
  "submission_id": "sub-3",
  "participant_id": "anyone",
  "epoch": 1,
  "data": {
    "action": "leaderboard"
  }
}
```

**Response:**
```json
{
  "request_id": "req-3",
  "success": true,
  "error": null,
  "score": 0.0,
  "results": {
    "leaderboard": [
      {
        "miner_hotkey": "5GrwvaEF...",
        "github_username": "octocat",
        "valid_issues": 5,
        "score": 0.258,
        "last_updated": "2024-01-15T10:30:00Z"
      },
      {
        "miner_hotkey": "5FHneW46...",
        "github_username": "torvalds",
        "valid_issues": 3,
        "score": 0.2,
        "last_updated": "2024-01-14T15:20:00Z"
      }
    ]
  },
  "execution_time_ms": 12,
  "cost": null
}
```

---

### Validate

Quick validation without full evaluation.

```
POST /validate
```

**Request:**
```json
{
  "data": {
    "action": "claim",
    "github_username": "octocat",
    "issue_numbers": [42]
  }
}
```

**Response:**
```json
{
  "valid": true,
  "errors": [],
  "warnings": []
}
```

**Validation Errors:**
- `"Missing github_username"` - Required field not provided
- `"Missing issue_numbers for claim action"` - Claim requires issue numbers

---

## Error Responses

All endpoints return errors in this format:

```json
{
  "request_id": "req-1",
  "success": false,
  "error": "Error description",
  "score": 0.0,
  "results": null,
  "execution_time_ms": 5,
  "cost": null
}
```

**HTTP Status Codes:**
- `200` - Success
- `400` - Bad request (validation error)
- `500` - Internal server error

---

## Rate Limits

GitHub API has rate limits that affect this challenge:

| Auth Type | Limit |
|-----------|-------|
| No token | 60 requests/hour |
| With token | 5,000 requests/hour |

Set `GITHUB_TOKEN` environment variable for higher limits.
