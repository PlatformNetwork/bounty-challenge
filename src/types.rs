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
    #[serde(default)]
    pub hotkey: String,
    pub github_username: String,
    #[serde(default)]
    pub signature: Vec<u8>,
    #[serde(default)]
    pub timestamp: i64,
}

/// Simplified claim request - authentication done via headers
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClaimRequest {
    pub issue_url: String,
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
    #[serde(default)]
    pub has_ide_label: bool,
    pub claimed_by_hotkey: Option<String>,
    pub recorded_epoch: u64,
    #[serde(default)]
    pub has_duplicate_label: bool,
    #[serde(default)]
    pub has_malicious_label: bool,
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
    #[serde(default)]
    pub malicious_count: u32,
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
    #[serde(default)]
    pub duplicate_issues: u32,
    #[serde(default)]
    pub malicious_issues: u32,
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
    pub hotkey: String,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IssuesStatsResponse {
    pub total: u64,
    pub open: u64,
    pub closed: u64,
    pub valid: u64,
    pub invalid: u64,
    pub pending: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IssueShort {
    pub issue_id: u32,
    pub repo_owner: String,
    pub repo_name: String,
    pub title: String,
    pub state: String,
    pub labels: Vec<String>,
    pub updated_at: String,
    pub issue_url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitHubUserDetailsResponse {
    pub github_username: String,
    pub hotkey: Option<String>,
    pub registered_at: Option<String>,
    pub total_issues: u64,
    pub valid_issues: u64,
    pub invalid_issues: u64,
    pub open_issues: u64,
    pub recent_issues: Vec<IssueShort>,
}

/// A single entry in a bulk migration: links a GitHub username to a hotkey.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MigrationEntry {
    pub hotkey: String,
    pub github_username: String,
}

/// Sudo request to bulk-register users (migration).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BulkMigrationRequest {
    pub entries: Vec<MigrationEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HotkeyDetailsResponse {
    pub hotkey: String,
    pub github_username: String,
    pub registered_at: String,
    pub valid_issues: u32,
    pub invalid_issues: u32,
    pub duplicate_issues: u32,
    pub total_points: f64,
    pub penalty_points: f64,
    pub net_points: f64,
    pub balance: UserBalance,
    pub is_penalized: bool,
    pub weight: f64,
    pub recent_issues: Vec<IssueShort>,
}

/// Metrics attached to each evaluation for multi-validator aggregation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvalMetrics {
    pub claimed_count: u32,
    pub rejected_count: u32,
    pub total_valid: u32,
    pub weight: f64,
}

/// Entry used in aggregation output leaderboard.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AggregatedMinerEntry {
    pub hotkey: String,
    pub aggregated_score: f64,
    pub validator_count: u32,
    pub total_stake: u64,
}

pub use platform_challenge_sdk_wasm::{LlmMessage, LlmRequest, LlmResponse};

/// Result of a sync operation for consensus
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncResult {
    pub leaderboard_hash: [u8; 32],
    pub total_users: u32,
    pub total_valid_issues: u32,
    pub total_invalid_issues: u32,
    pub total_pending_issues: u32,
    pub sync_timestamp: i64,
}
