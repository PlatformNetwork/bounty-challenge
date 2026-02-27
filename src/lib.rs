#![no_std]

extern crate alloc;

mod api;
mod github_sync;
mod routes;
mod scoring;
pub mod ss58;
pub mod storage;
pub mod types;
mod validation;

use alloc::string::String;
use alloc::vec::Vec;
use bincode::Options;
use platform_challenge_sdk_wasm::{
    AggregationInput, AggregationOutput, Challenge, EvaluationInput, EvaluationOutput,
    WasmRouteRequest, WeightEntry,
};

use crate::types::BountySubmission;

const MAX_SUBMISSION_SIZE: u64 = 4 * 1024 * 1024;
const MAX_ROUTE_REQUEST_SIZE: u64 = 1024 * 1024;

fn bincode_options_submission() -> impl Options {
    bincode::DefaultOptions::new()
        .with_limit(MAX_SUBMISSION_SIZE)
        .with_fixint_encoding()
        .allow_trailing_bytes()
}

fn bincode_options_route_request() -> impl Options {
    bincode::DefaultOptions::new()
        .with_limit(MAX_ROUTE_REQUEST_SIZE)
        .with_fixint_encoding()
        .allow_trailing_bytes()
}

pub struct BountyChallengeWasm;

impl Default for BountyChallengeWasm {
    fn default() -> Self {
        Self
    }
}

impl BountyChallengeWasm {
    pub const fn new() -> Self {
        Self
    }
}

impl Challenge for BountyChallengeWasm {
    fn name(&self) -> &'static str {
        "bounty-challenge"
    }

    fn version(&self) -> &'static str {
        "2.0.0"
    }

    fn evaluate(&self, input: EvaluationInput) -> EvaluationOutput {
        let submission: BountySubmission =
            match bincode_options_submission().deserialize(&input.agent_data) {
                Ok(s) => s,
                Err(_) => return EvaluationOutput::failure("failed to deserialize submission"),
            };

        if submission.hotkey.is_empty() {
            return EvaluationOutput::failure("missing hotkey");
        }

        if submission.github_username.is_empty() {
            return EvaluationOutput::failure("missing github_username");
        }

        if submission.issue_numbers.is_empty() {
            return EvaluationOutput::failure("no issues to claim");
        }

        if submission.signature.is_empty() {
            return EvaluationOutput::failure("missing signature");
        }

        let reg = match storage::get_user_by_hotkey(&submission.hotkey) {
            Some(r) => r,
            None => return EvaluationOutput::failure("hotkey not registered"),
        };

        if reg.github_username.to_lowercase() != submission.github_username.to_lowercase() {
            return EvaluationOutput::failure("github username mismatch with registration");
        }

        storage::ensure_hotkey_tracked(&submission.hotkey);

        let synced_issues = storage::get_synced_issues();
        let result = validation::process_claims(&submission, &synced_issues);

        if !result.claimed.is_empty() {
            scoring::rebuild_leaderboard();
        }

        let score = (result.score * 10_000.0) as i64;

        let mut message = String::from("claimed=");
        let claimed_count = result.claimed.len();
        let rejected_count = result.rejected.len();
        let _ = core::fmt::Write::write_fmt(
            &mut message,
            format_args!(
                "{} rejected={} total_valid={} weight={:.4}",
                claimed_count, rejected_count, result.total_valid, result.score
            ),
        );

        let metrics = bincode::serialize(&types::EvalMetrics {
            claimed_count: claimed_count as u32,
            rejected_count: rejected_count as u32,
            total_valid: result.total_valid,
            weight: result.score,
        })
        .unwrap_or_default();

        EvaluationOutput::success(score, &message).with_metrics(metrics)
    }

    fn validate(&self, input: EvaluationInput) -> bool {
        let submission: BountySubmission =
            match bincode_options_submission().deserialize(&input.agent_data) {
                Ok(s) => s,
                Err(_) => return false,
            };

        validation::validate_submission(&submission)
    }

    fn routes(&self) -> Vec<u8> {
        let defs = routes::get_route_definitions();
        bincode::serialize(&defs).unwrap_or_default()
    }

    fn handle_route(&self, request_data: &[u8]) -> Vec<u8> {
        let request: WasmRouteRequest =
            match bincode_options_route_request().deserialize(request_data) {
                Ok(r) => r,
                Err(_) => return Vec::new(),
            };
        let response = routes::handle_route_request(&request);
        bincode::serialize(&response).unwrap_or_default()
    }

    fn get_weights(&self) -> Vec<u8> {
        // Compute weights deterministically from P2P-consensus storage.
        // We recount balances in-memory from the committed issues (not from
        // stored balances which may differ between validators due to write lag).
        // This ensures all validators with the same committed issues produce
        // identical weights, which is critical for vTrust convergence.
        let weights = scoring::compute_weights_from_issues();
        bincode::serialize(&weights).unwrap_or_default()
    }

    fn sync(&self) -> Vec<u8> {
        let result = scoring::perform_sync();
        bincode::serialize(&result).unwrap_or_default()
    }

    fn aggregate(&self, input: &[u8]) -> Vec<u8> {
        let agg_input: AggregationInput = match bincode::deserialize(input) {
            Ok(i) => i,
            Err(_) => return Vec::new(),
        };

        if agg_input.evaluations.is_empty() {
            return Vec::new();
        }

        use alloc::collections::BTreeMap;

        // Group evaluations by miner hotkey.
        // Each entry: (sum_of_weighted_scores, total_stake, validator_count)
        let mut miner_scores: BTreeMap<String, (f64, u64, u32)> = BTreeMap::new();

        for eval in &agg_input.evaluations {
            let entry = miner_scores
                .entry(eval.miner_hotkey.clone())
                .or_insert((0.0, 0, 0));
            let stake = if eval.validator_stake == 0 {
                1u64
            } else {
                eval.validator_stake
            };
            entry.0 += eval.score * stake as f64;
            entry.1 += stake;
            entry.2 += 1;
        }

        // Build leaderboard entries from stake-weighted scores
        let mut entries: Vec<types::LeaderboardEntry> = miner_scores
            .iter()
            .map(|(hotkey, (weighted_sum, total_stake, _vcount))| {
                let avg_score = if *total_stake > 0 {
                    weighted_sum / *total_stake as f64
                } else {
                    0.0
                };
                // Convert score back to net_points (score was score_f64_scaled * 10000)
                let net_points = avg_score / 10_000.0;
                types::LeaderboardEntry {
                    rank: 0,
                    hotkey: hotkey.clone(),
                    github_username: String::new(),
                    score: avg_score,
                    valid_issues: 0,
                    invalid_issues: 0,
                    pending_issues: 0,
                    star_count: 0,
                    star_bonus: 0.0,
                    net_points,
                    is_penalized: false,
                    last_epoch: agg_input.epoch,
                    duplicate_issues: 0,
                    malicious_issues: 0,
                }
            })
            .filter(|e| e.net_points > 0.0)
            .collect();

        // Sort by net_points descending
        entries.sort_by(|a, b| {
            b.net_points
                .partial_cmp(&a.net_points)
                .unwrap_or(core::cmp::Ordering::Equal)
        });
        for (i, entry) in entries.iter_mut().enumerate() {
            entry.rank = (i + 1) as u32;
        }

        // Compute weights from leaderboard
        let weight_assignments = scoring::calculate_weights_from_leaderboard(&entries);

        // Convert to WeightEntry format (need UID mapping)
        // For now, use sequential UIDs since the on-chain mapping happens
        // at the validator level when submitting weights.
        let weights: Vec<WeightEntry> = weight_assignments
            .iter()
            .enumerate()
            .map(|(i, wa)| WeightEntry {
                uid: i as u16,
                weight: (wa.weight * 65535.0) as u16,
            })
            .collect();

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

        let output = AggregationOutput {
            leaderboard: leaderboard_data,
            weights,
            leaderboard_hash,
        };

        bincode::serialize(&output).unwrap_or_default()
    }
}

platform_challenge_sdk_wasm::register_challenge!(BountyChallengeWasm, BountyChallengeWasm::new());
