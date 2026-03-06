use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::storage;
use crate::types::{LeaderboardEntry, SyncResult};

pub const WEIGHT_PER_POINT: f64 = 0.02;
pub const STAR_BONUS_PER_REPO: f64 = 0.25;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeightAssignment {
    pub hotkey: String,
    pub weight: f64,
}

pub fn calculate_weight_from_points(valid_count: u32, star_count: u32) -> f64 {
    let issue_points = valid_count as f64;
    let star_points = star_count as f64 * STAR_BONUS_PER_REPO;
    let total_points = issue_points + star_points;
    total_points * WEIGHT_PER_POINT
}

pub fn calculate_net_points(
    valid_count: u32,
    invalid_count: u32,
    duplicate_count: u32,
    malicious_count: u32,
    star_count: u32,
) -> f64 {
    let valid = valid_count as f64;
    let invalid = invalid_count as f64;
    let duplicate = duplicate_count as f64 * 0.5;
    let malicious = malicious_count as f64 * 5.0;
    let star_points = star_count as f64 * STAR_BONUS_PER_REPO;
    (valid - invalid - duplicate - malicious + star_points).max(0.0)
}

/// Compute weights deterministically from committed issues in P2P storage.
/// Does NOT read or write balances -- recomputes everything in-memory from
/// the issue records. This ensures all validators with the same committed
/// issues produce identical weight vectors (critical for vTrust).
pub fn compute_weights_from_issues() -> Vec<WeightAssignment> {
    use alloc::collections::BTreeMap;

    let all_issues = storage::get_synced_issues();
    let hotkeys = storage::get_registered_hotkeys();

    // Recount balances in-memory (not from stored balances)
    let mut valid_counts: BTreeMap<String, u32> = BTreeMap::new();
    let mut invalid_counts: BTreeMap<String, u32> = BTreeMap::new();
    let mut duplicate_counts: BTreeMap<String, u32> = BTreeMap::new();
    let mut malicious_counts: BTreeMap<String, u32> = BTreeMap::new();

    for issue in &all_issues {
        let hotkey = match &issue.claimed_by_hotkey {
            Some(h) if !h.is_empty() => h.clone(),
            _ => match storage::get_hotkey_by_github(&issue.author) {
                Some(h) => h,
                None => continue,
            },
        };

        if issue.has_malicious_label {
            *malicious_counts.entry(hotkey).or_insert(0) += 1;
        } else if issue.has_invalid_label {
            *invalid_counts.entry(hotkey).or_insert(0) += 1;
        } else if issue.has_duplicate_label {
            *duplicate_counts.entry(hotkey).or_insert(0) += 1;
        } else if issue.has_valid_label {
            *valid_counts.entry(hotkey).or_insert(0) += 1;
        }
    }

    // Build leaderboard entries in-memory
    let mut entries = Vec::with_capacity(hotkeys.len());
    for hotkey in &hotkeys {
        let hk = crate::ss58::normalize_hotkey(hotkey).unwrap_or_else(|| hotkey.clone());
        let valid = valid_counts.get(&hk).copied().unwrap_or(0);
        let invalid = invalid_counts.get(&hk).copied().unwrap_or(0);
        let duplicate = duplicate_counts.get(&hk).copied().unwrap_or(0);
        let malicious = malicious_counts.get(&hk).copied().unwrap_or(0);

        let net_points = calculate_net_points(valid, invalid, duplicate, malicious, 0);
        entries.push(LeaderboardEntry {
            rank: 0,
            hotkey: hk,
            github_username: String::new(),
            score: 0.0,
            valid_issues: valid,
            invalid_issues: invalid,
            pending_issues: 0,
            star_count: 0,
            star_bonus: 0.0,
            net_points,
            is_penalized: false,
            last_epoch: 0,
            duplicate_issues: duplicate,
            malicious_issues: malicious,
        });
    }

    calculate_weights_from_leaderboard(&entries)
}

pub fn calculate_weights_from_leaderboard(entries: &[LeaderboardEntry]) -> Vec<WeightAssignment> {
    let mut weights: Vec<WeightAssignment> = entries
        .iter()
        .filter(|e| !crate::storage::is_banned(&e.hotkey) && e.net_points > 0.0)
        .map(|e| WeightAssignment {
            hotkey: crate::ss58::normalize_hotkey(&e.hotkey).unwrap_or_else(|| e.hotkey.clone()),
            weight: e.net_points,
        })
        .collect();

    let total: f64 = weights.iter().map(|w| w.weight).sum();
    if total > 0.0 {
        for w in &mut weights {
            w.weight /= total;
        }
    }

    // Gradual emission scaling: weights reach full (1.0) at MATURITY_THRESHOLD
    // valid issues across the network. Below that, scale down with a sqrt curve
    // so early contributors still get meaningful rewards but the system ramps up
    // smoothly. The remainder goes to burn (UID 0) via fill_with_burn on the
    // validator side.
    const MATURITY_THRESHOLD: f64 = 100.0;
    let total_valid: f64 = entries.iter().map(|e| e.valid_issues as f64).sum();
    let scale = if total_valid >= MATURITY_THRESHOLD {
        1.0
    } else {
        (total_valid / MATURITY_THRESHOLD).sqrt()
    };

    if scale < 1.0 {
        for w in &mut weights {
            w.weight *= scale;
        }
    }

    weights.sort_by(|a, b| {
        b.weight
            .partial_cmp(&a.weight)
            .unwrap_or(core::cmp::Ordering::Equal)
    });

    weights
}

/// Rebuild the leaderboard from registered hotkeys and their balances.
/// Returns the computed entries directly AND writes them to storage (P2P consensus).
/// Callers that need the result immediately (sync, get_weights) should use
/// the returned value instead of reading back from storage, because the P2P
/// write may not have landed yet.
pub fn rebuild_leaderboard() -> Vec<LeaderboardEntry> {
    let hotkeys = storage::get_registered_hotkeys();
    let mut entries = Vec::with_capacity(hotkeys.len());

    for hotkey in &hotkeys {
        let balance = storage::get_user_balance(hotkey);
        let github_username = storage::get_github_by_hotkey(hotkey).unwrap_or_default();

        let net_points = calculate_net_points(
            balance.valid_count,
            balance.invalid_count,
            balance.duplicate_count,
            balance.malicious_count,
            balance.star_count,
        );
        let score = net_points * WEIGHT_PER_POINT;

        let epoch = platform_challenge_sdk_wasm::host_functions::host_consensus_get_epoch();
        let current_epoch = if epoch >= 0 { epoch as u64 } else { 0 };

        entries.push(LeaderboardEntry {
            rank: 0,
            hotkey: crate::ss58::normalize_hotkey(hotkey).unwrap_or_else(|| hotkey.clone()),
            github_username,
            score,
            valid_issues: balance.valid_count,
            invalid_issues: balance.invalid_count,
            pending_issues: 0,
            star_count: balance.star_count,
            star_bonus: balance.star_count as f64 * STAR_BONUS_PER_REPO,
            net_points,
            is_penalized: balance.is_penalized,
            last_epoch: current_epoch,
            duplicate_issues: balance.duplicate_count,
            malicious_issues: balance.malicious_count,
        });
    }

    entries.sort_by(|a, b| {
        b.net_points
            .partial_cmp(&a.net_points)
            .unwrap_or(core::cmp::Ordering::Equal)
    });

    for (i, entry) in entries.iter_mut().enumerate() {
        entry.rank = (i + 1) as u32;
    }

    storage::store_leaderboard(&entries);
    entries
}

/// Lazily fetch GitHub issues + recount if 20 min elapsed.
/// Called from handle_route so it works even when sync/background_tick
/// are not running (e.g. bootnode).
pub fn maybe_refresh() {
    const GITHUB_FETCH_INTERVAL_MS: i64 = 20 * 60 * 1000;

    let now = platform_challenge_sdk_wasm::host_functions::host_get_timestamp();
    let last = storage::get_last_refreshed();

    if last == 0 || (now - last) >= GITHUB_FETCH_INTERVAL_MS {
        crate::github_sync::fetch_and_process_issues();
        storage::recount_all_balances();
        storage::store_last_refreshed(now);
    }
}

/// Called every ~12s on the persistent WASM instance.
/// Recount balances and rebuild leaderboard on every tick.
/// Every ~20 min, fetch fresh issues from GitHub (throttled by timestamp).
pub fn background_tick() {
    const GITHUB_FETCH_INTERVAL_MS: i64 = 20 * 60 * 1000;

    let now = platform_challenge_sdk_wasm::host_functions::host_get_timestamp();
    let last = storage::get_last_refreshed();

    if last == 0 || (now - last) >= GITHUB_FETCH_INTERVAL_MS {
        crate::github_sync::fetch_and_process_issues();
        storage::store_last_refreshed(now);
    }

    storage::recount_all_balances();
    rebuild_leaderboard();
}

/// Perform a full sync: rebuild leaderboard and return sync result for consensus
pub fn perform_sync() -> SyncResult {
    const GITHUB_FETCH_INTERVAL_MS: i64 = 20 * 60 * 1000;

    let now = platform_challenge_sdk_wasm::host_functions::host_get_timestamp();
    let last = storage::get_last_refreshed();

    if last == 0 || (now - last) >= GITHUB_FETCH_INTERVAL_MS {
        crate::github_sync::fetch_and_process_issues();
        storage::store_last_refreshed(now);
    }

    storage::recount_all_balances();

    let entries = rebuild_leaderboard();
    let hotkeys = storage::get_registered_hotkeys();

    // Calculate totals
    let mut total_valid = 0u32;
    let mut total_invalid = 0u32;

    for hotkey in &hotkeys {
        let balance = storage::get_user_balance(hotkey);
        total_valid = total_valid.saturating_add(balance.valid_count);
        total_invalid = total_invalid.saturating_add(balance.invalid_count);
    }

    // Count pending issues
    let total_pending = storage::get_pending_issues_count();

    // Hash the leaderboard for consensus comparison
    let leaderboard_data = bincode::serialize(&entries).unwrap_or_default();
    let leaderboard_hash = {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&leaderboard_data);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    };

    let epoch = platform_challenge_sdk_wasm::host_functions::host_consensus_get_epoch();

    SyncResult {
        leaderboard_hash,
        total_users: hotkeys.len() as u32,
        total_valid_issues: total_valid,
        total_invalid_issues: total_invalid,
        total_pending_issues: total_pending,
        sync_timestamp: epoch,
    }
}
