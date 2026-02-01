//! GitHub API client for fetching issues
//!
//! Supports authentication via environment variables:
//! - EXTRA_GITHUB_TOKEN (priority, passed from platform-server)
//! - GITHUB_TOKEN (fallback)

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

const GITHUB_API_BASE: &str = "https://api.github.com";

/// Minimum remaining requests before we start throttling
const RATE_LIMIT_THRESHOLD: u32 = 100;

/// Get GitHub token from environment (EXTRA_GITHUB_TOKEN takes priority)
fn get_github_token() -> Option<String> {
    std::env::var("EXTRA_GITHUB_TOKEN")
        .or_else(|_| std::env::var("GITHUB_TOKEN"))
        .ok()
}

/// Rate limit information from GitHub API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    pub limit: u32,
    pub remaining: u32,
    pub reset: i64,
    pub used: u32,
}

impl RateLimitInfo {
    /// Check if we're running low on API calls
    pub fn is_low(&self) -> bool {
        self.remaining < RATE_LIMIT_THRESHOLD
    }

    /// Seconds until rate limit resets
    pub fn seconds_until_reset(&self) -> i64 {
        let now = Utc::now().timestamp();
        (self.reset - now).max(0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIssue {
    pub id: u64,
    pub number: u32,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub user: GitHubUser,
    pub labels: Vec<GitHubLabel>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub html_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubLabel {
    pub name: String,
}

impl GitHubIssue {
    pub fn has_valid_label(&self) -> bool {
        self.labels.iter().any(|l| l.name.to_lowercase() == "valid")
    }

    pub fn has_invalid_label(&self) -> bool {
        self.labels
            .iter()
            .any(|l| l.name.to_lowercase() == "invalid")
    }

    pub fn is_closed(&self) -> bool {
        self.state == "closed"
    }

    pub fn is_valid_bounty(&self) -> bool {
        self.is_closed() && self.has_valid_label()
    }

    pub fn label_names(&self) -> Vec<String> {
        self.labels.iter().map(|l| l.name.clone()).collect()
    }
}

pub struct GitHubClient {
    client: reqwest::Client,
    owner: String,
    repo: String,
    token: Option<String>,
}

impl GitHubClient {
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        let token = get_github_token();
        if token.is_some() {
            info!("GitHub client initialized with authentication token");
        } else {
            warn!(
                "GitHub client initialized WITHOUT token - rate limits will be very low (60/hour)"
            );
        }
        Self {
            client: reqwest::Client::new(),
            owner: owner.into(),
            repo: repo.into(),
            token,
        }
    }

    /// Check if authenticated
    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }

    fn build_request(&self, url: &str) -> reqwest::RequestBuilder {
        let mut req = self
            .client
            .get(url)
            .header("User-Agent", "bounty-challenge/0.1.0")
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28");

        if let Some(token) = &self.token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        req
    }

    /// Check GitHub API rate limit status (heartbeat)
    pub async fn check_rate_limit(&self) -> Result<RateLimitInfo> {
        let url = format!("{}/rate_limit", GITHUB_API_BASE);
        let response = self.build_request(&url).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to check rate limit: {}", response.status());
        }

        #[derive(Deserialize)]
        struct RateLimitResponse {
            rate: RateLimitCore,
        }
        #[derive(Deserialize)]
        struct RateLimitCore {
            limit: u32,
            remaining: u32,
            reset: i64,
            used: u32,
        }

        let data: RateLimitResponse = response.json().await?;
        Ok(RateLimitInfo {
            limit: data.rate.limit,
            remaining: data.rate.remaining,
            reset: data.rate.reset,
            used: data.rate.used,
        })
    }

    /// Fetch all issues (open and closed)
    /// Always fetches ALL issues to ensure nothing is missed
    pub async fn get_all_issues(&self) -> Result<Vec<GitHubIssue>> {
        let mut all_issues = Vec::new();
        let mut page = 1;
        let per_page = 100;

        loop {
            let url = format!(
                "{}/repos/{}/{}/issues?state=all&per_page={}&page={}&sort=updated&direction=desc",
                GITHUB_API_BASE, self.owner, self.repo, per_page, page
            );

            debug!("Fetching all issues page {}: {}", page, url);

            let response = self.build_request(&url).send().await?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                warn!("GitHub API error {}: {}", status, body);
                break;
            }

            let issues: Vec<GitHubIssue> = response.json().await?;
            let count = issues.len();

            info!("Page {}: fetched {} issues", page, count);
            all_issues.extend(issues);

            if count < per_page {
                break;
            }
            page += 1;

            // Rate limit protection
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(all_issues)
    }

    pub async fn get_issue(&self, number: u32) -> Result<GitHubIssue> {
        let url = format!(
            "{}/repos/{}/{}/issues/{}",
            GITHUB_API_BASE, self.owner, self.repo, number
        );

        let response = self.build_request(&url).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch issue #{}: {}", number, response.status());
        }

        Ok(response.json().await?)
    }

    pub async fn verify_issue_validity(
        &self,
        issue_number: u32,
        author: &str,
    ) -> Result<BountyVerification> {
        let issue = self.get_issue(issue_number).await?;

        let is_author_match = issue.user.login.to_lowercase() == author.to_lowercase();
        let is_valid = issue.is_valid_bounty();

        Ok(BountyVerification {
            issue_number,
            claimed_author: author.to_string(),
            actual_author: issue.user.login.clone(),
            is_author_match,
            is_closed: issue.is_closed(),
            has_valid_label: issue.has_valid_label(),
            is_valid_bounty: is_valid && is_author_match,
            issue_url: issue.html_url,
            closed_at: issue.closed_at,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BountyVerification {
    pub issue_number: u32,
    pub claimed_author: String,
    pub actual_author: String,
    pub is_author_match: bool,
    pub is_closed: bool,
    pub has_valid_label: bool,
    pub is_valid_bounty: bool,
    pub issue_url: String,
    pub closed_at: Option<DateTime<Utc>>,
}

/// Fetch all stargazers for a repository
pub async fn get_stargazers(owner: &str, repo: &str) -> Result<Vec<String>> {
    let client = reqwest::Client::new();
    let token = get_github_token();

    let mut all_stargazers = Vec::new();
    let mut page = 1;
    let per_page = 100;

    loop {
        let url = format!(
            "{}/repos/{}/{}/stargazers?per_page={}&page={}",
            GITHUB_API_BASE, owner, repo, per_page, page
        );

        let mut req = client
            .get(&url)
            .header("User-Agent", "bounty-challenge/0.1.0")
            .header("Accept", "application/vnd.github+json");

        if let Some(ref t) = token {
            req = req.header("Authorization", format!("Bearer {}", t));
        }

        let response = req.send().await?;

        if !response.status().is_success() {
            if response.status().as_u16() == 404 {
                debug!("Repo {}/{} not found or no access", owner, repo);
                return Ok(vec![]);
            }
            warn!(
                "Failed to fetch stargazers for {}/{}: {}",
                owner,
                repo,
                response.status()
            );
            break;
        }

        let stargazers: Vec<GitHubUser> = response.json().await?;

        if stargazers.is_empty() {
            break;
        }

        let count = stargazers.len();
        for user in stargazers {
            all_stargazers.push(user.login);
        }

        if count < per_page {
            break;
        }

        page += 1;

        // Rate limit protection
        if page > 10 {
            info!("Stopping at page 10 for {}/{} stargazers", owner, repo);
            break;
        }
    }

    debug!(
        "Found {} stargazers for {}/{}",
        all_stargazers.len(),
        owner,
        repo
    );
    Ok(all_stargazers)
}
