//! Metagraph Cache
//!
//! Verifies hotkeys are registered on the Bittensor subnet.

use parking_lot::RwLock;
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

const CACHE_REFRESH_INTERVAL: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Deserialize)]
pub struct MinerInfo {
    pub hotkey: String,
    #[serde(default)]
    pub stake: u64,
    #[serde(default)]
    pub is_active: bool,
}

/// Metagraph cache for registered hotkeys
pub struct MetagraphCache {
    platform_url: String,
    hotkeys: Arc<RwLock<HashSet<String>>>,
    miners: Arc<RwLock<Vec<MinerInfo>>>,
    last_refresh: Arc<RwLock<Option<Instant>>>,
}

impl MetagraphCache {
    pub fn new(platform_url: String) -> Self {
        Self {
            platform_url,
            hotkeys: Arc::new(RwLock::new(HashSet::new())),
            miners: Arc::new(RwLock::new(Vec::new())),
            last_refresh: Arc::new(RwLock::new(None)),
        }
    }

    /// Check if a hotkey is registered in the metagraph
    pub fn is_registered(&self, hotkey: &str) -> bool {
        let hotkeys = self.hotkeys.read();
        let normalized = hotkey.to_lowercase();
        
        if hotkeys.contains(&normalized) {
            return true;
        }

        // Also check SS58 format
        hotkeys.contains(hotkey)
    }

    pub fn count(&self) -> usize {
        self.hotkeys.read().len()
    }

    pub fn needs_refresh(&self) -> bool {
        let last = self.last_refresh.read();
        match *last {
            None => true,
            Some(t) => t.elapsed() > CACHE_REFRESH_INTERVAL,
        }
    }

    /// Refresh from Platform Server
    pub async fn refresh(&self) -> Result<usize, String> {
        debug!("Refreshing metagraph cache from {}", self.platform_url);

        let client = reqwest::Client::new();
        let url = format!("{}/api/v1/miners", self.platform_url);

        let response = client
            .get(&url)
            .timeout(Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Platform Server: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Platform Server returned error: {}", response.status()));
        }

        let miners: Vec<MinerInfo> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse miner list: {}", e))?;

        let mut new_hotkeys = HashSet::new();
        for miner in &miners {
            new_hotkeys.insert(miner.hotkey.to_lowercase());
            new_hotkeys.insert(miner.hotkey.clone());
        }

        let count = miners.len();

        {
            let mut hotkeys = self.hotkeys.write();
            *hotkeys = new_hotkeys;
        }
        {
            let mut cached_miners = self.miners.write();
            *cached_miners = miners;
        }
        {
            let mut last = self.last_refresh.write();
            *last = Some(Instant::now());
        }

        info!("Metagraph cache refreshed: {} miners", count);
        Ok(count)
    }

    /// Get all miners
    pub fn get_miners(&self) -> Vec<MinerInfo> {
        self.miners.read().clone()
    }
}
