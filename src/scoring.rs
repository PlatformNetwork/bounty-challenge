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
    star_count: u32,
) -> f64 {
    let issue_points = valid_count as f64;
    let star_points = star_count as f64 * STAR_BONUS_PER_REPO;
    let total_negative = invalid_count.saturating_add(duplicate_count);
    let penalty = if total_negative > valid_count {
        (total_negative - valid_count) as f64
    } else {
        0.0
    };
    (issue_points + star_points - penalty).max(0.0)
}

pub fn calculate_weights_from_leaderboard(entries: &[LeaderboardEntry]) -> Vec<WeightAssignment> {
    let mut weights: Vec<WeightAssignment> = entries
        .iter()
        .filter(|e| !e.is_penalized && e.net_points > 0.0)
        .map(|e| WeightAssignment {
            hotkey: e.hotkey.clone(),
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

pub fn rebuild_leaderboard() {
    let hotkeys = storage::get_registered_hotkeys();
    let mut entries = Vec::with_capacity(hotkeys.len());

    for hotkey in &hotkeys {
        let balance = storage::get_user_balance(hotkey);
        let github_username = storage::get_github_by_hotkey(hotkey).unwrap_or_default();

        let net_points = calculate_net_points(
            balance.valid_count,
            balance.invalid_count,
            balance.duplicate_count,
            balance.star_count,
        );
        let score = calculate_weight_from_points(balance.valid_count, balance.star_count);

        let epoch = platform_challenge_sdk_wasm::host_functions::host_consensus_get_epoch();
        let current_epoch = if epoch >= 0 { epoch as u64 } else { 0 };

        entries.push(LeaderboardEntry {
            rank: 0,
            hotkey: hotkey.clone(),
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
}

/// Perform a full sync: rebuild leaderboard and return sync result for consensus
pub fn perform_sync() -> SyncResult {
    // Every 120 blocks, fetch GitHub issues and award/penalize
    let block = platform_challenge_sdk_wasm::host_functions::host_consensus_get_block_height();
    if block > 0 && block % 120 == 0 {
        crate::github_sync::fetch_and_process_issues();
    }

    rebuild_leaderboard();

    let entries = storage::get_leaderboard();
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
