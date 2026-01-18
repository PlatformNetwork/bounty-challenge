//! PostgreSQL Storage for Bounty Challenge
//!
//! Provides persistent storage for the reward system.
//! Connects to PostgreSQL with DATABASE_URL from platform-challenge.

use anyhow::Result;
use chrono::{DateTime, Utc};
use deadpool_postgres::{Config, Pool, Runtime};
use serde::{Deserialize, Serialize};
use tokio_postgres::NoTls;
use tracing::{debug, info};

/// Maximum emission rate: 250 issues per day = full weight
pub const MAX_ISSUES_FOR_FULL_EMISSION: i32 = 250;

/// Base weight per resolved issue
pub const BASE_WEIGHT_PER_ISSUE: f64 = 0.01;

/// Threshold after which weight per issue decreases
pub const ADAPTATION_THRESHOLD: i32 = 100;

/// Database pool configuration
const DB_POOL_MAX_SIZE: usize = 20;
const DB_QUERY_TIMEOUT_SECS: u64 = 30;

// ============================================================================
// DATA STRUCTURES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRegistration {
    pub id: i32,
    pub github_username: String,
    pub hotkey: String,
    pub registered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetRepo {
    pub id: i32,
    pub owner: String,
    pub repo: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedIssue {
    pub id: i32,
    pub issue_id: i64,
    pub repo_owner: String,
    pub repo_name: String,
    pub github_username: String,
    pub hotkey: Option<String>,
    pub issue_url: String,
    pub issue_title: Option<String>,
    pub resolved_at: DateTime<Utc>,
    pub weight_attributed: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardSnapshot {
    pub id: i32,
    pub snapshot_at: DateTime<Utc>,
    pub github_username: String,
    pub hotkey: String,
    pub issues_resolved_24h: i32,
    pub total_issues_24h: i32,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentWeight {
    pub github_username: String,
    pub hotkey: String,
    pub issues_resolved_24h: i32,
    pub total_issues_24h: i32,
    pub weight: f64,
    #[serde(default)]
    pub is_penalized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidIssue {
    pub id: i32,
    pub issue_id: i64,
    pub repo_owner: String,
    pub repo_name: String,
    pub github_username: String,
    pub hotkey: Option<String>,
    pub issue_url: String,
    pub issue_title: Option<String>,
    pub reason: Option<String>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBalance {
    pub hotkey: String,
    pub github_username: String,
    pub valid_count: i32,
    pub invalid_count: i32,
    pub balance: i32,
    pub is_penalized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStats {
    pub date: chrono::NaiveDate,
    pub total_issues_opened: i32,
    pub total_issues_resolved: i32,
    pub unique_contributors: i32,
    pub total_weight_distributed: f64,
}

// ============================================================================
// PG STORAGE
// ============================================================================

#[derive(Clone)]
pub struct PgStorage {
    pool: Pool,
}

impl PgStorage {
    /// Create storage from DATABASE_URL
    pub async fn new(database_url: &str) -> Result<Self> {
        use deadpool_postgres::{ManagerConfig, PoolConfig, RecyclingMethod};
        use std::time::Duration;

        let mut config = Config::new();
        config.url = Some(database_url.to_string());

        config.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });

        config.pool = Some(PoolConfig {
            max_size: DB_POOL_MAX_SIZE,
            timeouts: deadpool_postgres::Timeouts {
                wait: Some(Duration::from_secs(DB_QUERY_TIMEOUT_SECS)),
                create: Some(Duration::from_secs(10)),
                recycle: Some(Duration::from_secs(30)),
            },
            ..Default::default()
        });

        let pool = config.create_pool(Some(Runtime::Tokio1), NoTls)?;

        // Test connection
        let client = pool.get().await?;
        client
            .execute(
                &format!("SET statement_timeout = '{}s'", DB_QUERY_TIMEOUT_SECS),
                &[],
            )
            .await?;

        info!(
            "Connected to PostgreSQL (pool_size: {}, query_timeout: {}s)",
            DB_POOL_MAX_SIZE, DB_QUERY_TIMEOUT_SECS
        );

        let storage = Self { pool };
        storage.run_migrations().await?;

        Ok(storage)
    }

    /// Create storage from DATABASE_URL environment variable
    pub async fn from_env() -> Result<Self> {
        let url =
            std::env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL not set"))?;
        Self::new(&url).await
    }

    /// Run embedded migrations
    async fn run_migrations(&self) -> Result<()> {
        let client = self.pool.get().await?;

        // Check if migrations table exists
        let exists: bool = client
            .query_one(
                "SELECT EXISTS(SELECT 1 FROM information_schema.tables WHERE table_name = 'schema_migrations')",
                &[],
            )
            .await?
            .get(0);

        if !exists {
            // Run initial schema migration
            let migration_sql = include_str!("../migrations/001_schema.sql");
            client.batch_execute(migration_sql).await?;
            info!("Applied migration 001_schema");
        }

        // Check for penalty system migration (version 3)
        let has_penalty: bool = client
            .query_one(
                "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 3)",
                &[],
            )
            .await?
            .get(0);

        if !has_penalty {
            let migration_sql = include_str!("../migrations/002_penalty.sql");
            client.batch_execute(migration_sql).await?;
            info!("Applied migration 002_penalty");
        }

        Ok(())
    }

    // ========================================================================
    // REGISTRATIONS
    // ========================================================================

    /// Register a GitHub username with a hotkey
    pub async fn register_user(&self, github_username: &str, hotkey: &str) -> Result<()> {
        let client = self.pool.get().await?;
        let username_lower = github_username.to_lowercase();

        // Use upsert with ON CONFLICT for github_username
        // First, try to delete any existing registration for this hotkey (to avoid conflicts)
        client
            .execute(
                "DELETE FROM github_registrations WHERE hotkey = $1 AND github_username != $2",
                &[&hotkey, &username_lower],
            )
            .await?;

        // Now insert or update
        client
            .execute(
                "INSERT INTO github_registrations (github_username, hotkey)
                 VALUES ($1, $2)
                 ON CONFLICT (github_username) DO UPDATE SET hotkey = EXCLUDED.hotkey, registered_at = NOW()",
                &[&username_lower, &hotkey],
            )
            .await?;

        info!("Registered {} with hotkey {}", github_username, &hotkey[..16.min(hotkey.len())]);
        Ok(())
    }

    /// Get hotkey for a GitHub username
    pub async fn get_hotkey_by_github(&self, github_username: &str) -> Result<Option<String>> {
        let client = self.pool.get().await?;

        let row = client
            .query_opt(
                "SELECT hotkey FROM github_registrations WHERE LOWER(github_username) = LOWER($1)",
                &[&github_username],
            )
            .await?;

        Ok(row.map(|r| r.get(0)))
    }

    /// Get GitHub username for a hotkey
    pub async fn get_github_by_hotkey(&self, hotkey: &str) -> Result<Option<String>> {
        let client = self.pool.get().await?;

        let row = client
            .query_opt(
                "SELECT github_username FROM github_registrations WHERE hotkey = $1",
                &[&hotkey],
            )
            .await?;

        Ok(row.map(|r| r.get(0)))
    }

    // ========================================================================
    // TARGET REPOS
    // ========================================================================

    /// Get all active target repositories
    pub async fn get_active_repos(&self) -> Result<Vec<TargetRepo>> {
        let client = self.pool.get().await?;

        let rows = client
            .query(
                "SELECT id, owner, repo, active FROM target_repos WHERE active = true",
                &[],
            )
            .await?;

        Ok(rows
            .iter()
            .map(|r| TargetRepo {
                id: r.get(0),
                owner: r.get(1),
                repo: r.get(2),
                active: r.get(3),
            })
            .collect())
    }

    /// Add a target repository
    pub async fn add_target_repo(&self, owner: &str, repo: &str) -> Result<()> {
        let client = self.pool.get().await?;

        client
            .execute(
                "INSERT INTO target_repos (owner, repo) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                &[&owner, &repo],
            )
            .await?;

        info!("Added target repo {}/{}", owner, repo);
        Ok(())
    }

    // ========================================================================
    // RESOLVED ISSUES
    // ========================================================================

    /// Record a resolved issue
    pub async fn record_resolved_issue(
        &self,
        issue_id: i64,
        repo_owner: &str,
        repo_name: &str,
        github_username: &str,
        issue_url: &str,
        issue_title: Option<&str>,
        resolved_at: DateTime<Utc>,
    ) -> Result<bool> {
        let client = self.pool.get().await?;

        // Get hotkey if registered
        let hotkey = self.get_hotkey_by_github(github_username).await?;

        // Calculate weight at time of resolution
        let weight = self.calculate_issue_weight().await?;

        let result = client
            .execute(
                "INSERT INTO resolved_issues (issue_id, repo_owner, repo_name, github_username, hotkey, issue_url, issue_title, resolved_at, weight_attributed)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                 ON CONFLICT (repo_owner, repo_name, issue_id) DO NOTHING",
                &[
                    &issue_id,
                    &repo_owner,
                    &repo_name,
                    &github_username.to_lowercase(),
                    &hotkey,
                    &issue_url,
                    &issue_title,
                    &resolved_at,
                    &(weight as f32),
                ],
            )
            .await?;

        if result > 0 {
            info!(
                "Recorded issue #{} from {}/{} by {} (weight: {:.4})",
                issue_id, repo_owner, repo_name, github_username, weight
            );
            Ok(true)
        } else {
            debug!("Issue #{} already recorded", issue_id);
            Ok(false)
        }
    }

    /// Check if an issue is already recorded
    pub async fn is_issue_recorded(&self, repo_owner: &str, repo_name: &str, issue_id: i64) -> Result<bool> {
        let client = self.pool.get().await?;

        let row = client
            .query_opt(
                "SELECT 1 FROM resolved_issues WHERE repo_owner = $1 AND repo_name = $2 AND issue_id = $3",
                &[&repo_owner, &repo_name, &issue_id],
            )
            .await?;

        Ok(row.is_some())
    }

    /// Get resolved issues for a user in the last 24h
    pub async fn get_user_issues_24h(&self, github_username: &str) -> Result<Vec<ResolvedIssue>> {
        let client = self.pool.get().await?;

        let rows = client
            .query(
                "SELECT id, issue_id, repo_owner, repo_name, github_username, hotkey, issue_url, issue_title, resolved_at, weight_attributed::FLOAT8
                 FROM resolved_issues
                 WHERE LOWER(github_username) = LOWER($1)
                   AND resolved_at >= NOW() - INTERVAL '24 hours'
                 ORDER BY resolved_at DESC",
                &[&github_username],
            )
            .await?;

        Ok(rows
            .iter()
            .map(|r| ResolvedIssue {
                id: r.get(0),
                issue_id: r.get(1),
                repo_owner: r.get(2),
                repo_name: r.get(3),
                github_username: r.get(4),
                hotkey: r.get(5),
                issue_url: r.get(6),
                issue_title: r.get(7),
                resolved_at: r.get(8),
                weight_attributed: r.get(9),
            })
            .collect())
    }

    // ========================================================================
    // WEIGHT CALCULATION
    // ========================================================================

    /// Calculate the current weight for a single issue
    /// Based on: max 250 issues/day = full emission, 0.01 per issue, adaptive if > 100
    pub async fn calculate_issue_weight(&self) -> Result<f64> {
        let client = self.pool.get().await?;

        let row = client
            .query_one(
                "SELECT COUNT(*) FROM resolved_issues WHERE resolved_at >= NOW() - INTERVAL '24 hours'",
                &[],
            )
            .await?;

        let total_issues_24h: i64 = row.get(0);
        let total = total_issues_24h as i32;

        // Adaptive weight calculation
        let weight_per_issue = if total > ADAPTATION_THRESHOLD {
            BASE_WEIGHT_PER_ISSUE * (ADAPTATION_THRESHOLD as f64 / total as f64)
        } else {
            BASE_WEIGHT_PER_ISSUE
        };

        Ok(weight_per_issue)
    }

    /// Get current weights for all registered users
    pub async fn get_current_weights(&self) -> Result<Vec<CurrentWeight>> {
        let client = self.pool.get().await?;

        let rows = client
            .query(
                "SELECT github_username, hotkey, issues_resolved_24h, total_issues_24h, weight::FLOAT8, is_penalized 
                 FROM current_weights ORDER BY weight DESC",
                &[],
            )
            .await?;

        Ok(rows
            .iter()
            .map(|r| CurrentWeight {
                github_username: r.get(0),
                hotkey: r.get(1),
                issues_resolved_24h: r.get(2),
                total_issues_24h: r.get(3),
                weight: r.get(4),
                is_penalized: r.get(5),
            })
            .collect())
    }

    /// Calculate weight for a specific user
    pub async fn calculate_user_weight(&self, hotkey: &str) -> Result<f64> {
        let client = self.pool.get().await?;

        // Get total issues in 24h
        let total_row = client
            .query_one(
                "SELECT COUNT(*) FROM resolved_issues WHERE resolved_at >= NOW() - INTERVAL '24 hours'",
                &[],
            )
            .await?;
        let total_issues_24h: i64 = total_row.get(0);
        let total = total_issues_24h as i32;

        // Get user's issues in 24h
        let user_row = client
            .query_one(
                "SELECT COUNT(*) FROM resolved_issues 
                 WHERE hotkey = $1 AND resolved_at >= NOW() - INTERVAL '24 hours'",
                &[&hotkey],
            )
            .await?;
        let user_issues: i64 = user_row.get(0);
        let user_count = user_issues as i32;

        // Calculate weight
        let weight = calculate_weight(user_count, total);
        Ok(weight)
    }

    // ========================================================================
    // SNAPSHOTS
    // ========================================================================

    /// Take a snapshot of current weights
    pub async fn take_snapshot(&self) -> Result<i32> {
        let client = self.pool.get().await?;

        let result = client
            .execute(
                "INSERT INTO reward_snapshots (snapshot_at, github_username, hotkey, issues_resolved_24h, total_issues_24h, weight)
                 SELECT NOW(), github_username, hotkey, issues_resolved_24h, total_issues_24h, weight
                 FROM current_weights",
                &[],
            )
            .await?;

        info!("Took snapshot of {} weight entries", result);
        Ok(result as i32)
    }

    /// Get snapshots for a hotkey
    pub async fn get_snapshots_for_hotkey(&self, hotkey: &str, limit: i32) -> Result<Vec<RewardSnapshot>> {
        let client = self.pool.get().await?;

        let rows = client
            .query(
                "SELECT id, snapshot_at, github_username, hotkey, issues_resolved_24h, total_issues_24h, weight::FLOAT8
                 FROM reward_snapshots
                 WHERE hotkey = $1
                 ORDER BY snapshot_at DESC
                 LIMIT $2",
                &[&hotkey, &(limit as i64)],
            )
            .await?;

        Ok(rows
            .iter()
            .map(|r| RewardSnapshot {
                id: r.get(0),
                snapshot_at: r.get(1),
                github_username: r.get(2),
                hotkey: r.get(3),
                issues_resolved_24h: r.get(4),
                total_issues_24h: r.get(5),
                weight: r.get(6),
            })
            .collect())
    }

    // ========================================================================
    // STATS
    // ========================================================================

    /// Get stats for the last 24 hours
    pub async fn get_stats_24h(&self) -> Result<DailyStats> {
        let client = self.pool.get().await?;

        let row = client
            .query_one(
                "SELECT 
                    COUNT(*) as total_resolved,
                    COUNT(DISTINCT github_username) as unique_contributors,
                    COALESCE(SUM(weight_attributed), 0.0)::FLOAT8 as total_weight
                 FROM resolved_issues
                 WHERE resolved_at >= NOW() - INTERVAL '24 hours'",
                &[],
            )
            .await?;

        Ok(DailyStats {
            date: chrono::Utc::now().date_naive(),
            total_issues_opened: 0, // Would need GitHub API to track opens
            total_issues_resolved: row.get::<_, i64>(0) as i32,
            unique_contributors: row.get::<_, i64>(1) as i32,
            total_weight_distributed: row.get(2),
        })
    }

    /// Get leaderboard (top users by weight)
    pub async fn get_leaderboard(&self, limit: i32) -> Result<Vec<CurrentWeight>> {
        let client = self.pool.get().await?;

        let rows = client
            .query(
                "SELECT github_username, hotkey, issues_resolved_24h, total_issues_24h, weight::FLOAT8, is_penalized
                 FROM current_weights
                 ORDER BY weight DESC
                 LIMIT $1",
                &[&(limit as i64)],
            )
            .await?;

        Ok(rows
            .iter()
            .map(|r| CurrentWeight {
                github_username: r.get(0),
                hotkey: r.get(1),
                issues_resolved_24h: r.get(2),
                total_issues_24h: r.get(3),
                weight: r.get(4),
                is_penalized: r.get(5),
            })
            .collect())
    }

    // ========================================================================
    // PENALTY SYSTEM
    // ========================================================================

    /// Record an invalid issue (closed without 'valid' label)
    pub async fn record_invalid_issue(
        &self,
        issue_id: i64,
        repo_owner: &str,
        repo_name: &str,
        github_username: &str,
        issue_url: &str,
        issue_title: Option<&str>,
        reason: Option<&str>,
    ) -> Result<()> {
        let client = self.pool.get().await?;

        // Look up hotkey for this GitHub user
        let hotkey = self.get_hotkey_by_github(github_username).await?;

        client
            .execute(
                "INSERT INTO invalid_issues (issue_id, repo_owner, repo_name, github_username, hotkey, issue_url, issue_title, reason)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                 ON CONFLICT (repo_owner, repo_name, issue_id) DO NOTHING",
                &[
                    &issue_id,
                    &repo_owner,
                    &repo_name,
                    &github_username.to_lowercase(),
                    &hotkey,
                    &issue_url,
                    &issue_title,
                    &reason,
                ],
            )
            .await?;

        info!(
            "Recorded invalid issue #{} from {}/{} by {}",
            issue_id, repo_owner, repo_name, github_username
        );
        Ok(())
    }

    /// Get user balance (valid - invalid issues)
    pub async fn get_user_balance(&self, hotkey: &str) -> Result<Option<UserBalance>> {
        let client = self.pool.get().await?;

        let row = client
            .query_opt(
                "SELECT hotkey, github_username, valid_count, invalid_count, balance, is_penalized
                 FROM user_balance
                 WHERE hotkey = $1",
                &[&hotkey],
            )
            .await?;

        Ok(row.map(|r| UserBalance {
            hotkey: r.get(0),
            github_username: r.get(1),
            valid_count: r.get(2),
            invalid_count: r.get(3),
            balance: r.get(4),
            is_penalized: r.get(5),
        }))
    }

    /// Check if a hotkey is penalized
    pub async fn is_penalized(&self, hotkey: &str) -> Result<bool> {
        let balance = self.get_user_balance(hotkey).await?;
        Ok(balance.map(|b| b.is_penalized).unwrap_or(false))
    }

    /// Get all invalid issues for a hotkey
    pub async fn get_invalid_issues_for_hotkey(&self, hotkey: &str) -> Result<Vec<InvalidIssue>> {
        let client = self.pool.get().await?;

        let rows = client
            .query(
                "SELECT id, issue_id, repo_owner, repo_name, github_username, hotkey, issue_url, issue_title, reason, recorded_at
                 FROM invalid_issues
                 WHERE hotkey = $1
                 ORDER BY recorded_at DESC",
                &[&hotkey],
            )
            .await?;

        Ok(rows
            .iter()
            .map(|r| InvalidIssue {
                id: r.get(0),
                issue_id: r.get(1),
                repo_owner: r.get(2),
                repo_name: r.get(3),
                github_username: r.get(4),
                hotkey: r.get(5),
                issue_url: r.get(6),
                issue_title: r.get(7),
                reason: r.get(8),
                recorded_at: r.get(9),
            })
            .collect())
    }

    /// Get penalty stats
    pub async fn get_penalty_stats(&self) -> Result<(i32, i32)> {
        let client = self.pool.get().await?;

        let row = client
            .query_one(
                "SELECT 
                    COUNT(*) FILTER (WHERE is_penalized = true) as penalized_count,
                    COUNT(*) as total_users
                 FROM user_balance",
                &[],
            )
            .await?;

        Ok((row.get::<_, i64>(0) as i32, row.get::<_, i64>(1) as i32))
    }
}

// ============================================================================
// WEIGHT CALCULATION (standalone function)
// ============================================================================

/// Calculate weight for a user based on their issues and total issues in 24h
/// 
/// Rules:
/// - Maximum emission rate reached at 250 issues per day
/// - max_weight_total = min(total_issues_24h / 250, 1.0)
/// - Each issue gives max 0.01 weight
/// - If total > 100 issues, weight per issue decreases proportionally
pub fn calculate_weight(issues_resolved_by_user: i32, total_issues_24h: i32) -> f64 {
    if issues_resolved_by_user == 0 || total_issues_24h == 0 {
        return 0.0;
    }

    // Maximum total weight available
    let max_weight_total = (total_issues_24h as f64 / MAX_ISSUES_FOR_FULL_EMISSION as f64).min(1.0);

    // Adapt weight per issue if too many issues
    let weight_per_issue = if total_issues_24h > ADAPTATION_THRESHOLD {
        BASE_WEIGHT_PER_ISSUE * (ADAPTATION_THRESHOLD as f64 / total_issues_24h as f64)
    } else {
        BASE_WEIGHT_PER_ISSUE
    };

    // User's weight (capped at max_weight_total)
    let user_weight = (issues_resolved_by_user as f64 * weight_per_issue).min(max_weight_total);

    user_weight
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weight_calculation_basic() {
        // 1 issue out of 10 = 0.01
        assert!((calculate_weight(1, 10) - 0.01).abs() < 0.0001);
        
        // 5 issues out of 50 = 0.05
        assert!((calculate_weight(5, 50) - 0.05).abs() < 0.0001);
    }

    #[test]
    fn test_weight_calculation_adaptive() {
        // 10 issues out of 200 (>100, so adaptive)
        // weight_per_issue = 0.01 * (100/200) = 0.005
        // user_weight = 10 * 0.005 = 0.05
        let weight = calculate_weight(10, 200);
        assert!((weight - 0.05).abs() < 0.0001);
    }

    #[test]
    fn test_weight_calculation_max_cap() {
        // 250 issues = max emission (1.0)
        // Even with 1000 issues resolved, max weight is 1.0
        let weight = calculate_weight(1000, 250);
        assert!(weight <= 1.0);
    }

    #[test]
    fn test_weight_calculation_zero() {
        assert_eq!(calculate_weight(0, 100), 0.0);
        assert_eq!(calculate_weight(10, 0), 0.0);
    }
}
