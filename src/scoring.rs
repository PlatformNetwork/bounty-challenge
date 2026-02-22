use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::storage;
use crate::types::LeaderboardEntry;

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

pub fn calculate_net_points(valid_count: u32, invalid_count: u32, star_count: u32) -> f64 {
    let issue_points = valid_count as f64;
    let star_points = star_count as f64 * STAR_BONUS_PER_REPO;
    let penalty = if invalid_count > valid_count {
        (invalid_count - valid_count) as f64
    } else {
        0.0
    };
    (issue_points + star_points - penalty).max(0.0)
}

pub fn calculate_weights_from_leaderboard(entries: &[LeaderboardEntry]) -> Vec<WeightAssignment> {
    let mut weights: Vec<WeightAssignment> = entries
        .iter()
        .filter(|e| !e.is_penalized && e.score > 0.0)
        .map(|e| WeightAssignment {
            hotkey: e.hotkey.clone(),
            weight: e.score,
        })
        .collect();

    let total: f64 = weights.iter().map(|w| w.weight).sum();
    if total > 0.0 {
        for w in &mut weights {
            w.weight /= total;
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
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(core::cmp::Ordering::Equal)
    });

    for (i, entry) in entries.iter_mut().enumerate() {
        entry.rank = (i + 1) as u32;
    }

    storage::store_leaderboard(&entries);
}
