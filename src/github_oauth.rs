//! GitHub Device Flow Authentication
//!
//! Uses GitHub's Device Flow for CLI-friendly OAuth without callbacks.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info};

const GITHUB_DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
const GITHUB_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const GITHUB_USER_URL: &str = "https://api.github.com/user";

#[derive(Debug, Clone)]
pub struct GitHubDeviceAuth {
    client_id: String,
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[allow(dead_code)]
enum TokenResponse {
    Success {
        access_token: String,
        token_type: String,
        scope: String,
    },
    Pending {
        error: String,
        error_description: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub id: u64,
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
}

impl GitHubDeviceAuth {
    pub fn new(client_id: String) -> Self {
        Self {
            client_id,
            client: reqwest::Client::new(),
        }
    }

    pub fn from_env() -> Result<Self> {
        let config = crate::config::Config::load()?;
        Ok(Self::new(config.github_client_id()))
    }

    /// Step 1: Request device code
    pub async fn request_device_code(&self) -> Result<DeviceCodeResponse> {
        debug!("Requesting device code from GitHub");

        let response = self
            .client
            .post(GITHUB_DEVICE_CODE_URL)
            .header("Accept", "application/json")
            .form(&[
                ("client_id", &self.client_id),
                ("scope", &"read:user".to_string()),
            ])
            .send()
            .await
            .context("Failed to request device code")?;

        if !response.status().is_success() {
            let text = response.text().await?;
            anyhow::bail!("GitHub device code request failed: {}", text);
        }

        let device_code: DeviceCodeResponse = response
            .json()
            .await
            .context("Failed to parse device code response")?;

        Ok(device_code)
    }

    /// Step 2: Poll for access token (blocks until user authorizes or timeout)
    pub async fn poll_for_token(&self, device_code: &DeviceCodeResponse) -> Result<String> {
        let interval = Duration::from_secs(device_code.interval.max(5));
        let deadline = std::time::Instant::now() + Duration::from_secs(device_code.expires_in);

        loop {
            if std::time::Instant::now() > deadline {
                anyhow::bail!("Authorization timed out. Please try again.");
            }

            tokio::time::sleep(interval).await;

            let response = self
                .client
                .post(GITHUB_TOKEN_URL)
                .header("Accept", "application/json")
                .form(&[
                    ("client_id", &self.client_id),
                    ("device_code", &device_code.device_code),
                    (
                        "grant_type",
                        &"urn:ietf:params:oauth:grant-type:device_code".to_string(),
                    ),
                ])
                .send()
                .await?;

            let token_response: TokenResponse = response.json().await?;

            match token_response {
                TokenResponse::Success { access_token, .. } => {
                    return Ok(access_token);
                }
                TokenResponse::Pending { error, .. } => match error.as_str() {
                    "authorization_pending" => {
                        debug!("Waiting for user authorization...");
                        continue;
                    }
                    "slow_down" => {
                        debug!("Rate limited, slowing down");
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        continue;
                    }
                    "expired_token" => {
                        anyhow::bail!("Device code expired. Please try again.");
                    }
                    "access_denied" => {
                        anyhow::bail!("Access denied by user.");
                    }
                    _ => {
                        anyhow::bail!("GitHub error: {}", error);
                    }
                },
            }
        }
    }

    /// Step 3: Get user info from access token
    pub async fn get_user(&self, access_token: &str) -> Result<GitHubUser> {
        debug!("Fetching GitHub user info");

        let response = self
            .client
            .get(GITHUB_USER_URL)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("User-Agent", "bounty-challenge/0.1.0")
            .send()
            .await
            .context("Failed to fetch user info")?;

        if !response.status().is_success() {
            let text = response.text().await?;
            anyhow::bail!("GitHub user fetch failed: {}", text);
        }

        let user: GitHubUser = response
            .json()
            .await
            .context("Failed to parse user response")?;

        info!("GitHub user verified: {}", user.login);
        Ok(user)
    }

    /// Complete flow: request code, wait for auth, get user
    pub async fn authenticate_interactive(&self) -> Result<(GitHubUser, DeviceCodeResponse)> {
        let device_code = self.request_device_code().await?;

        // Return device code so caller can display instructions
        // Then poll for token
        let access_token = self.poll_for_token(&device_code).await?;
        let user = self.get_user(&access_token).await?;

        Ok((user, device_code))
    }
}
