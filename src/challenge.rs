//! Bounty Challenge implementation

use std::sync::Arc;

use async_trait::async_trait;

use platform_challenge_sdk::server::{
    ConfigLimits, ConfigResponse, EvaluationRequest, EvaluationResponse, ServerChallenge,
    ValidationRequest, ValidationResponse,
};
use platform_challenge_sdk::ChallengeError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{info, warn};

use crate::github::GitHubClient;
use crate::pg_storage::PgStorage;

const CHALLENGE_ID: &str = "bounty-challenge";
const CHALLENGE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Deserialize)]
pub struct ClaimSubmission {
    pub github_username: String,
    pub issue_numbers: Vec<u32>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterSubmission {
    pub github_username: String,
}

#[derive(Debug, Serialize)]
pub struct ClaimResult {
    pub claimed: Vec<ClaimedIssue>,
    pub rejected: Vec<RejectedIssue>,
    pub total_valid: u32,
    pub score: f64,
}

#[derive(Debug, Serialize)]
pub struct ClaimedIssue {
    pub issue_number: u32,
    pub issue_url: String,
}

#[derive(Debug, Serialize)]
pub struct RejectedIssue {
    pub issue_number: u32,
    pub reason: String,
}

pub struct BountyChallenge {
    github: GitHubClient,
    storage: Arc<PgStorage>,
}

impl BountyChallenge {
    pub fn new_with_storage(storage: Arc<PgStorage>) -> Self {
        Self {
            github: GitHubClient::new("PlatformNetwork", "bounty-challenge"),
            storage,
        }
    }

    async fn handle_register(
        &self,
        participant_id: &str,
        data: RegisterSubmission,
    ) -> Result<EvaluationResponse, ChallengeError> {
        self.storage
            .register_user(&data.github_username, participant_id)
            .await
            .map_err(|e| ChallengeError::Internal(e.to_string()))?;

        info!(
            "Registered miner {} with GitHub user {}",
            participant_id, data.github_username
        );

        Ok(EvaluationResponse::success(
            participant_id,
            1.0,
            json!({
                "registered": true,
                "github_username": data.github_username
            }),
        ))
    }

    async fn handle_claim(
        &self,
        request_id: &str,
        participant_id: &str,
        data: ClaimSubmission,
    ) -> Result<EvaluationResponse, ChallengeError> {
        let mut claimed = Vec::new();
        let mut rejected = Vec::new();

        // Get repo info (default to PlatformNetwork/bounty-challenge)
        let repo_owner = "PlatformNetwork";
        let repo_name = "bounty-challenge";

        for issue_number in &data.issue_numbers {
            // Check if already recorded
            if self
                .storage
                .is_issue_recorded(repo_owner, repo_name, *issue_number as i64)
                .await
                .map_err(|e| ChallengeError::Internal(e.to_string()))?
            {
                rejected.push(RejectedIssue {
                    issue_number: *issue_number,
                    reason: "Issue already claimed".to_string(),
                });
                continue;
            }

            // Verify with GitHub API
            match self
                .github
                .verify_issue_validity(*issue_number, &data.github_username)
                .await
            {
                Ok(verification) => {
                    if !verification.is_valid_bounty {
                        let reason = if !verification.is_author_match {
                            format!(
                                "Author mismatch: expected {}, got {}",
                                data.github_username, verification.actual_author
                            )
                        } else if !verification.is_closed {
                            "Issue not closed".to_string()
                        } else {
                            "Issue missing 'valid' label".to_string()
                        };
                        rejected.push(RejectedIssue {
                            issue_number: *issue_number,
                            reason,
                        });
                        continue;
                    }

                    // Record the resolved issue
                    self.storage
                        .record_resolved_issue(
                            *issue_number as i64,
                            repo_owner,
                            repo_name,
                            &data.github_username,
                            &verification.issue_url,
                            Some(&format!("Issue #{}", issue_number)),
                            chrono::Utc::now(),
                        )
                        .await
                        .map_err(|e| ChallengeError::Internal(e.to_string()))?;

                    claimed.push(ClaimedIssue {
                        issue_number: *issue_number,
                        issue_url: verification.issue_url,
                    });
                }
                Err(e) => {
                    warn!("Failed to verify issue #{}: {}", issue_number, e);
                    rejected.push(RejectedIssue {
                        issue_number: *issue_number,
                        reason: format!("Verification failed: {}", e),
                    });
                }
            }
        }

        // Calculate score based on weight from PgStorage
        let weight = self
            .storage
            .calculate_user_weight(participant_id)
            .await
            .unwrap_or(0.0);

        let user_balance = self
            .storage
            .get_user_balance(participant_id)
            .await
            .ok()
            .flatten();
        let total_valid = user_balance.map(|b| b.valid_count as u32).unwrap_or(0);

        let result = ClaimResult {
            claimed,
            rejected,
            total_valid,
            score: weight,
        };

        let result_json = serde_json::to_value(&result).unwrap_or_else(|e| {
            warn!("Failed to serialize claim result: {}", e);
            serde_json::json!({
                "claimed": [],
                "rejected": [],
                "total_valid": 0,
                "score": weight,
                "error": "Failed to serialize result"
            })
        });

        Ok(EvaluationResponse::success(request_id, weight, result_json))
    }

    pub async fn get_leaderboard(&self) -> Result<Vec<serde_json::Value>, ChallengeError> {
        // Get extended leaderboard (includes users with pending issues and detailed scoring)
        let entries = self
            .storage
            .get_extended_leaderboard(100)
            .await
            .map_err(|e| ChallengeError::Internal(e.to_string()))?;

        let leaderboard: Vec<_> = entries
            .into_iter()
            .map(|e| {
                json!({
                    "hotkey": e.hotkey,
                    "github_username": e.github_username,
                    "score": e.score,
                    "valid_issues": e.valid_issues,
                    "invalid_issues": e.invalid_issues,
                    "pending_issues": e.pending_issues,
                    "total_points": e.total_points,
                    "penalty_points": e.penalty_points,
                    "net_points": e.net_points,
                    "star_count": e.star_count,
                    "starred_repos": e.starred_repos,
                    "star_bonus": e.star_bonus,
                    "is_penalized": e.is_penalized,
                    "last_activity": e.last_activity,
                })
            })
            .collect();

        Ok(leaderboard)
    }
}

#[async_trait]
impl ServerChallenge for BountyChallenge {
    fn challenge_id(&self) -> &str {
        CHALLENGE_ID
    }

    fn name(&self) -> &str {
        "Bounty Challenge"
    }

    fn version(&self) -> &str {
        CHALLENGE_VERSION
    }

    async fn evaluate(
        &self,
        request: EvaluationRequest,
    ) -> Result<EvaluationResponse, ChallengeError> {
        let action = request
            .data
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("claim");

        info!(
            "Evaluating {} for participant {}",
            action, request.participant_id
        );

        match action {
            "register" => {
                let data: RegisterSubmission = serde_json::from_value(request.data.clone())
                    .map_err(|e| ChallengeError::Validation(e.to_string()))?;
                self.handle_register(&request.participant_id, data).await
            }
            "claim" => {
                let data: ClaimSubmission = serde_json::from_value(request.data.clone())
                    .map_err(|e| ChallengeError::Validation(e.to_string()))?;
                self.handle_claim(&request.request_id, &request.participant_id, data)
                    .await
            }
            "leaderboard" => {
                let leaderboard = self.get_leaderboard().await?;
                Ok(EvaluationResponse::success(
                    &request.request_id,
                    0.0,
                    json!({ "leaderboard": leaderboard }),
                ))
            }
            _ => Err(ChallengeError::Validation(format!(
                "Unknown action: {}",
                action
            ))),
        }
    }

    async fn validate(
        &self,
        request: ValidationRequest,
    ) -> Result<ValidationResponse, ChallengeError> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check required fields
        if request.data.get("github_username").is_none() {
            errors.push("Missing github_username".to_string());
        }

        let action = request
            .data
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("claim");

        if action == "claim" && request.data.get("issue_numbers").is_none() {
            errors.push("Missing issue_numbers for claim action".to_string());
        }

        if let Some(issues) = request.data.get("issue_numbers") {
            if let Some(arr) = issues.as_array() {
                if arr.is_empty() {
                    warnings.push("Empty issue_numbers array".to_string());
                }
            }
        }

        Ok(ValidationResponse {
            valid: errors.is_empty(),
            errors,
            warnings,
        })
    }

    fn config(&self) -> ConfigResponse {
        ConfigResponse {
            challenge_id: CHALLENGE_ID.to_string(),
            name: "Bounty Challenge".to_string(),
            version: CHALLENGE_VERSION.to_string(),
            config_schema: Some(json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["register", "claim", "leaderboard"]
                    },
                    "github_username": {
                        "type": "string",
                        "description": "GitHub username to link with miner"
                    },
                    "issue_numbers": {
                        "type": "array",
                        "items": { "type": "integer" },
                        "description": "Issue numbers to claim bounty for"
                    }
                },
                "required": ["github_username"]
            })),
            features: vec!["github-verification".to_string(), "anti-abuse".to_string()],
            limits: ConfigLimits {
                max_submission_size: Some(10 * 1024),
                max_evaluation_time: Some(60),
                max_cost: None,
            },
        }
    }
}
