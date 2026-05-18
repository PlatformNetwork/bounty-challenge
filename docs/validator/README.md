# Validator And Owner Guide

## Purpose

Bounty Challenge lets a validator or owner operate a manual project-reward subnet. The operator
publishes project briefs, receives miner GitHub submissions, records human reviews, and assigns raw
emission weights that Platform normalizes for the network.

## Responsibilities

Validators and owners are responsible for:

- publishing clear project briefs and rubrics;
- keeping project status accurate;
- reviewing submitted work fairly;
- assigning emissions only to miner hotkeys that earned rewards;
- protecting owner and Platform tokens;
- monitoring health, version, submissions, reviews, and leaderboard output.

## Authentication

Public miner requests do not require owner credentials. Owner requests require either:

```http
Authorization: Bearer <owner-token>
```

or:

```http
X-Owner-Token: <owner-token>
```

Platform internal weight requests require the shared challenge token and challenge slug header.

## Runtime Configuration

All runtime settings use the `CHALLENGE_` environment prefix.

| Setting | Purpose |
| --- | --- |
| `CHALLENGE_SLUG` | Challenge identifier; defaults to `bounty-challenge`. |
| `CHALLENGE_NAME` | Human-readable challenge name. |
| `CHALLENGE_DATABASE_URL` | Persistent storage location. |
| `CHALLENGE_ARTIFACT_ROOT` | Reserved location for future submission assets. |
| `CHALLENGE_SHARED_TOKEN` | Shared token for Platform internal calls. |
| `CHALLENGE_SHARED_TOKEN_FILE` | File containing the Platform shared token. |
| `CHALLENGE_OWNER_TOKEN` | Owner token for admin operations. |
| `CHALLENGE_OWNER_TOKEN_FILE` | File containing the owner token. |
| `CHALLENGE_HOST` | Bind host. |
| `CHALLENGE_PORT` | Bind port. |

Use file-backed secrets in production so tokens are not exposed in process lists or shell history.

## Project Management

Create or update a project:

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
  "deadline_at": "2026-06-01T00:00:00Z",
  "metadata": {
    "rubric": [
      "visual design",
      "UX flow",
      "accessibility",
      "implementation quality"
    ]
  }
}
```

Supported project statuses:

| Status | Meaning |
| --- | --- |
| `active` | Miners can submit work. |
| `paused` | Project is visible but closed to new submissions. |
| `completed` | Project is finished. |
| `cancelled` | Project is cancelled. |

## Submission Review

Review a miner submission:

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

Supported review statuses:

| Status | Meaning |
| --- | --- |
| `reviewed` | Feedback was recorded, but no final decision is implied. |
| `accepted` | Submission is accepted by the owner. |
| `rejected` | Submission is rejected by the owner. |

Review scores must be between `0` and `100`. Scores are audit information; emissions decide final
network rewards.

## Emission Management

Set emissions manually:

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

Emission behavior:

- `replace: true` clears previous emissions before inserting the new set.
- `replace: false` updates only the provided hotkeys.
- Only positive finite weights are exposed to Platform.
- Platform normalizes raw weights into final network weights.

Reward one submission directly:

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

List current owner-set emissions:

```http
GET /admin/emissions
```

## Public Miner Surface

Validators should expect miners and dashboards to use:

```http
GET /projects
GET /projects/{project_id}
POST /submissions
GET /submissions
GET /submissions/{submission_id}
GET /leaderboard
```

## Platform Contract

Health check:

```http
GET /health
```

Version and capability check:

```http
GET /version
```

Weight request:

```http
GET /internal/v1/get_weights
Authorization: Bearer <shared-token>
X-Platform-Challenge-Slug: bounty-challenge
```

Example weight response:

```json
{
  "challenge_slug": "bounty-challenge",
  "epoch": 1760000000,
  "weights": {
    "5...": 2.5
  }
}
```

## Review Process Checklist

1. Publish a clear project brief and rubric.
2. Confirm project status is `active`.
3. Monitor incoming submissions.
4. Clone and review each submitted repository.
5. Record a review with score and comments.
6. Assign emissions to winning hotkeys.
7. Confirm the leaderboard matches the intended reward allocation.
8. Confirm Platform can read the protected weight contract.
9. Mark the project `completed` when rewards are final.

## Security Checklist

- Keep owner tokens private.
- Keep shared Platform tokens private.
- Prefer secret files over direct environment values.
- Do not reward submissions that cannot be inspected.
- Do not accept private repositories unless the owner has explicit access.
- Verify hotkeys before assigning emissions.
- Keep review comments professional and auditable.
- Back up persistent storage before major reward changes.
