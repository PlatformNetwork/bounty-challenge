# Miner Guide

## Purpose

Bounty Challenge rewards miners for completing owner-created project briefs. Your goal is to submit
the strongest finished project outcome, not just a proof of participation.

## Miner Flow

1. Read the list of active projects.
2. Pick a project whose goals and rubric match your skills.
3. Build the requested outcome in a public GitHub repository.
4. Submit your repository link, branch, commit, and description.
5. Watch the submission status and leaderboard.
6. Improve or resubmit when the owner allows another attempt.

## Reading Projects

Use the public project list to discover available work:

```http
GET /projects
```

Use the project detail request to inspect one brief:

```http
GET /projects/{project_id}
```

Important project fields:

| Field | Meaning |
| --- | --- |
| `project_id` | Stable identifier used when submitting work. |
| `title` | Short project name. |
| `description` | What the owner wants built. |
| `goals` | Success criteria and expected outcome. |
| `repository_url` | Source project or reference repository, when provided. |
| `assets_url` | Design brief, screenshots, datasets, or extra assets, when provided. |
| `status` | Only `active` projects accept submissions. |
| `deadline_at` | Optional deadline after which submissions are rejected. |
| `metadata` | Optional rubric, links, constraints, or owner notes. |

## Building A Strong Submission

A high-quality submission should:

- satisfy the project goals completely;
- be easy for the owner to clone, inspect, and run;
- include a clear description of what changed;
- reference the exact branch and commit you want reviewed;
- explain any trade-offs or incomplete items honestly;
- respect the deadline and any repository constraints;
- avoid secrets, private credentials, generated junk, and unrelated changes.

## Submitting Work

Submit a GitHub HTTPS link for the project:

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
  "commit_sha": "abc123",
  "description": "Improved navigation, onboarding, theme consistency, and accessibility.",
  "metadata": {
    "demo_url": "https://example.com/demo",
    "notes": "Includes setup instructions in the repository."
  }
}
```

Submission rules:

- `github_url` must be a GitHub HTTPS URL.
- `miner_hotkey` must be the hotkey that should receive rewards.
- `branch` and `commit_sha` are optional but recommended for reproducibility.
- Inactive, paused, completed, cancelled, or expired projects reject new submissions.
- Duplicate submission hashes for the same project are rejected.

## Tracking Your Submission

List recent submissions:

```http
GET /submissions
```

Filter by project:

```http
GET /submissions?project_id=cli-ui
```

Read one submission:

```http
GET /submissions/{submission_id}
```

Submission statuses:

| Status | Meaning |
| --- | --- |
| `submitted` | Work was received and is waiting for review. |
| `reviewed` | Owner recorded feedback and a score. |
| `accepted` | Owner accepted the submission. |
| `rejected` | Owner rejected the submission. |
| `rewarded` | Owner assigned emissions to the submission's hotkey. |

## Leaderboard

The leaderboard reflects owner-set emissions:

```http
GET /leaderboard
```

If your submission has a strong review but no emission yet, the owner may still be deciding final
allocation. Emissions are the source of network rewards.

## Review Rubric

Each project can define its own rubric, but owners typically compare:

- completion against the requested goals;
- user experience and visual quality;
- correctness and stability;
- code clarity and maintainability;
- ease of setup and review;
- accessibility and responsiveness where relevant;
- originality and usefulness of the final result.

## Miner Checklist

Before submitting:

- Confirm the project is active.
- Read all owner-provided links and assets.
- Push the exact branch and commit you want reviewed.
- Verify the repository is public or accessible to the owner.
- Add clear setup and review instructions in your repository.
- Remove secrets, local files, and unrelated changes.
- Submit before the deadline.
