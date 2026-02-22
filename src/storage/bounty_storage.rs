use alloc::string::String;
use alloc::vec::Vec;
use platform_challenge_sdk_wasm::host_functions::{
    host_consensus_get_epoch, host_storage_get, host_storage_set,
};

use crate::types::{
    InvalidIssueRecord, IssueRecord, LeaderboardEntry, UserBalance, UserRegistration,
};

fn make_key(prefix: &[u8], suffix: &str) -> Vec<u8> {
    let mut key = Vec::from(prefix);
    key.extend_from_slice(suffix.as_bytes());
    key
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
    let existing_hotkey = get_hotkey_by_github(github_username);
    if let Some(ref existing) = existing_hotkey {
        if existing != hotkey {
            return false;
        }
    }

    let existing_github = get_github_by_hotkey(hotkey);
    if let Some(ref existing) = existing_github {
        if existing.to_lowercase() != github_username.to_lowercase() {
            return false;
        }
    }

    let epoch = host_consensus_get_epoch();
    let current_epoch = if epoch >= 0 { epoch as u64 } else { 0 };

    let registration = UserRegistration {
        hotkey: String::from(hotkey),
        github_username: String::from(github_username),
        registered_epoch: current_epoch,
    };

    let data = match bincode::serialize(&registration) {
        Ok(d) => d,
        Err(_) => return false,
    };

    let user_key = make_key(b"user:", hotkey);
    if host_storage_set(&user_key, &data).is_err() {
        return false;
    }

    let github_key = make_key(b"github:", &github_username.to_lowercase());
    if host_storage_set(&github_key, hotkey.as_bytes()).is_err() {
        return false;
    }

    true
}

pub fn get_user_by_hotkey(hotkey: &str) -> Option<UserRegistration> {
    let key = make_key(b"user:", hotkey);
    let data = host_storage_get(&key).ok()?;
    if data.is_empty() {
        return None;
    }
    bincode::deserialize(&data).ok()
}

pub fn get_hotkey_by_github(github_username: &str) -> Option<String> {
    let key = make_key(b"github:", &github_username.to_lowercase());
    let data = host_storage_get(&key).ok()?;
    if data.is_empty() {
        return None;
    }
    String::from_utf8(data).ok()
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
    let key = issue_key(repo_owner, repo_name, issue_number);

    if let Ok(data) = host_storage_get(&key) {
        if !data.is_empty() {
            return false;
        }
    }

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
        claimed_by_hotkey: Some(String::from(hotkey)),
        recorded_epoch: current_epoch,
    };

    let data = match bincode::serialize(&record) {
        Ok(d) => d,
        Err(_) => return false,
    };

    if host_storage_set(&key, &data).is_err() {
        return false;
    }

    increment_valid_count(hotkey);
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

    let record = InvalidIssueRecord {
        issue_number,
        repo_owner: String::from(repo_owner),
        repo_name: String::from(repo_name),
        github_username: String::from(github_username),
        reason: reason.map(String::from),
        recorded_epoch: current_epoch,
    };

    let mut key = Vec::from(b"invalid_issue:" as &[u8]);
    key.extend_from_slice(repo_owner.as_bytes());
    key.push(b'/');
    key.extend_from_slice(repo_name.as_bytes());
    key.push(b':');
    key.extend_from_slice(&issue_number.to_le_bytes());

    let data = match bincode::serialize(&record) {
        Ok(d) => d,
        Err(_) => return false,
    };

    if host_storage_set(&key, &data).is_err() {
        return false;
    }

    if let Some(hotkey) = get_hotkey_by_github(github_username) {
        increment_invalid_count(&hotkey);
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
    bincode::deserialize(&data).ok()
}

pub fn get_user_balance(hotkey: &str) -> UserBalance {
    let key = make_key(b"balance:", hotkey);
    host_storage_get(&key)
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

fn store_user_balance(hotkey: &str, balance: &UserBalance) {
    let key = make_key(b"balance:", hotkey);
    if let Ok(data) = bincode::serialize(balance) {
        let _ = host_storage_set(&key, &data);
    }
}

fn increment_valid_count(hotkey: &str) {
    let mut balance = get_user_balance(hotkey);
    balance.valid_count = balance.valid_count.saturating_add(1);
    store_user_balance(hotkey, &balance);
}

fn increment_invalid_count(hotkey: &str) {
    let mut balance = get_user_balance(hotkey);
    balance.invalid_count = balance.invalid_count.saturating_add(1);
    let penalty = balance
        .invalid_count
        .saturating_sub(balance.valid_count)
        .saturating_add(balance.duplicate_count.saturating_sub(balance.valid_count));
    balance.is_penalized = penalty > 0;
    store_user_balance(hotkey, &balance);
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

pub fn get_registered_hotkeys() -> Vec<String> {
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
    let mut hotkeys = get_registered_hotkeys();
    if !hotkeys.iter().any(|h| h == hotkey) {
        hotkeys.push(String::from(hotkey));
        if let Ok(data) = bincode::serialize(&hotkeys) {
            let _ = host_storage_set(b"registered_hotkeys", &data);
        }
    }
}

pub fn store_issue_data(issues: &[IssueRecord]) -> bool {
    if let Ok(data) = bincode::serialize(issues) {
        return host_storage_set(b"synced_issues", &data).is_ok();
    }
    false
}

pub fn get_synced_issues() -> Vec<IssueRecord> {
    host_storage_get(b"synced_issues")
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

pub fn ensure_hotkey_tracked(hotkey: &str) {
    add_registered_hotkey(hotkey);
}
