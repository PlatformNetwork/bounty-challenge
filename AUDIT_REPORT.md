# Bounty-Challenge Data Flow Audit Report

## Files Audited
- `src/api/handlers.rs` — All route handlers
- `src/storage/bounty_storage.rs` — All storage operations
- `src/scoring.rs` — Weight & leaderboard calculation
- `src/types.rs` — Data structures
- `src/validation.rs` — Submission and issue validation
- `src/lib.rs` — Challenge trait implementation (evaluate, validate, get_weights)
- `src/routes.rs` — Route definitions and dispatch

---

## Storage Key Map

| Key Pattern | Value Type | Used By |
|---|---|---|
| `user:{hotkey}` | `UserRegistration` (bincode) | register_user, get_user_by_hotkey |
| `github:{lowercase_github}` | hotkey string (raw bytes) | register_user, get_hotkey_by_github |
| `issue:{owner}/{repo}:{issue_number_le_u32}` | `IssueRecord` (bincode) | record_valid_issue, is_issue_recorded, get_issue_record |
| `invalid_issue:{owner}/{repo}:{issue_number_le_u32}` | `InvalidIssueRecord` (bincode) | record_invalid_issue |
| `balance:{hotkey}` | `UserBalance` (bincode) | get/store_user_balance, increment_valid/invalid_count |
| `leaderboard` | `Vec<LeaderboardEntry>` (bincode) | get/store_leaderboard |
| `registered_hotkeys` | `Vec<String>` (bincode) | get/add_registered_hotkey |
| `synced_issues` | `Vec<IssueRecord>` (bincode) | store_issue_data, get_synced_issues |
| `active_miner_count` | u64 (LE bytes) | store/get_active_miner_count |
| `validator_count` | u64 (LE bytes) | store/get_validator_count |

**No key collisions** — all prefixes are distinct. Binary issue_number encoding in `issue:` and `invalid_issue:` keys is safely separated by the `:` delimiter from repo_name (GitHub names cannot contain `:`).

---

## Issue List

### CRITICAL Issues

#### C1. Legacy claim path allows impersonation (`handle_claim`, bincode path)
- **Location**: `src/api/handlers.rs`, lines ~199-215 (legacy bincode path in `handle_claim`)
- **Description**: The legacy bincode `BountySubmission` path extracts `auth_hotkey` from headers but never verifies that `submission.hotkey == auth_hotkey`. The `process_claims` function uses `submission.hotkey` (from the untrusted body) for recording claims and incrementing balances. An attacker authenticating as hotkey A can submit a `BountySubmission` with `hotkey: B` and claim issues on behalf of B, or grief B by recording invalid claims.
- **Impact**: Any authenticated user can claim issues attributed to any other hotkey.

#### C2. `handle_issues_sync` has no authorization — any authenticated user can overwrite all issue data
- **Location**: `src/api/handlers.rs`, `handle_issues_sync`
- **Description**: The sync endpoint only checks `is_authenticated(request)`, which merely verifies a non-empty `auth_hotkey`. There is no role check (validator, admin, etc.). Any miner/user with an auth token can call `/issues/sync` and completely replace the `synced_issues` dataset.
- **Impact**: Attacker can inject fabricated issues (with `has_valid_label: true`, `is_closed: true`, `has_ide_label: true`, `author: attacker_github`) into synced data, then immediately claim them via `/claim`. This enables unlimited self-awarded points. Alternatively, syncing an empty list DoS's all claim validation.

#### C3. No validation of synced issue data
- **Location**: `src/api/handlers.rs`, `handle_issues_sync`; `src/storage/bounty_storage.rs`, `store_issue_data`
- **Description**: Synced issue data is deserialized and stored with zero validation. No checks on: issue_number validity, repo_owner/repo_name format, label consistency, author format, or whether the data matches any real GitHub state.
- **Impact**: Combined with C2, this is a complete bypass of the issue validation system.

#### C4. Signature is never cryptographically verified
- **Location**: `src/validation.rs`, `validate_submission`; `src/lib.rs`, `evaluate`
- **Description**: `validate_submission` checks `submission.signature.is_empty()` to ensure a signature is present, but never verifies it against the hotkey or any public key. The `evaluate` function also requires non-empty signature but never validates it. The JSON claim path bypasses signature entirely (sets empty vec).
- **Impact**: The signature field provides zero security guarantee. Any non-empty byte array passes validation.

---

### MAJOR Issues

#### M1. `is_penalized` flag is never cleared when valid issues are added
- **Location**: `src/storage/bounty_storage.rs`, `increment_valid_count` (line ~220)
- **Description**: `increment_valid_count` only increments `valid_count` and stores the balance — it does NOT re-evaluate `is_penalized`. The penalty is only re-evaluated inside `increment_invalid_count`. Once a user is penalized, claiming more valid issues increases their `valid_count` but the `is_penalized` flag remains `true` permanently (unless they also receive another invalid issue, which triggers re-evaluation).
- **Impact**: Users who recover from a penalized state (by getting more valid issues than invalid) remain stuck as penalized with zero weight forever.

#### M2. `duplicate_count` field is never incremented — dead code in penalty formula
- **Location**: `src/types.rs` (line 69), `src/storage/bounty_storage.rs` (line 232)
- **Description**: `UserBalance.duplicate_count` is declared and used in the penalty formula: `penalty = (invalid_count - valid_count) + (duplicate_count - valid_count)`, but `duplicate_count` is never incremented anywhere in the codebase. It is always 0.
- **Impact**: The penalty formula's duplicate_count component is dead code. The actual penalty reduces to just `invalid_count > valid_count`.

#### M3. Penalty formula double-counts `valid_count` as buffer
- **Location**: `src/storage/bounty_storage.rs`, `increment_invalid_count` (lines ~228-233)
- **Description**: The penalty formula is: `penalty = (invalid_count - valid_count).saturating + (duplicate_count - valid_count).saturating`. This subtracts `valid_count` separately from both `invalid_count` and `duplicate_count`. This means `valid_count` acts as an independent buffer against each type of infraction rather than a shared buffer. Example: if valid=5, invalid=4, duplicate=4 → penalty = 0+0 = 0 (not penalized). But total infractions (8) exceed valid (5).
- **Impact**: Currently moot because `duplicate_count` is always 0, but if `duplicate_count` is ever used, the penalty logic will be more lenient than likely intended.

#### M4. Leaderboard sorted by `score` but weights use `net_points` — ranking inconsistency
- **Location**: `src/scoring.rs`, `rebuild_leaderboard` (line ~84) vs `calculate_weights_from_leaderboard` (line ~38)
- **Description**: `rebuild_leaderboard` sorts entries by `score` which is `(valid_count + star_count * 0.25) * 0.02` — ignoring invalid issues. But `calculate_weights_from_leaderboard` distributes weights based on `net_points` which subtracts penalties. A user ranked #1 on the leaderboard (by `score`) could receive less weight than a lower-ranked user (by `net_points`).
- **Impact**: The displayed leaderboard ranking does not correspond to actual weight distribution. Misleading to users.

#### M5. Empty `github_username` accepted in registration
- **Location**: `src/api/handlers.rs`, `handle_register`; `src/storage/bounty_storage.rs`, `register_user`
- **Description**: Neither `handle_register` nor `register_user` validates that `github_username` is non-empty. An empty github_username creates storage entries at `user:{hotkey}` with empty github and `github:` (prefix with empty suffix). The `github:` key could collide with future uses or interfere with lookups.
- **Impact**: Allows registration with empty github username. The user would never be able to claim issues (author matching would fail), but pollutes storage and the registered_hotkeys list.

#### M6. Legacy claim path doesn't verify `submission.github_username` matches registered github
- **Location**: `src/api/handlers.rs`, `handle_claim` (legacy bincode path)
- **Description**: The legacy path calls `validate_submission` (checks non-empty fields) then `process_claims`, but never checks that `submission.github_username` matches the github registered for `submission.hotkey`. This is checked in `evaluate()` but not in `handle_claim`. In `process_claims` → `validate_issue`, the `expected_author` is `submission.github_username` which is user-supplied.
- **Impact**: A user could set `github_username` to any value in the submission, bypassing author verification for issues authored by that github user.

#### M7. `process_claims` doesn't verify the submission hotkey is registered
- **Location**: `src/validation.rs`, `process_claims`
- **Description**: `process_claims` directly calls `record_valid_issue` with `submission.hotkey` without checking if it's a registered user. `record_valid_issue` increments `balance:{hotkey}` even for unregistered hotkeys. These unregistered hotkeys won't appear on the leaderboard (since they're not in `registered_hotkeys`), but their balance data accumulates silently.
- **Impact**: Phantom balance entries for unregistered hotkeys. If the hotkey registers later, those balances are retroactively picked up.

---

### MINOR Issues

#### m1. Re-registration overwrites `registered_epoch`
- **Location**: `src/storage/bounty_storage.rs`, `register_user`
- **Description**: If a hotkey re-registers with the same github username (valid re-registration), the `registered_epoch` is overwritten with the current epoch. This loses the original registration timestamp.
- **Impact**: Minor data loss — original registration date is lost on re-registration.

#### m2. Issue URL parsing in JSON claim doesn't validate domain or path structure
- **Location**: `src/api/handlers.rs`, `handle_claim` (JSON path, lines ~170-185)
- **Description**: The URL is split by `/` and only `parts.len() >= 7` is checked. There is no validation that `parts[2]` is `github.com` or that `parts[5]` is `issues`. A URL like `https://evil.com/owner/repo/pulls/123` would pass parsing and be treated as a valid issue claim. The claim would likely fail because the issue wouldn't exist in synced data, but the parsing is over-permissive.
- **Impact**: Low — the issue must still match synced data. But opens door to confusion or exploits if synced data is manipulated (see C2/C3).

#### m3. `record_valid_issue` hardcodes issue properties
- **Location**: `src/storage/bounty_storage.rs`, `record_valid_issue` (lines ~108-120)
- **Description**: When recording a claimed issue, properties like `is_closed: true`, `has_valid_label: true`, `has_ide_label: true` are hardcoded rather than copied from the actual synced issue data. The stored claim record doesn't reflect the actual issue state at claim time.
- **Impact**: Minor — the values represent required conditions for a valid claim, so they're correct by construction. But if requirements change, these would need manual updating.

#### m4. `weight` field in `StatusResponse` is raw (un-normalized) value
- **Location**: `src/api/handlers.rs`, `handle_status` and `handle_hotkey_details`
- **Description**: The `weight` field is computed as `calculate_weight_from_points(valid_count, star_count)` which returns an absolute value (not normalized 0-1). Meanwhile, `/get_weights` returns normalized weights that sum to 1.0. The two "weight" concepts are different but use the same name.
- **Impact**: API consumers may confuse the raw weight in `/status/:hotkey` with the normalized weight from `/get_weights`. Documentation/naming concern.

#### m5. `synced_issues` stored as single monolithic blob
- **Location**: `src/storage/bounty_storage.rs`, `store_issue_data` / `get_synced_issues`
- **Description**: All synced issues are serialized and stored as a single key-value pair. Every read deserializes the entire collection. Every sync replaces the entire collection atomically.
- **Impact**: Scalability concern as issue count grows. Not a correctness issue.

#### m6. Atomicity concern in `register_user`
- **Location**: `src/storage/bounty_storage.rs`, `register_user`
- **Description**: Registration writes two keys: `user:{hotkey}` and `github:{username}`. If the first write succeeds but the second fails, the user record exists but the github→hotkey reverse mapping does not. The function returns `false` if the second write fails, but the first write is not rolled back.
- **Impact**: Inconsistent state on partial failure — the user appears registered by hotkey but not findable by github username. Depends on host storage reliability.

#### m7. `get_pending_issues` filter logic may be wrong
- **Location**: `src/storage/bounty_storage.rs`, `get_pending_issues`
- **Description**: Pending issues are defined as `!i.is_closed && i.claimed_by_hotkey.is_none()`. But in `handle_issues_stats`, "pending" is defined as `i.is_closed && !i.has_valid_label && !i.has_invalid_label` (closed but not yet reviewed). These two definitions of "pending" are inconsistent.
- **Impact**: The `/issues/pending` endpoint and the `pending` count in `/issues/stats` show different things under the same name.

---

## Data Flow Summary

### Registration: `POST /register`
1. Auth check → deserialize `RegisterRequest` → extract hotkey from auth header (fallback to body)
2. `register_user`: checks 1-to-1 mapping (github↔hotkey), stores `user:{hotkey}` + `github:{name}`
3. `ensure_hotkey_tracked`: adds hotkey to `registered_hotkeys` list

**Gaps**: No github_username format/emptiness validation. No rate limiting.

### Issue Sync: `POST /issues/sync`
1. Auth check (any non-empty auth_hotkey) → deserialize `Vec<IssueRecord>` → store at `synced_issues`

**Gaps**: No authorization (anyone can sync). No data validation. Complete overwrite.

### Claiming: `POST /claim`
1. Auth check → parse JSON `ClaimRequest` (or legacy `BountySubmission`)
2. JSON path: parse URL, lookup github from auth_hotkey, build `BountySubmission`
3. Legacy path: deserialize from body (hotkey from body, not verified against auth)
4. `process_claims`: for each issue_number: check `is_issue_recorded` → find in synced_issues → `validate_issue` (closed, ide label, valid label, not invalid, author match, not claimed) → `record_valid_issue` (stores at `issue:...` key, increments `balance:{hotkey}.valid_count`)
5. If any issues claimed → `rebuild_leaderboard`

**Gaps**: Legacy path doesn't verify body hotkey == auth hotkey. Legacy path doesn't verify body github == registered github. No registration check in process_claims.

### Leaderboard: `rebuild_leaderboard()`
1. Iterate all `registered_hotkeys` → get `UserBalance` → compute `net_points` and `score`
2. Sort by `score` (ignores penalties) → assign ranks → store at `leaderboard`

**Gaps**: Sorted by `score` (not `net_points`). Stale `is_penalized` flag used.

### Weight Calculation: `calculate_weights_from_leaderboard()`
1. Filter: `!is_penalized && net_points > 0.0`
2. Normalize: each weight = `net_points / total_net_points`

**Logic is correct** given accurate input data. Issues stem from stale `is_penalized` and leaderboard sorting inconsistency upstream.
