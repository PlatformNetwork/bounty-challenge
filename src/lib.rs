//! Bounty Challenge - Reward miners for valid GitHub issues
//!
//! This challenge incentivizes the discovery and reporting of valid bugs
//! in the PlatformNetwork/bounty-challenge repository. Miners earn rewards for submitting
//! issues that are closed with the "valid" label.
//!
//! # How it works
//!
//! 1. Miners register their GitHub username with their hotkey (via sr25519 signature)
//! 2. Miners create issues on PlatformNetwork/bounty-challenge
//! 3. Project maintainers review and close issues with "valid" label
//! 4. Validators sync issues from GitHub and auto-credit rewards
//! 5. Weight calculated: 1 point per valid issue + 0.25 per starred repo
//!
//! # Anti-abuse measures
//!
//! - Only closed issues with "valid" label count
//! - Issue author must match registered GitHub username
//! - Each issue can only be claimed once (first reporter wins)
//! - Linear scoring with 50-point cap prevents gaming

pub mod auth;
pub mod challenge;
pub mod config;
pub mod gh_cli;
pub mod github;
pub mod github_oauth;
pub mod metagraph;
pub mod pg_storage;
pub mod server;

pub use auth::{is_valid_ss58_hotkey, verify_signature};
pub use challenge::BountyChallenge;
pub use gh_cli::{sync_repo_with_gh, GhCli, GhIssue, SyncResult as GhSyncResult};
pub use github::{GitHubClient, RateLimitInfo};
pub use metagraph::MetagraphCache;
pub use pg_storage::{
    calculate_weight_from_points, PgStorage, MAX_POINTS_FOR_FULL_WEIGHT, WEIGHT_PER_POINT,
};
