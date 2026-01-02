//! Configuration management

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

const DEFAULT_CONFIG: &str = include_str!("../config.toml");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub github: GitHubConfig,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub scoring: ScoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    pub client_id: String,
    pub owner: String,
    pub repo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringConfig {
    pub divisor: f64,
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

    /// Get GitHub client ID (env var takes precedence)
    pub fn github_client_id(&self) -> String {
        std::env::var("GITHUB_CLIENT_ID").unwrap_or_else(|_| self.github.client_id.clone())
    }
}

impl Default for Config {
    fn default() -> Self {
        toml::from_str(DEFAULT_CONFIG).expect("Default config should be valid")
    }
}
