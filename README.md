<div align="center">

# bουηtү chαllεηgε

**Incentivizing miners to complete project tasks and deliver the best final project outcome**

![Bounty Challenge Banner](assets/banner.jpg)

</div>

## Overview

Bounty Challenge is a Python/FastAPI challenge service for Platform Network that incentivizes
miners to complete project tasks and compete for the best final rendering of a requested project.
Admins create project bounties, miners submit GitHub links for their work, and the owner manually
reviews submissions and sets final reward emissions.

This version does not auto-score GitHub issues. It is designed for subjective work such as:

- best UI/UX for a CLI;
- best interface redesign for a project;
- implementation of a requested product flow;
- design or frontend polish tasks that need human judgment.

## How It Works

1. The owner creates a project with goals, resources, and optional deadline.
2. Miners submit a GitHub repository/link for the project.
3. The owner reviews submissions manually.
4. The owner sets emissions directly for winning hotkeys.
5. Platform calls `/internal/v1/get_weights` and distributes the owner-set weights.

## Public Miner API

### List projects

```http
GET /projects
```

### Get one project

```http
GET /projects/{project_id}
```

### Submit work

```http
POST /submissions
Content-Type: application/json
```

```json
{
  "project_id": "cli-ui",
  "miner_hotkey": "5...",
  "name": "polished-cli",
  "github_url": "https://github.com/miner/polished-cli",
  "branch": "main",
  "commit_sha": "optional",
  "description": "Improved navigation, theme, and onboarding flow."
}
```

### List submissions

```http
GET /submissions
GET /submissions?project_id=cli-ui
```

### Leaderboard

```http
GET /leaderboard
```

The leaderboard reflects current owner-set emissions.

## Owner/Admin API

Owner routes require:

```http
Authorization: Bearer <CHALLENGE_OWNER_TOKEN>
```

or:

```http
X-Owner-Token: <CHALLENGE_OWNER_TOKEN>
```

### Create or update a project

```http
POST /admin/projects
```

```json
{
  "project_id": "cli-ui",
  "title": "Best CLI UI/UX",
  "description": "Create the best UI/UX for the Platform CLI.",
  "goals": "Improve clarity, onboarding, layout, speed of use, and visual polish.",
  "repository_url": "https://github.com/PlatformNetwork/platform",
  "assets_url": "https://example.com/brief",
  "status": "active",
  "metadata": {
    "rubric": ["visual design", "UX flow", "accessibility", "implementation quality"]
  }
}
```

Project statuses: `active`, `paused`, `completed`, `cancelled`.

### Review a submission

```http
POST /admin/reviews
```

```json
{
  "submission_id": 1,
  "score": 94,
  "status": "accepted",
  "comments": "Best UX and cleanest implementation.",
  "reviewer": "owner"
}
```

Review statuses: `reviewed`, `accepted`, `rejected`.

### Set emissions manually

```http
POST /admin/emissions
```

```json
{
  "replace": true,
  "updated_by": "sudo-owner",
  "emissions": [
    {
      "hotkey": "5...",
      "weight": 2.5,
      "reason": "winner of cli-ui",
      "project_id": "cli-ui",
      "submission_id": 1
    }
  ]
}
```

- `replace: true` clears previous emissions before inserting the new set.
- `replace: false` updates only the provided hotkeys.
- Weights are raw weights; Platform normalizes final subnet weights.

### Reward one submission directly

```http
POST /admin/submissions/{submission_id}/reward
```

```json
{
  "weight": 2.5,
  "reason": "winner",
  "updated_by": "sudo-owner"
}
```

## Platform Contract

### Health

```http
GET /health
```

### Version

```http
GET /version
```

Capabilities include:

- `get_weights`
- `proxy_routes`
- `sqlite`
- `manual_bounties`
- `manual_emissions`

### Weights

```http
GET /internal/v1/get_weights
Authorization: Bearer <CHALLENGE_SHARED_TOKEN>
X-Platform-Challenge-Slug: bounty-challenge
```

Response:

```json
{
  "challenge_slug": "bounty-challenge",
  "weights": {
    "5...": 2.5
  }
}
```

## Configuration

Environment variables use the `CHALLENGE_` prefix.

| Variable | Default |
| --- | --- |
| `CHALLENGE_SLUG` | `bounty-challenge` |
| `CHALLENGE_NAME` | `Bounty Challenge` |
| `CHALLENGE_DATABASE_URL` | `sqlite+aiosqlite:////data/bounty-challenge.sqlite3` |
| `CHALLENGE_SHARED_TOKEN_FILE` | `/run/secrets/platform/challenge_token` |
| `CHALLENGE_OWNER_TOKEN_FILE` | `/run/secrets/platform/owner_token` |
| `CHALLENGE_HOST` | `0.0.0.0` |
| `CHALLENGE_PORT` | `8000` |

For local development:

```bash
export CHALLENGE_SHARED_TOKEN=test-token
export CHALLENGE_OWNER_TOKEN=owner-token
```

## Development

```bash
python -m pip install -e ".[dev]"
ruff check .
ruff format --check .
pytest -q
```

Run locally:

```bash
uvicorn bounty_challenge.app:app --reload
```
