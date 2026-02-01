# Anti-Abuse Mechanisms

Bounty Challenge implements multiple layers of protection against gaming and abuse.

## Threat Model

### Potential Attack Vectors

| Attack | Description | Mitigation |
|--------|-------------|------------|
| **Spam** | Mass low-quality issues | Maintainer gatekeeping + logarithmic scoring |
| **Duplication** | Same bug reported twice | First reporter wins (single-claim rule) |
| **Collusion** | Fake maintainer approval | Project-level access control |
| **Frontrunning** | Claiming others' issues | Author match verification |
| **Self-Approval** | Miners validating own issues | Only project members can add labels |

## Protection Layers

### 1. Maintainer Gatekeeping

**Only project maintainers can add the `valid` label.**

```
Issue Created → Maintainer Review → Valid Label Added → Bounty Eligible
      ↓                ↓                    
   (anyone)      (project members only)     
```

This is the primary defense: no matter how many issues you create, they're worthless without maintainer approval.

### 2. GitHub API Verification

Every claim is verified in real-time via GitHub API:

```rust
pub async fn verify_issue_validity(&self, issue_number: u32, author: &str) -> Result<BountyVerification> {
    let issue = self.get_issue(issue_number).await?;
    
    let is_author_match = issue.user.login.to_lowercase() == author.to_lowercase();
    let is_valid = issue.is_closed() && issue.has_valid_label();
    
    Ok(BountyVerification {
        is_valid_bounty: is_valid && is_author_match,
        // ...
    })
}
```

### 3. Author Verification

The GitHub username claiming the bounty **must match** the issue author:

```
Miner registers: github_username = "alice"
Issue #42 author: "alice" ✅ Can claim
Issue #43 author: "bob"   ❌ Cannot claim
```

This prevents:
- Claiming others' work
- Frontrunning legitimate reporters

### 4. Single Claim Rule

Each issue can only be claimed once:

```sql
INSERT OR IGNORE INTO validated_bounties (issue_number, ...)
```

First valid claim wins. Subsequent attempts are rejected with:
```json
{"reason": "Issue already claimed"}
```

### 5. Linear Points System

Each valid issue earns exactly 1 point with a clear cap:

| Issues | Points | Weight |
|--------|--------|--------|
| 1 | 1 | 2% |
| 10 | 10 | 20% |
| 50 | 50 | 100% (capped) |

The 50-point cap and maintainer gatekeeping ensure mass submission is uneconomical.

### 6. Registration Requirement

Miners must register their GitHub username before claiming:

```
1. Register: hotkey → github_username
2. Create issues (as github_username)
3. Claim bounties (verified against registration)
```

This creates an audit trail and prevents anonymous claiming.

## Verification Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    CLAIM VERIFICATION                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Input: issue_number=42, github_username="alice"            │
│                                                              │
│  ┌─────────────────┐                                        │
│  │ Already claimed?│──Yes──▶ REJECT: "Already claimed"      │
│  └────────┬────────┘                                        │
│           │No                                                │
│  ┌────────▼────────┐                                        │
│  │ Fetch from      │                                        │
│  │ GitHub API      │                                        │
│  └────────┬────────┘                                        │
│           │                                                  │
│  ┌────────▼────────┐                                        │
│  │ Is closed?      │──No───▶ REJECT: "Issue not closed"     │
│  └────────┬────────┘                                        │
│           │Yes                                               │
│  ┌────────▼────────┐                                        │
│  │ Has valid label?│──No───▶ REJECT: "Missing valid label"  │
│  └────────┬────────┘                                        │
│           │Yes                                               │
│  ┌────────▼────────┐                                        │
│  │ Author matches? │──No───▶ REJECT: "Author mismatch"      │
│  └────────┬────────┘                                        │
│           │Yes                                               │
│  ┌────────▼────────┐                                        │
│  │ Record bounty   │                                        │
│  │ Update score    │                                        │
│  └────────┬────────┘                                        │
│           │                                                  │
│           ▼                                                  │
│       ACCEPT ✅                                              │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Rate Limiting

### GitHub API Limits

| Authentication | Rate Limit |
|----------------|------------|
| No token | 60 requests/hour |
| With token | 5,000 requests/hour |

The challenge respects these limits with:
- 100ms delay between paginated requests
- Error handling for rate limit responses

### Claim Rate

No explicit claim rate limit (GitHub API is the bottleneck), but:
- Each claim requires an API call
- Failed claims don't count
- Repeated claims for same issue are ignored

## Monitoring

### Suspicious Patterns

Operators should monitor for:

1. **Rapid registrations**: Same IP registering many hotkeys
2. **Claim spikes**: Sudden large batch claims
3. **Low approval rate**: Many claims, few valid
4. **Maintainer anomalies**: Unusual label additions

### Audit Trail

All bounties are logged with:
- Issue number
- GitHub username
- Miner hotkey
- Timestamp
- Issue URL

```sql
SELECT * FROM validated_bounties 
WHERE validated_at > datetime('now', '-1 day')
ORDER BY validated_at DESC;
```

## Future Enhancements

Potential additional protections:

1. **Reputation system**: Weight recent vs historical contributions
2. **Issue quality scoring**: NLP analysis of issue content
3. **Cooldown periods**: Time between registrations/claims
4. **Stake requirements**: Minimum stake to participate
5. **Maintainer rotation**: Prevent single-point collusion
