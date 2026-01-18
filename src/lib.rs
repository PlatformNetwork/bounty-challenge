//! Bounty Challenge - Reward miners for valid GitHub issues
//!
//! This challenge incentivizes the discovery and reporting of valid bugs
//! in the CortexLM/fabric repository. Miners earn rewards for submitting
//! issues that are closed with the "valid" label.
//!
//! # How it works
//!
//! 1. Miners register their GitHub username with their hotkey (via OAuth)
//! 2. Miners create issues on CortexLM/fabric
//! 3. Project maintainers review and close issues with "valid" label
//! 4. Miners claim bounties by submitting issue numbers
//! 5. Challenge verifies via GitHub API and assigns weights
//!
//! # Anti-abuse measures
//!
//! - Only closed issues with "valid" label count
//! - Issue author must match registered GitHub username
//! - Each issue can only be claimed once (first reporter wins)
//! - Logarithmic scoring prevents gaming

pub mod auth;
pub mod challenge;
pub mod config;
pub mod discovery;
pub mod github;
pub mod github_oauth;
pub mod metagraph;
pub mod migrations;
pub mod pg_storage;
pub mod server;
pub mod storage;

pub use auth::{is_valid_ss58_hotkey, verify_signature};
pub use challenge::BountyChallenge;
pub use github::GitHubClient;
pub use github_oauth::GitHubDeviceAuth;
pub use metagraph::MetagraphCache;
pub use pg_storage::{calculate_weight, PgStorage};
pub use storage::BountyStorage;
