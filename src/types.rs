use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BountySubmission {
    pub hotkey: String,
    pub github_username: String,
    pub issue_numbers: Vec<u32>,
    pub repo_owner: String,
    pub repo_name: String,
    pub signature: Vec<u8>,
    pub timestamp: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub hotkey: String,
    pub github_username: String,
    pub signature: Vec<u8>,
    pub timestamp: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserRegistration {
    pub hotkey: String,
    pub github_username: String,
    pub registered_epoch: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IssueRecord {
    pub issue_number: u32,
    pub repo_owner: String,
    pub repo_name: String,
    pub author: String,
    pub is_closed: bool,
    pub has_valid_label: bool,
    pub has_invalid_label: bool,
    pub claimed_by_hotkey: Option<String>,
    pub recorded_epoch: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvalidIssueRecord {
    pub issue_number: u32,
    pub repo_owner: String,
    pub repo_name: String,
    pub github_username: String,
    pub reason: Option<String>,
    pub recorded_epoch: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UserBalance {
    pub valid_count: u32,
    pub invalid_count: u32,
    pub duplicate_count: u32,
    pub star_count: u32,
    pub is_penalized: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub rank: u32,
    pub hotkey: String,
    pub github_username: String,
    pub score: f64,
    pub valid_issues: u32,
    pub invalid_issues: u32,
    pub pending_issues: u32,
    pub star_count: u32,
    pub star_bonus: f64,
    pub net_points: f64,
    pub is_penalized: bool,
    pub last_epoch: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatsResponse {
    pub total_bounties: u64,
    pub active_miners: u64,
    pub validator_count: u64,
    pub total_issues: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    pub registered: bool,
    pub github_username: Option<String>,
    pub valid_issues_count: u32,
    pub invalid_issues_count: u32,
    pub balance: UserBalance,
    pub weight: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClaimResult {
    pub claimed: Vec<ClaimedIssue>,
    pub rejected: Vec<RejectedIssue>,
    pub total_valid: u32,
    pub score: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClaimedIssue {
    pub issue_number: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RejectedIssue {
    pub issue_number: u32,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IssueProposal {
    pub validator_id: String,
    pub issues: Vec<IssueRecord>,
    pub epoch: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IssueValidityProposal {
    pub validator_id: String,
    pub issue_number: u32,
    pub repo_owner: String,
    pub repo_name: String,
    pub is_valid: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeoutConfig {
    pub review_timeout_blocks: u64,
    pub sync_timeout_blocks: u64,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            review_timeout_blocks: 1_800,
            sync_timeout_blocks: 300,
        }
    }
}

pub use platform_challenge_sdk_wasm::{LlmMessage, LlmRequest, LlmResponse};
