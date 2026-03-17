use alloc::string::{String, ToString};
use alloc::vec::Vec;
use platform_challenge_sdk_wasm::host_functions::{
    host_consensus_get_epoch, host_storage_get, host_storage_list_prefix, host_storage_set,
};
use serde::Deserialize;

use crate::ss58;
use crate::types::{
    InvalidIssueRecord, IssueRecord, LeaderboardEntry, UserBalance, UserRegistration,
};

const MAX_SYNCED_ISSUES: usize = 500_000;

fn make_key(prefix: &[u8], suffix: &str) -> Vec<u8> {
    let mut key = Vec::from(prefix);
    key.extend_from_slice(suffix.as_bytes());
    key
}

/// Normalize hotkey to SS58 for storage. Falls back to original if conversion fails.
fn normalize_hotkey_for_storage(hotkey: &str) -> String {
    ss58::normalize_hotkey(hotkey).unwrap_or_else(|| hotkey.to_string())
}

fn issue_key(repo_owner: &str, repo_name: &str, issue_number: u32) -> Vec<u8> {
    let mut key = Vec::from(b"issue:" as &[u8]);
    key.extend_from_slice(repo_owner.as_bytes());
    key.push(b'/');
    key.extend_from_slice(repo_name.as_bytes());
    key.push(b':');
    key.extend_from_slice(&issue_number.to_le_bytes());
    key
}

pub fn register_user(github_username: &str, hotkey: &str) -> bool {
    let hotkey_ss58 = normalize_hotkey_for_storage(hotkey);

    let existing_hotkey = get_hotkey_by_github(github_username);
    if let Some(ref existing) = existing_hotkey {
        if existing != &hotkey_ss58 {
            return false;
        }
    }

    let existing_github = get_github_by_hotkey(&hotkey_ss58);
    if let Some(ref existing) = existing_github {
        if existing.to_lowercase() != github_username.to_lowercase() {
            return false;
        }
    }

    let epoch = host_consensus_get_epoch();
    let current_epoch = if epoch >= 0 { epoch as u64 } else { 0 };

    let registration = UserRegistration {
        hotkey: hotkey_ss58.clone(),
        github_username: String::from(github_username),
        registered_epoch: current_epoch,
    };

    let data = match bincode::serialize(&registration) {
        Ok(d) => d,
        Err(_) => return false,
    };

    let user_key = make_key(b"user:", &hotkey_ss58);
    if host_storage_set(&user_key, &data).is_err() {
        return false;
    }

    let github_key = make_key(b"github:", &github_username.to_lowercase());
    if host_storage_set(&github_key, hotkey_ss58.as_bytes()).is_err() {
        return false;
    }

    true
}

pub fn get_user_by_hotkey(hotkey: &str) -> Option<UserRegistration> {
    let hotkey_ss58 = normalize_hotkey_for_storage(hotkey);

    // Try SS58 key first
    let key = make_key(b"user:", &hotkey_ss58);
    let data = host_storage_get(&key).ok()?;
    if !data.is_empty() {
        return bincode::deserialize(&data).ok();
    }

    // Fallback: try original key (for migration)
    if hotkey != hotkey_ss58 {
        let key = make_key(b"user:", hotkey);
        let data = host_storage_get(&key).ok()?;
        if !data.is_empty() {
            return bincode::deserialize(&data).ok();
        }
    }

    None
}

pub fn get_hotkey_by_github(github_username: &str) -> Option<String> {
    let key = make_key(b"github:", &github_username.to_lowercase());
    let data = host_storage_get(&key).ok()?;
    if data.is_empty() {
        return None;
    }
    let hotkey = String::from_utf8(data).ok()?;
    // Normalize to SS58 on read
    Some(normalize_hotkey_for_storage(&hotkey))
}

pub fn get_github_by_hotkey(hotkey: &str) -> Option<String> {
    let reg = get_user_by_hotkey(hotkey)?;
    Some(reg.github_username)
}

pub fn record_valid_issue(
    issue_number: u32,
    repo_owner: &str,
    repo_name: &str,
    author: &str,
    hotkey: &str,
) -> bool {
    let hotkey_ss58 = normalize_hotkey_for_storage(hotkey);
    let key = issue_key(repo_owner, repo_name, issue_number);

    let epoch = host_consensus_get_epoch();
    let current_epoch = if epoch >= 0 { epoch as u64 } else { 0 };

    let record = IssueRecord {
        issue_number,
        repo_owner: String::from(repo_owner),
        repo_name: String::from(repo_name),
        author: String::from(author),
        is_closed: true,
        has_valid_label: true,
        has_invalid_label: false,
        has_ide_label: true,
        claimed_by_hotkey: Some(hotkey_ss58.clone()),
        recorded_epoch: current_epoch,
        has_duplicate_label: false,
        has_malicious_label: false,
        created_at_ms: 0,
    };

    let data = match bincode::serialize(&record) {
        Ok(d) => d,
        Err(_) => return false,
    };

    if host_storage_set(&key, &data).is_err() {
        return false;
    }

    true
}

pub fn record_invalid_issue(
    issue_number: u32,
    repo_owner: &str,
    repo_name: &str,
    github_username: &str,
    reason: Option<&str>,
) -> bool {
    let epoch = host_consensus_get_epoch();
    let current_epoch = if epoch >= 0 { epoch as u64 } else { 0 };

    let inv_record = InvalidIssueRecord {
        issue_number,
        repo_owner: String::from(repo_owner),
        repo_name: String::from(repo_name),
        github_username: String::from(github_username),
        reason: reason.map(String::from),
        recorded_epoch: current_epoch,
    };

    let mut inv_key = Vec::from(b"invalid_issue:" as &[u8]);
    inv_key.extend_from_slice(repo_owner.as_bytes());
    inv_key.push(b'/');
    inv_key.extend_from_slice(repo_name.as_bytes());
    inv_key.push(b':');
    inv_key.extend_from_slice(&issue_number.to_le_bytes());

    let inv_data = match bincode::serialize(&inv_record) {
        Ok(d) => d,
        Err(_) => return false,
    };

    if host_storage_set(&inv_key, &inv_data).is_err() {
        return false;
    }

    // Also write an IssueRecord under the canonical issue: key so
    // get_issue_record/is_issue_recorded can detect label changes
    let hotkey = get_hotkey_by_github(github_username);
    let issue_record = IssueRecord {
        issue_number,
        repo_owner: String::from(repo_owner),
        repo_name: String::from(repo_name),
        author: String::from(github_username),
        is_closed: true,
        has_valid_label: false,
        has_invalid_label: true,
        has_ide_label: false,
        claimed_by_hotkey: hotkey,
        recorded_epoch: current_epoch,
        has_duplicate_label: false,
        has_malicious_label: false,
        created_at_ms: 0,
    };
    let key = issue_key(repo_owner, repo_name, issue_number);
    if let Ok(data) = bincode::serialize(&issue_record) {
        let _ = host_storage_set(&key, &data);
    }

    true
}

pub fn record_duplicate_issue(
    issue_number: u32,
    repo_owner: &str,
    repo_name: &str,
    github_username: &str,
    hotkey: &str,
) -> bool {
    let hotkey_ss58 = normalize_hotkey_for_storage(hotkey);
    let key = issue_key(repo_owner, repo_name, issue_number);

    let epoch = host_consensus_get_epoch();
    let current_epoch = if epoch >= 0 { epoch as u64 } else { 0 };

    let record = IssueRecord {
        issue_number,
        repo_owner: String::from(repo_owner),
        repo_name: String::from(repo_name),
        author: String::from(github_username),
        is_closed: true,
        has_valid_label: false,
        has_invalid_label: false,
        has_ide_label: false,
        claimed_by_hotkey: Some(hotkey_ss58),
        recorded_epoch: current_epoch,
        has_duplicate_label: true,
        has_malicious_label: false,
        created_at_ms: 0,
    };

    let data = match bincode::serialize(&record) {
        Ok(d) => d,
        Err(_) => return false,
    };

    if host_storage_set(&key, &data).is_err() {
        return false;
    }

    // Also write an InvalidIssueRecord so recount picks it up
    let inv_record = InvalidIssueRecord {
        issue_number,
        repo_owner: String::from(repo_owner),
        repo_name: String::from(repo_name),
        github_username: String::from(github_username),
        reason: Some(String::from("Issue marked duplicate")),
        recorded_epoch: current_epoch,
    };
    let mut inv_key = Vec::from(b"invalid_issue:" as &[u8]);
    inv_key.extend_from_slice(repo_owner.as_bytes());
    inv_key.push(b'/');
    inv_key.extend_from_slice(repo_name.as_bytes());
    inv_key.push(b':');
    inv_key.extend_from_slice(&issue_number.to_le_bytes());
    if let Ok(inv_data) = bincode::serialize(&inv_record) {
        let _ = host_storage_set(&inv_key, &inv_data);
    }

    true
}

pub fn record_malicious_issue(
    issue_number: u32,
    repo_owner: &str,
    repo_name: &str,
    github_username: &str,
) -> bool {
    let epoch = host_consensus_get_epoch();
    let current_epoch = if epoch >= 0 { epoch as u64 } else { 0 };

    let hotkey = get_hotkey_by_github(github_username);
    let record = IssueRecord {
        issue_number,
        repo_owner: String::from(repo_owner),
        repo_name: String::from(repo_name),
        author: String::from(github_username),
        is_closed: true,
        has_valid_label: false,
        has_invalid_label: false,
        has_ide_label: false,
        claimed_by_hotkey: hotkey,
        recorded_epoch: current_epoch,
        has_duplicate_label: false,
        has_malicious_label: true,
        created_at_ms: 0,
    };
    let key = issue_key(repo_owner, repo_name, issue_number);
    if let Ok(data) = bincode::serialize(&record) {
        let _ = host_storage_set(&key, &data);
    }

    let mut mal_key = Vec::from(b"malicious_issue:" as &[u8]);
    mal_key.extend_from_slice(repo_owner.as_bytes());
    mal_key.push(b'/');
    mal_key.extend_from_slice(repo_name.as_bytes());
    mal_key.push(b':');
    mal_key.extend_from_slice(&issue_number.to_le_bytes());
    let inv_record = InvalidIssueRecord {
        issue_number,
        repo_owner: String::from(repo_owner),
        repo_name: String::from(repo_name),
        github_username: String::from(github_username),
        reason: Some(String::from("Issue marked malicious")),
        recorded_epoch: current_epoch,
    };
    if let Ok(data) = bincode::serialize(&inv_record) {
        let _ = host_storage_set(&mal_key, &data);
    }

    true
}

pub fn is_issue_recorded(repo_owner: &str, repo_name: &str, issue_number: u32) -> bool {
    let key = issue_key(repo_owner, repo_name, issue_number);
    if let Ok(data) = host_storage_get(&key) {
        return !data.is_empty();
    }
    false
}

pub fn get_issue_record(
    repo_owner: &str,
    repo_name: &str,
    issue_number: u32,
) -> Option<IssueRecord> {
    let key = issue_key(repo_owner, repo_name, issue_number);
    let data = host_storage_get(&key).ok()?;
    if data.is_empty() {
        return None;
    }
    deserialize_issue_record(&data)
}

pub fn delete_issue_record(repo_owner: &str, repo_name: &str, issue_number: u32) {
    // Clear both canonical key and invalid_issue key
    let key = issue_key(repo_owner, repo_name, issue_number);
    let _ = host_storage_set(&key, &[]);

    let mut inv_key = Vec::from(b"invalid_issue:" as &[u8]);
    inv_key.extend_from_slice(repo_owner.as_bytes());
    inv_key.push(b'/');
    inv_key.extend_from_slice(repo_name.as_bytes());
    inv_key.push(b':');
    inv_key.extend_from_slice(&issue_number.to_le_bytes());
    let _ = host_storage_set(&inv_key, &[]);

    let mut mal_key = Vec::from(b"malicious_issue:" as &[u8]);
    mal_key.extend_from_slice(repo_owner.as_bytes());
    mal_key.push(b'/');
    mal_key.extend_from_slice(repo_name.as_bytes());
    mal_key.push(b':');
    mal_key.extend_from_slice(&issue_number.to_le_bytes());
    let _ = host_storage_set(&mal_key, &[]);
}

/// Legacy struct without malicious_count for bincode compat
#[derive(Clone, Debug, Deserialize)]
struct LegacyUserBalance {
    pub valid_count: u32,
    pub invalid_count: u32,
    pub duplicate_count: u32,
    pub star_count: u32,
    pub is_penalized: bool,
}

impl LegacyUserBalance {
    fn into_current(self) -> UserBalance {
        UserBalance {
            valid_count: self.valid_count,
            invalid_count: self.invalid_count,
            duplicate_count: self.duplicate_count,
            star_count: self.star_count,
            is_penalized: self.is_penalized,
            malicious_count: 0,
        }
    }
}

fn deserialize_user_balance(data: &[u8]) -> Option<UserBalance> {
    bincode::deserialize::<UserBalance>(data).ok().or_else(|| {
        bincode::deserialize::<LegacyUserBalance>(data)
            .ok()
            .map(|l| l.into_current())
    })
}

pub fn get_user_balance(hotkey: &str) -> UserBalance {
    let hotkey_ss58 = normalize_hotkey_for_storage(hotkey);
    let key = make_key(b"balance:", &hotkey_ss58);
    let result = host_storage_get(&key).ok().and_then(|d| {
        if d.is_empty() {
            None
        } else {
            deserialize_user_balance(&d)
        }
    });

    // Fallback to original key for migration
    if result.is_none() && hotkey != hotkey_ss58 {
        let key = make_key(b"balance:", hotkey);
        return host_storage_get(&key)
            .ok()
            .and_then(|d| {
                if d.is_empty() {
                    None
                } else {
                    deserialize_user_balance(&d)
                }
            })
            .unwrap_or_default();
    }

    result.unwrap_or_default()
}

pub fn store_user_balance(hotkey: &str, balance: &UserBalance) {
    let hotkey_ss58 = normalize_hotkey_for_storage(hotkey);
    let key = make_key(b"balance:", &hotkey_ss58);
    if let Ok(data) = bincode::serialize(balance) {
        let _ = host_storage_set(&key, &data);
    }
}

pub fn increment_duplicate_count(hotkey: &str) {
    let hotkey_ss58 = normalize_hotkey_for_storage(hotkey);
    let mut balance = get_user_balance(&hotkey_ss58);
    balance.duplicate_count = balance.duplicate_count.saturating_add(1);
    let penalty_points = balance.invalid_count as f64 + balance.duplicate_count as f64 * 0.5;
    balance.is_penalized = penalty_points > 0.0;
    store_user_balance(&hotkey_ss58, &balance);
}

/// Rebuild github:{username} -> hotkey index from user:{hotkey} records.
/// Called at recount time to recover from missing github: keys.
pub fn rebuild_github_index() {
    let hotkeys = get_registered_hotkeys();
    for hk in &hotkeys {
        if let Some(reg) = get_user_by_hotkey(hk) {
            if !reg.github_username.is_empty() {
                let key = make_key(b"github:", &reg.github_username.to_lowercase());
                let _ = host_storage_set(&key, reg.hotkey.as_bytes());
            }
        }
    }
}

/// Recount all balances by scanning stored issues. Returns JSON summary.
pub fn recount_all_balances() -> serde_json::Value {
    use alloc::collections::BTreeMap;

    let all_issues = get_synced_issues();
    let mut valid_counts: BTreeMap<String, u32> = BTreeMap::new();
    let mut invalid_counts: BTreeMap<String, u32> = BTreeMap::new();
    let mut duplicate_counts: BTreeMap<String, u32> = BTreeMap::new();
    let mut malicious_counts: BTreeMap<String, u32> = BTreeMap::new();

    for issue in &all_issues {
        let hotkey = match &issue.claimed_by_hotkey {
            Some(h) if !h.is_empty() => h.clone(),
            _ => match get_hotkey_by_github(&issue.author) {
                Some(h) => h,
                None => continue,
            },
        };

        if issue.has_malicious_label {
            *malicious_counts.entry(hotkey.clone()).or_insert(0) += 1;
        } else if issue.has_invalid_label {
            *invalid_counts.entry(hotkey.clone()).or_insert(0) += 1;
        } else if issue.has_duplicate_label {
            *duplicate_counts.entry(hotkey.clone()).or_insert(0) += 1;
        } else if issue.has_valid_label {
            *valid_counts.entry(hotkey.clone()).or_insert(0) += 1;
        }
    }

    let mut all_hotkeys: BTreeMap<String, bool> = BTreeMap::new();
    for k in valid_counts.keys() {
        all_hotkeys.insert(k.clone(), true);
    }
    for k in invalid_counts.keys() {
        all_hotkeys.insert(k.clone(), true);
    }
    for k in duplicate_counts.keys() {
        all_hotkeys.insert(k.clone(), true);
    }
    for k in malicious_counts.keys() {
        all_hotkeys.insert(k.clone(), true);
    }
    for hk in get_registered_hotkeys() {
        all_hotkeys.insert(hk, true);
    }

    let mut updated = 0u32;
    for hotkey in all_hotkeys.keys() {
        let mut balance = UserBalance::default();
        let old = get_user_balance(hotkey);
        balance.star_count = old.star_count;

        balance.valid_count = valid_counts.get(hotkey).copied().unwrap_or(0);
        balance.invalid_count = invalid_counts.get(hotkey).copied().unwrap_or(0);
        balance.duplicate_count = duplicate_counts.get(hotkey).copied().unwrap_or(0);
        balance.malicious_count = malicious_counts.get(hotkey).copied().unwrap_or(0);
        let penalty_points = balance.invalid_count as f64
            + balance.duplicate_count as f64 * 0.5
            + balance.malicious_count as f64 * 5.0;
        balance.is_penalized = penalty_points > 0.0;
        store_user_balance(hotkey, &balance);
        updated += 1;
    }

    serde_json::json!({
        "success": true,
        "total_issues_scanned": all_issues.len(),
        "hotkeys_updated": updated,
        "unique_valid_hotkeys": valid_counts.len(),
        "unique_invalid_hotkeys": invalid_counts.len(),
        "unique_malicious_hotkeys": malicious_counts.len()
    })
}

pub fn get_leaderboard() -> Vec<LeaderboardEntry> {
    host_storage_get(b"leaderboard")
        .ok()
        .and_then(|d| {
            if d.is_empty() {
                None
            } else {
                bincode::deserialize(&d).ok()
            }
        })
        .unwrap_or_default()
}

pub fn store_leaderboard(entries: &[LeaderboardEntry]) -> bool {
    if let Ok(data) = bincode::serialize(entries) {
        return host_storage_set(b"leaderboard", &data).is_ok();
    }
    false
}

/// Deserialize the bincode Vec<(Vec<u8>, Vec<u8>)> returned by list_prefix
fn decode_list_prefix(data: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
    bincode::deserialize(data).unwrap_or_default()
}

pub fn get_registered_hotkeys() -> Vec<String> {
    // Try indexed storage first (hotkey_idx: prefix)
    if let Ok(data) = host_storage_list_prefix(b"hotkey_idx:", 10_000) {
        if !data.is_empty() {
            let pairs = decode_list_prefix(&data);
            if !pairs.is_empty() {
                return pairs
                    .into_iter()
                    .filter_map(|(_k, v)| String::from_utf8(v).ok())
                    .collect();
            }
        }
    }

    // Fallback: legacy blob
    host_storage_get(b"registered_hotkeys")
        .ok()
        .and_then(|d| {
            if d.is_empty() {
                None
            } else {
                bincode::deserialize(&d).ok()
            }
        })
        .unwrap_or_default()
}

fn add_registered_hotkey(hotkey: &str) {
    let hotkey_ss58 = normalize_hotkey_for_storage(hotkey);

    // Write individual indexed key
    let idx_key = make_key(b"hotkey_idx:", &hotkey_ss58);
    let _ = host_storage_set(&idx_key, hotkey_ss58.as_bytes());
}

pub fn store_issue_data(issues: &[IssueRecord]) -> bool {
    let truncated = if issues.len() > MAX_SYNCED_ISSUES {
        &issues[..MAX_SYNCED_ISSUES]
    } else {
        issues
    };
    // Store as a single blob -- this is the only source of truth.
    // Individual issue: records are no longer written to avoid stale accumulation.
    if let Ok(data) = bincode::serialize(truncated) {
        return host_storage_set(b"synced_issues", &data).is_ok();
    }
    false
}

/// Legacy struct without has_malicious_label for bincode compat
#[derive(Clone, Debug, Deserialize)]
struct LegacyIssueRecord {
    pub issue_number: u32,
    pub repo_owner: String,
    pub repo_name: String,
    pub author: String,
    pub is_closed: bool,
    pub has_valid_label: bool,
    pub has_invalid_label: bool,
    pub has_ide_label: bool,
    pub claimed_by_hotkey: Option<String>,
    pub recorded_epoch: u64,
    pub has_duplicate_label: bool,
}

impl LegacyIssueRecord {
    fn into_current(self) -> IssueRecord {
        IssueRecord {
            issue_number: self.issue_number,
            repo_owner: self.repo_owner,
            repo_name: self.repo_name,
            author: self.author,
            is_closed: self.is_closed,
            has_valid_label: self.has_valid_label,
            has_invalid_label: self.has_invalid_label,
            has_ide_label: self.has_ide_label,
            claimed_by_hotkey: self.claimed_by_hotkey,
            recorded_epoch: self.recorded_epoch,
            has_duplicate_label: self.has_duplicate_label,
            has_malicious_label: false,
            created_at_ms: 0,
        }
    }
}

fn deserialize_issue_record(data: &[u8]) -> Option<IssueRecord> {
    bincode::deserialize::<IssueRecord>(data).ok().or_else(|| {
        bincode::deserialize::<LegacyIssueRecord>(data)
            .ok()
            .map(|l| l.into_current())
    })
}

pub fn get_synced_issues() -> Vec<IssueRecord> {
    // Single source of truth: the synced_issues blob.
    // Individual issue: records are legacy and must NOT be read to avoid
    // stale data accumulation from P2P state sync.
    host_storage_get(b"synced_issues")
        .ok()
        .and_then(|d| {
            if d.is_empty() {
                None
            } else {
                bincode::deserialize::<Vec<IssueRecord>>(&d)
                    .ok()
                    .or_else(|| {
                        // Try legacy format
                        bincode::deserialize::<Vec<LegacyIssueRecord>>(&d)
                            .ok()
                            .map(|v| v.into_iter().map(|l| l.into_current()).collect())
                    })
            }
        })
        .unwrap_or_default()
}

pub fn get_pending_issues() -> Vec<IssueRecord> {
    let issues = get_synced_issues();
    issues
        .into_iter()
        .filter(|i| !i.is_closed && i.claimed_by_hotkey.is_none())
        .collect()
}

pub fn store_active_miner_count(count: u64) {
    let _ = host_storage_set(b"active_miner_count", &count.to_le_bytes());
}

pub fn get_active_miner_count() -> u64 {
    host_storage_get(b"active_miner_count")
        .ok()
        .and_then(|d| {
            if d.len() >= 8 {
                let mut buf = [0u8; 8];
                buf.copy_from_slice(&d[..8]);
                Some(u64::from_le_bytes(buf))
            } else {
                None
            }
        })
        .unwrap_or(0)
}

pub fn store_validator_count(count: u64) {
    let _ = host_storage_set(b"validator_count", &count.to_le_bytes());
}

pub fn get_validator_count() -> u64 {
    host_storage_get(b"validator_count")
        .ok()
        .and_then(|d| {
            if d.len() >= 8 {
                let mut buf = [0u8; 8];
                buf.copy_from_slice(&d[..8]);
                Some(u64::from_le_bytes(buf))
            } else {
                None
            }
        })
        .unwrap_or(0)
}

pub fn store_last_refreshed(ts: i64) {
    let _ = host_storage_set(b"last_refreshed", &ts.to_le_bytes());
}

pub fn get_last_refreshed() -> i64 {
    host_storage_get(b"last_refreshed")
        .ok()
        .and_then(|d| {
            if d.len() >= 8 {
                let mut buf = [0u8; 8];
                buf.copy_from_slice(&d[..8]);
                Some(i64::from_le_bytes(buf))
            } else {
                None
            }
        })
        .unwrap_or(0)
}

pub fn ensure_hotkey_tracked(hotkey: &str) {
    add_registered_hotkey(hotkey);
}

const SUDO_OWNER_KEY: &[u8] = b"sudo_owner";
const DEFAULT_SUDO_OWNER: &str = "5GziQCcRpN8NCJktX343brnfuVe3w6gUYieeStXPD1Dag2At";

pub fn get_sudo_owner() -> Option<String> {
    let data = host_storage_get(SUDO_OWNER_KEY).ok()?;
    if data.is_empty() {
        return None;
    }
    String::from_utf8(data).ok()
}

pub fn set_sudo_owner(hotkey: &str) {
    let _ = host_storage_set(SUDO_OWNER_KEY, hotkey.as_bytes());
}

pub fn is_sudo_owner(hotkey: &str) -> bool {
    let hotkey_normalized = normalize_hotkey_for_storage(hotkey);
    match get_sudo_owner() {
        Some(owner) => {
            let owner_normalized = normalize_hotkey_for_storage(&owner);
            owner_normalized == hotkey_normalized
        }
        None => hotkey_normalized == DEFAULT_SUDO_OWNER,
    }
}

pub fn bulk_register_users(entries: &[(String, String)]) -> (u32, u32) {
    let mut success_count = 0u32;
    let mut skip_count = 0u32;

    for (hotkey, github_username) in entries {
        if hotkey.is_empty() || github_username.is_empty() {
            skip_count += 1;
            continue;
        }

        let existing_hotkey = get_hotkey_by_github(github_username);
        if let Some(ref existing) = existing_hotkey {
            if existing != hotkey {
                skip_count += 1;
                continue;
            }
        }

        let existing_github = get_github_by_hotkey(hotkey);
        if let Some(ref existing) = existing_github {
            if existing.to_lowercase() != github_username.to_lowercase() {
                skip_count += 1;
                continue;
            }
        }

        if register_user(github_username, hotkey) {
            ensure_hotkey_tracked(hotkey);
            success_count += 1;
        } else {
            skip_count += 1;
        }
    }

    (success_count, skip_count)
}

pub fn get_pending_issues_count() -> u32 {
    let issues = get_pending_issues();
    issues.len() as u32
}

pub fn ban_user(hotkey: &str) {
    let key = make_key(b"banned:", &normalize_hotkey_for_storage(hotkey));
    let _ = host_storage_set(&key, &[1]);
}

pub fn unban_user(hotkey: &str) {
    let key = make_key(b"banned:", &normalize_hotkey_for_storage(hotkey));
    let _ = host_storage_set(&key, &[]);
}

pub fn is_banned(hotkey: &str) -> bool {
    let key = make_key(b"banned:", &normalize_hotkey_for_storage(hotkey));
    if let Ok(data) = host_storage_get(&key) {
        return !data.is_empty();
    }
    false
}
