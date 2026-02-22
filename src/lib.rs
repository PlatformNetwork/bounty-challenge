#![no_std]

extern crate alloc;

mod api;
mod routes;
mod scoring;
pub mod storage;
pub mod types;
mod validation;

use alloc::string::String;
use alloc::vec::Vec;
use bincode::Options;
use platform_challenge_sdk_wasm::host_functions::host_storage_get;
use platform_challenge_sdk_wasm::{Challenge, EvaluationInput, EvaluationOutput, WasmRouteRequest};

use crate::types::{BountySubmission, LeaderboardEntry};

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

        EvaluationOutput::success(score, &message)
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
        let entries: Vec<LeaderboardEntry> = host_storage_get(b"leaderboard")
            .ok()
            .and_then(|d| {
                if d.is_empty() {
                    None
                } else {
                    bincode::deserialize(&d).ok()
                }
            })
            .unwrap_or_default();

        let weights = scoring::calculate_weights_from_leaderboard(&entries);
        bincode::serialize(&weights).unwrap_or_default()
    }
}

platform_challenge_sdk_wasm::register_challenge!(BountyChallengeWasm, BountyChallengeWasm::new());
