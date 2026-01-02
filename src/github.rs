//! GitHub API client for fetching issues

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

const GITHUB_API_BASE: &str = "https://api.github.com";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIssue {
    pub id: u64,
    pub number: u32,
    pub title: String,
    pub state: String,
    pub user: GitHubUser,
    pub labels: Vec<GitHubLabel>,
    pub created_at: DateTime<Utc>,
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

    pub fn is_closed(&self) -> bool {
        self.state == "closed"
    }

    pub fn is_valid_bounty(&self) -> bool {
        self.is_closed() && self.has_valid_label()
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
        Self {
            client: reqwest::Client::new(),
            owner: owner.into(),
            repo: repo.into(),
            token: std::env::var("GITHUB_TOKEN").ok(),
        }
    }

    pub fn with_token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }

    fn build_request(&self, url: &str) -> reqwest::RequestBuilder {
        let mut req = self
            .client
            .get(url)
            .header("User-Agent", "bounty-challenge/0.1.0")
            .header("Accept", "application/vnd.github+json");

        if let Some(token) = &self.token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        req
    }

    pub async fn get_closed_issues_with_valid(&self, since: Option<DateTime<Utc>>) -> Result<Vec<GitHubIssue>> {
        let mut all_issues = Vec::new();
        let mut page = 1;
        let per_page = 100;

        loop {
            let mut url = format!(
                "{}/repos/{}/{}/issues?state=closed&per_page={}&page={}",
                GITHUB_API_BASE, self.owner, self.repo, per_page, page
            );

            if let Some(since_date) = since {
                url.push_str(&format!("&since={}", since_date.to_rfc3339()));
            }

            debug!("Fetching issues page {}: {}", page, url);

            let response = self
                .build_request(&url)
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                warn!("GitHub API error {}: {}", status, body);
                break;
            }

            let issues: Vec<GitHubIssue> = response.json().await?;
            let count = issues.len();

            // Filter to only closed issues with "valid" label
            let valid_issues: Vec<_> = issues
                .into_iter()
                .filter(|i| i.is_valid_bounty())
                .collect();

            info!("Page {}: found {} issues, {} valid", page, count, valid_issues.len());
            all_issues.extend(valid_issues);

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

    pub async fn verify_issue_validity(&self, issue_number: u32, author: &str) -> Result<BountyVerification> {
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
