//! Automatic Bounty Discovery
//!
//! Validators periodically scan GitHub for valid issues and auto-credit bounties.

use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use crate::github::GitHubClient;
use crate::storage::{BountyStorage, ValidatedBounty};

const SCAN_INTERVAL: Duration = Duration::from_secs(300); // 5 minutes

pub struct BountyDiscovery {
    github: GitHubClient,
    storage: Arc<BountyStorage>,
    last_scan: Option<DateTime<Utc>>,
}

impl BountyDiscovery {
    pub fn new(owner: &str, repo: &str, storage: Arc<BountyStorage>) -> Self {
        Self {
            github: GitHubClient::new(owner, repo),
            storage,
            last_scan: None,
        }
    }

    /// Run discovery loop (for validators)
    pub async fn run_loop(mut self) {
        info!("Starting bounty discovery loop");
        let mut ticker = interval(SCAN_INTERVAL);

        loop {
            ticker.tick().await;
            
            if let Err(e) = self.scan_and_credit().await {
                error!("Discovery scan failed: {}", e);
            }
        }
    }

    /// Single scan and credit run
    pub async fn scan_and_credit(&mut self) -> anyhow::Result<ScanResult> {
        info!("Scanning for new valid issues...");

        let since = self.last_scan;
        let issues = self.github.get_closed_issues_with_valid(since).await?;

        let mut result = ScanResult::default();
        result.total_found = issues.len();

        for issue in issues {
            // Check if already credited
            if self.storage.is_issue_claimed(issue.number)? {
                debug!("Issue #{} already credited", issue.number);
                result.already_claimed += 1;
                continue;
            }

            // Find miner with matching GitHub username
            let github_user = issue.user.login.to_lowercase();
            
            match self.storage.get_hotkey_by_github(&github_user)? {
                Some(hotkey) => {
                    // Auto-credit the bounty
                    let bounty = ValidatedBounty {
                        issue_number: issue.number,
                        github_username: issue.user.login.clone(),
                        miner_hotkey: hotkey.clone(),
                        validated_at: Utc::now(),
                        issue_url: issue.html_url.clone(),
                    };

                    self.storage.record_bounty(&bounty)?;
                    
                    info!(
                        "Auto-credited issue #{} to {} ({})",
                        issue.number, &hotkey[..16], issue.user.login
                    );
                    result.newly_credited += 1;
                }
                None => {
                    debug!(
                        "Issue #{} by @{} - no registered miner found",
                        issue.number, issue.user.login
                    );
                    result.no_miner += 1;
                }
            }
        }

        self.last_scan = Some(Utc::now());

        info!(
            "Scan complete: {} found, {} credited, {} already claimed, {} no miner",
            result.total_found, result.newly_credited, result.already_claimed, result.no_miner
        );

        Ok(result)
    }

    /// Manual trigger for single scan
    pub async fn scan_once(&mut self) -> anyhow::Result<ScanResult> {
        self.scan_and_credit().await
    }
}

#[derive(Debug, Default)]
pub struct ScanResult {
    pub total_found: usize,
    pub newly_credited: usize,
    pub already_claimed: usize,
    pub no_miner: usize,
}
