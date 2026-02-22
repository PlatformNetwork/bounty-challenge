# Anti-Abuse Mechanisms

Bounty Challenge implements multiple layers of protection against gaming and abuse.

## Threat Model

### Potential Attack Vectors

| Attack | Description | Mitigation |
|--------|-------------|------------|
| **Spam** | Mass low-quality issues | Maintainer gatekeeping + penalty system |
| **Duplication** | Same bug reported twice | First reporter wins (single-claim rule) |
| **Collusion** | Fake maintainer approval | Project-level access control |
| **Frontrunning** | Claiming others' issues | Author match verification |
| **Self-Approval** | Miners validating own issues | Only project members can add labels |
| **Validator Manipulation** | Single validator approving issues | Multi-validator consensus required |

## Protection Layers

### 1. Maintainer Gatekeeping

**Only project maintainers can add the `valid` label.**

```
Issue Created → Maintainer Review → Valid Label Added → Bounty Eligible
      ↓                ↓                    
   (anyone)      (project members only)     
```

This is the primary defense: no matter how many issues you create, they're worthless without maintainer approval.

### 2. Validator Consensus

The WASM module requires multiple validators to agree on issue data before it is accepted:

- **Sync Consensus**: Validators propose synced issue data via `/sync/propose`. A majority must agree before the data is stored.
- **Issue Validity Consensus**: Validators propose issue validity via `/issue/propose`. A majority vote determines the outcome.

```
Validator A proposes ─┐
Validator B proposes ─┼──▶ Majority? ──▶ Consensus reached ──▶ Data stored
Validator C proposes ─┘
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

The verification is performed in the WASM module's `validate_issue()` function:

```rust
if issue.author.to_lowercase() != expected_author.to_lowercase() {
    return (false, Some("Author mismatch"));
}
```

### 4. Single Claim Rule

Each issue can only be claimed once. The WASM module checks host storage before recording:

```rust
if storage::is_issue_recorded(&repo_owner, &repo_name, issue_number) {
    // Reject: "Issue already claimed"
}
```

First valid claim wins. Subsequent attempts are rejected.

### 5. Penalty System

Invalid and duplicate issues reduce a miner's balance:

| Penalty Type | Formula |
|-------------|---------|
| **Invalid** | `max(0, invalid_count - valid_count)` |
| **Duplicate** | `max(0, duplicate_count - valid_count)` |

If `net_points ≤ 0`, the miner's weight becomes **0** (penalized).

See [Scoring & Rewards](reference/scoring.md) for detailed penalty calculations.

### 6. Registration Requirement

Miners must register their GitHub username before claiming:

```
1. Register: hotkey → github_username (signed with sr25519)
2. Create issues (as github_username)
3. Claim bounties (verified against registration)
```

This creates an audit trail and prevents anonymous claiming. Each hotkey maps to exactly one GitHub username, and vice versa.

### 7. Signature Verification

All authenticated operations require sr25519 signatures, proving ownership of the hotkey. The Platform validator verifies signatures before forwarding requests to the WASM module.

## Verification Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    CLAIM VERIFICATION                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Input: issue_numbers=[42], github_username="alice"         │
│                                                              │
│  ┌─────────────────┐                                        │
│  │ Already claimed?│──Yes──▶ REJECT: "Already claimed"      │
│  └────────┬────────┘                                        │
│           │No                                                │
│  ┌────────▼────────┐                                        │
│  │ In synced data? │──No───▶ REJECT: "Not found"            │
│  └────────┬────────┘                                        │
│           │Yes                                               │
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
│  │ Record in       │                                        │
│  │ host storage    │                                        │
│  │ Update balance  │                                        │
│  └────────┬────────┘                                        │
│           │                                                  │
│           ▼                                                  │
│       ACCEPT ✅                                              │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Monitoring

### Suspicious Patterns

Operators should monitor for:

1. **Rapid registrations**: Same IP registering many hotkeys
2. **Claim spikes**: Sudden large batch claims
3. **Low approval rate**: Many claims, few valid
4. **Maintainer anomalies**: Unusual label additions

### Audit Trail

All bounties are tracked in host storage with:
- Issue number
- Repository owner and name
- GitHub username
- Miner hotkey
- Recording epoch

## Future Enhancements

Potential additional protections:

1. **Reputation system**: Weight recent vs historical contributions
2. **Issue quality scoring**: NLP analysis of issue content
3. **Cooldown periods**: Time between registrations/claims
4. **Stake requirements**: Minimum stake to participate
5. **Maintainer rotation**: Prevent single-point collusion
