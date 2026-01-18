//! Bounty Challenge Bridge API Client
//!
//! Routes requests through platform-server bridge API.
//! All requests go through /api/v1/bridge/bounty-challenge/...

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const CHALLENGE_ID: &str = "bounty-challenge";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Registration request sent to the bridge
#[derive(Debug, Serialize)]
pub struct RegisterRequest {
    pub hotkey: String,
    pub github_username: String,
    pub signature: String,
    pub timestamp: i64,
}

/// Registration response from the bridge
#[derive(Debug, Deserialize)]
pub struct RegisterResponse {
    pub success: bool,
    pub message: Option<String>,
    pub error: Option<String>,
}

/// Leaderboard entry
#[derive(Debug, Deserialize)]
pub struct LeaderboardEntry {
    pub github_username: String,
    pub hotkey: String,
    pub issues_resolved_24h: i32,
    pub weight: f64,
}

/// Status response
#[derive(Debug, Deserialize)]
pub struct StatusResponse {
    pub registered: bool,
    pub github_username: Option<String>,
    pub issues_resolved_24h: Option<i32>,
    pub weight: Option<f64>,
    pub recent_issues: Option<Vec<RecentIssue>>,
}

#[derive(Debug, Deserialize)]
pub struct RecentIssue {
    pub issue_id: i64,
    pub repo: String,
    pub title: Option<String>,
    pub resolved_at: String,
}

/// Bounty Challenge Bridge API client
pub struct BountyClient {
    client: Client,
    base_url: String,
}

impl BountyClient {
    /// Create a new client pointing to platform server
    pub fn new(platform_url: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(DEFAULT_TIMEOUT)
                .build()
                .expect("Failed to create HTTP client"),
            base_url: platform_url.trim_end_matches('/').to_string(),
        }
    }

    /// Create client with custom timeout
    pub fn with_timeout(platform_url: &str, timeout: Duration) -> Self {
        Self {
            client: Client::builder()
                .timeout(timeout)
                .build()
                .expect("Failed to create HTTP client"),
            base_url: platform_url.trim_end_matches('/').to_string(),
        }
    }

    /// Get the bridge URL for bounty-challenge endpoints
    fn bridge_url(&self, path: &str) -> String {
        let path = path.trim_start_matches('/');
        format!("{}/api/v1/bridge/{}/{}", self.base_url, CHALLENGE_ID, path)
    }

    /// Register a GitHub username with a hotkey
    pub async fn register(&self, request: &RegisterRequest) -> Result<RegisterResponse> {
        let url = self.bridge_url("register");
        let resp = self.client.post(&url).json(request).send().await?;

        let status = resp.status();
        if status.is_success() {
            Ok(resp.json().await?)
        } else {
            let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".into());
            Err(anyhow!("Registration failed ({}): {}", status, error_text))
        }
    }

    /// Get the leaderboard
    pub async fn get_leaderboard(&self, limit: usize) -> Result<Vec<LeaderboardEntry>> {
        let url = self.bridge_url(&format!("leaderboard?limit={}", limit));
        let resp = self.client.get(&url).send().await?;

        let status = resp.status();
        if status.is_success() {
            let data: serde_json::Value = resp.json().await?;
            if let Some(entries) = data.get("leaderboard").and_then(|v| v.as_array()) {
                let leaderboard: Vec<LeaderboardEntry> = entries
                    .iter()
                    .filter_map(|v| serde_json::from_value(v.clone()).ok())
                    .collect();
                Ok(leaderboard)
            } else {
                Ok(vec![])
            }
        } else {
            let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".into());
            Err(anyhow!("Failed to fetch leaderboard ({}): {}", status, error_text))
        }
    }

    /// Get status for a specific hotkey
    pub async fn get_status(&self, hotkey: &str) -> Result<StatusResponse> {
        let url = self.bridge_url(&format!("status/{}", hotkey));
        let resp = self.client.get(&url).send().await?;

        let status = resp.status();
        if status.is_success() {
            Ok(resp.json().await?)
        } else {
            let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".into());
            Err(anyhow!("Failed to fetch status ({}): {}", status, error_text))
        }
    }

    /// Get challenge stats
    pub async fn get_stats(&self) -> Result<serde_json::Value> {
        let url = self.bridge_url("stats");
        let resp = self.client.get(&url).send().await?;

        let status = resp.status();
        if status.is_success() {
            Ok(resp.json().await?)
        } else {
            let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".into());
            Err(anyhow!("Failed to fetch stats ({}): {}", status, error_text))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_new() {
        let client = BountyClient::new("https://api.example.com");
        assert_eq!(client.base_url, "https://api.example.com");
    }

    #[test]
    fn test_client_strips_trailing_slash() {
        let client = BountyClient::new("https://api.example.com/");
        assert_eq!(client.base_url, "https://api.example.com");
    }

    #[test]
    fn test_bridge_url() {
        let client = BountyClient::new("https://api.example.com");
        let url = client.bridge_url("register");
        assert_eq!(url, "https://api.example.com/api/v1/bridge/bounty-challenge/register");
    }

    #[test]
    fn test_bridge_url_with_query() {
        let client = BountyClient::new("https://api.example.com");
        let url = client.bridge_url("leaderboard?limit=10");
        assert!(url.contains("leaderboard?limit=10"));
    }
}
