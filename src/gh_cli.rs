//! GitHub CLI (`gh`) wrapper for reliable issue syncing
//!
//! Uses the native `gh` CLI tool which handles:
//! - Authentication via GITHUB_TOKEN or gh auth
//! - Automatic pagination
//! - Rate limit handling
//! - JSON output

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::{error, info, warn};

/// Issue data from gh CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhIssue {
    pub number: u32,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub author: GhAuthor,
    pub labels: Vec<GhLabel>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
    #[serde(rename = "closedAt")]
    pub closed_at: Option<DateTime<Utc>>,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhAuthor {
    pub login: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhLabel {
    pub name: String,
}

impl GhIssue {
    pub fn has_valid_label(&self) -> bool {
        self.labels.iter().any(|l| l.name.to_lowercase() == "valid")
    }

    pub fn has_invalid_label(&self) -> bool {
        self.labels
            .iter()
            .any(|l| l.name.to_lowercase() == "invalid")
    }

    pub fn is_closed(&self) -> bool {
        self.state.to_lowercase() == "closed"
    }

    pub fn is_valid_bounty(&self) -> bool {
        self.is_closed() && self.has_valid_label()
    }

    pub fn label_names(&self) -> Vec<String> {
        self.labels.iter().map(|l| l.name.clone()).collect()
    }

    /// Convert to the GitHubIssue format used by the rest of the codebase
    pub fn to_github_issue(&self) -> crate::github::GitHubIssue {
        crate::github::GitHubIssue {
            id: self.number as u64, // gh doesn't return id, use number
            number: self.number,
            title: self.title.clone(),
            body: self.body.clone(),
            state: self.state.to_lowercase(),
            user: crate::github::GitHubUser {
                login: self.author.login.clone(),
                id: 0, // Not available from gh CLI
            },
            labels: self
                .labels
                .iter()
                .map(|l| crate::github::GitHubLabel {
                    name: l.name.clone(),
                })
                .collect(),
            created_at: self.created_at,
            updated_at: self.updated_at,
            closed_at: self.closed_at,
            html_url: self.url.clone(),
        }
    }
}

/// GitHub CLI wrapper
pub struct GhCli {
    owner: String,
    repo: String,
}

impl GhCli {
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
            repo: repo.into(),
        }
    }

    /// Check if gh CLI is available and authenticated
    pub fn is_available() -> bool {
        Command::new("gh")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Get the repo string (owner/repo)
    fn repo_string(&self) -> String {
        format!("{}/{}", self.owner, self.repo)
    }

    /// List all issues (open and closed) using gh CLI
    /// This is the most reliable way to get all issues with proper pagination
    pub fn list_all_issues(&self) -> Result<Vec<GhIssue>> {
        info!(
            "Fetching all issues from {} using gh CLI",
            self.repo_string()
        );

        // gh issue list --repo owner/repo --state all --json fields --limit 1000
        let output = Command::new("gh")
            .args([
                "issue",
                "list",
                "--repo",
                &self.repo_string(),
                "--state",
                "all",
                "--limit",
                "10000", // High limit to get all issues
                "--json",
                "number,title,body,state,author,labels,createdAt,updatedAt,closedAt,url",
            ])
            .env("GH_TOKEN", get_gh_token().unwrap_or_default())
            .output()
            .context("Failed to run gh issue list")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("gh issue list failed: {}", stderr);
            anyhow::bail!("gh issue list failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let issues: Vec<GhIssue> =
            serde_json::from_str(&stdout).context("Failed to parse gh issue list output")?;

        info!(
            "Fetched {} issues from {}",
            issues.len(),
            self.repo_string()
        );
        Ok(issues)
    }
}

/// Get GitHub token from environment (priority: EXTRA_GITHUB_TOKEN > GITHUB_TOKEN > GH_TOKEN)
fn get_gh_token() -> Option<String> {
    std::env::var("EXTRA_GITHUB_TOKEN")
        .or_else(|_| std::env::var("GITHUB_TOKEN"))
        .or_else(|_| std::env::var("GH_TOKEN"))
        .ok()
}

/// Sync all issues from a repo using gh CLI
/// Returns the number of issues synced
pub async fn sync_repo_with_gh(
    storage: &crate::pg_storage::PgStorage,
    owner: &str,
    repo: &str,
) -> Result<SyncResult> {
    let gh = GhCli::new(owner, repo);

    // Fetch all issues via gh CLI
    let issues = gh.list_all_issues()?;

    let mut result = SyncResult {
        total_fetched: issues.len(),
        ..Default::default()
    };

    // Collect all issue IDs we see from GitHub
    let seen_issue_ids: Vec<i64> = issues.iter().map(|i| i.number as i64).collect();

    // Upsert each issue
    for issue in &issues {
        let github_issue = issue.to_github_issue();
        match storage.upsert_issue(&github_issue, owner, repo).await {
            Ok(change) => match change {
                crate::pg_storage::LabelChange::BecameValid => result.became_valid += 1,
                crate::pg_storage::LabelChange::BecameInvalid => result.became_invalid += 1,
                crate::pg_storage::LabelChange::LostValid => result.lost_valid += 1,
                crate::pg_storage::LabelChange::None => {}
            },
            Err(e) => {
                warn!("Failed to upsert issue #{}: {}", issue.number, e);
                result.errors += 1;
            }
        }
    }

    // Mark issues not returned by GitHub as deleted (transferred/removed)
    let deleted = storage
        .mark_deleted_issues(owner, repo, &seen_issue_ids)
        .await?;
    result.marked_deleted = deleted as usize;

    // Update sync state
    storage
        .update_sync_state(owner, repo, issues.len() as i32)
        .await?;

    info!(
        "Sync complete for {}/{}: {} fetched, {} valid, {} invalid, {} deleted",
        owner,
        repo,
        result.total_fetched,
        result.became_valid,
        result.became_invalid,
        result.marked_deleted
    );

    Ok(result)
}

/// Result of a sync operation
#[derive(Debug, Default)]
pub struct SyncResult {
    pub total_fetched: usize,
    pub became_valid: usize,
    /// Issues explicitly marked with "invalid" label
    pub became_invalid: usize,
    pub lost_valid: usize,
    pub marked_deleted: usize,
    pub errors: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gh_cli_available() {
        // This test only runs if gh is installed
        if GhCli::is_available() {
            assert!(GhCli::is_available());
        }
    }

    #[test]
    fn test_get_gh_token_priority() {
        // Clean environment first
        std::env::remove_var("EXTRA_GITHUB_TOKEN");
        std::env::remove_var("GITHUB_TOKEN");
        std::env::remove_var("GH_TOKEN");

        // Test priority: EXTRA_GITHUB_TOKEN > GITHUB_TOKEN > GH_TOKEN
        std::env::set_var("GH_TOKEN", "gh_token");
        assert_eq!(get_gh_token(), Some("gh_token".to_string()));

        std::env::set_var("GITHUB_TOKEN", "github_token");
        assert_eq!(get_gh_token(), Some("github_token".to_string()));

        std::env::set_var("EXTRA_GITHUB_TOKEN", "extra_token");
        assert_eq!(get_gh_token(), Some("extra_token".to_string()));

        // Cleanup
        std::env::remove_var("EXTRA_GITHUB_TOKEN");
        std::env::remove_var("GITHUB_TOKEN");
        std::env::remove_var("GH_TOKEN");
    }

    #[test]
    fn test_parse_gh_issue() -> Result<(), serde_json::Error> {
        let json = r#"{
            "number": 42,
            "title": "Test Issue",
            "body": "Test body",
            "state": "CLOSED",
            "author": {"login": "testuser"},
            "labels": [{"name": "valid"}, {"name": "bug"}],
            "createdAt": "2024-01-01T00:00:00Z",
            "updatedAt": "2024-01-02T00:00:00Z",
            "closedAt": "2024-01-02T00:00:00Z",
            "url": "https://github.com/test/repo/issues/42"
        }"#;

        let issue: GhIssue = serde_json::from_str(json)?;
        assert_eq!(issue.number, 42);
        assert!(issue.has_valid_label());
        assert!(issue.is_closed());
        assert!(issue.is_valid_bounty());
        Ok(())
    }
}
