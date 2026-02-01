//! Configuration management
//!
//! Loads configuration from config.toml with support for:
//! - GitHub OAuth client ID
//! - Target repositories for bounty tracking
//! - Server binding settings
//! - Reward system parameters

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

const DEFAULT_CONFIG: &str = include_str!("../config.toml");

/// Main configuration structure matching config.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub github: GitHubConfig,
    pub server: ServerConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    pub rewards: RewardsConfig,
}

/// GitHub configuration including OAuth and target repositories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    /// GitHub OAuth App Client ID (for Device Flow authentication)
    pub client_id: String,
    /// Target repositories for bounty tracking
    #[serde(default)]
    pub repos: Vec<RepoConfig>,
}

/// Repository configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoConfig {
    pub owner: String,
    pub repo: String,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

/// Database configuration (uses DATABASE_URL env var in practice)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DatabaseConfig {
    // Database URL is read from DATABASE_URL environment variable
    // This section exists for documentation and future extensibility
}

/// Rewards system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardsConfig {
    /// Points needed for full weight (100% = 1.0)
    pub max_points_for_full_weight: u32,
    /// Weight earned per point (e.g., 0.02 = 2% per point)
    pub weight_per_point: f64,
    /// Label required on issues to count as valid
    pub valid_label: String,
}

impl Config {
    /// Load from config.toml or use defaults
    pub fn load() -> Result<Self> {
        Self::load_from("config.toml")
    }

    /// Load from specific path
    pub fn load_from(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        if path.exists() {
            let content = std::fs::read_to_string(path).context("Failed to read config file")?;
            toml::from_str(&content).context("Failed to parse config file")
        } else {
            // Use embedded default config
            toml::from_str(DEFAULT_CONFIG).context("Failed to parse default config")
        }
    }

    /// Get GitHub client ID (env var takes precedence, required if config value is empty)
    pub fn github_client_id(&self) -> Option<String> {
        match std::env::var("GITHUB_CLIENT_ID") {
            Ok(id) if !id.is_empty() => Some(id),
            _ => {
                if self.github.client_id.is_empty() {
                    None
                } else {
                    Some(self.github.client_id.clone())
                }
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        // The embedded default config is validated at compile time,
        // so this should never fail. Using a fallback for robustness.
        toml::from_str(DEFAULT_CONFIG).unwrap_or_else(|_| Self {
            github: GitHubConfig {
                client_id: String::new(),
                repos: vec![RepoConfig {
                    owner: "PlatformNetwork".to_string(),
                    repo: "bounty-challenge".to_string(),
                }],
            },
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
            },
            database: DatabaseConfig::default(),
            rewards: RewardsConfig {
                max_points_for_full_weight: 50,
                weight_per_point: 0.02,
                valid_label: "valid".to_string(),
            },
        })
    }
}
