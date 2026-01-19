//! PostgreSQL Storage for Bounty Challenge
//!
//! Provides persistent storage for the reward system.
//! Connects to PostgreSQL with DATABASE_URL from platform-challenge.

use anyhow::Result;
use chrono::{DateTime, Utc};
use deadpool_postgres::{Config, Pool, Runtime};
use serde::{Deserialize, Serialize};
use tokio_postgres::NoTls;
use tracing::{debug, info, warn};

/// Maximum points for full weight (100 points = 100%)
pub const MAX_POINTS_FOR_FULL_WEIGHT: f64 = 100.0;

/// Weight per point (1 point = 1% = 0.01)
pub const WEIGHT_PER_POINT: f64 = 0.01;

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
    pub issues_resolved_24h: i64,
    pub total_issues_24h: i64,
    pub weight: f64,
    #[serde(default)]
    pub is_penalized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub github_username: String,
    pub hotkey: Option<String>,
    pub valid_issues: i32,
    pub pending_issues: i32,
    pub weight: f64,
    pub is_penalized: bool,
    pub last_activity: Option<DateTime<Utc>>,
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

        // Check for github issues migration (version 4)
        let has_issues: bool = client
            .query_one(
                "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 4)",
                &[],
            )
            .await?
            .get(0);

        if !has_issues {
            let migration_sql = include_str!("../migrations/003_github_issues.sql");
            client.batch_execute(migration_sql).await?;
            info!("Applied migration 003_github_issues");
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

    /// Get multiplier for a repository (defaults to 1.0 if not found)
    pub async fn get_repo_multiplier(&self, owner: &str, repo: &str) -> Result<f64> {
        let client = self.pool.get().await?;

        let row = client
            .query_opt(
                "SELECT multiplier FROM target_repos WHERE owner = $1 AND repo = $2",
                &[&owner, &repo],
            )
            .await?;

        Ok(row.map(|r| r.get::<_, f32>(0) as f64).unwrap_or(1.0))
    }

    /// Get multiplier from project tag (cortex, vgrep, etc.)
    /// Returns the multiplier for the first matching project tag found in labels
    pub async fn get_tag_multiplier(&self, labels: &[String]) -> Result<f64> {
        let client = self.pool.get().await?;

        // Check each label against project_tags
        for label in labels {
            let row = client
                .query_opt(
                    "SELECT multiplier FROM project_tags WHERE tag = $1 AND active = true",
                    &[&label.to_lowercase()],
                )
                .await?;

            if let Some(r) = row {
                return Ok(r.get::<_, f32>(0) as f64);
            }
        }

        // Default multiplier if no project tag found
        Ok(1.0)
    }

    // ========================================================================
    // RESOLVED ISSUES
    // ========================================================================

    /// Record a resolved issue
    /// 
    /// Points are based on multiplier:
    /// - cortex: 5 points
    /// - term-challenge: 1 point
    /// - vgrep: 1 point
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

        // Get repo multiplier (points per issue: cortex=5, vgrep=1, term-challenge=1)
        let multiplier = self.get_repo_multiplier(repo_owner, repo_name).await?;

        let result = client
            .execute(
                "INSERT INTO resolved_issues (issue_id, repo_owner, repo_name, github_username, hotkey, issue_url, issue_title, resolved_at, weight_attributed, multiplier)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
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
                    &(0.0_f32), // weight_attributed is deprecated, points come from multiplier
                    &(multiplier as f32),
                ],
            )
            .await?;

        if result > 0 {
            info!(
                "Recorded issue #{} from {}/{} by {} ({} points)",
                issue_id, repo_owner, repo_name, github_username, multiplier
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

    /// Calculate weight for a specific user based on points (multiplier * issues)
    /// 
    /// Points system:
    /// - cortex: 5 points per issue
    /// - term-challenge: 1 point per issue  
    /// - vgrep: 1 point per issue
    /// - 100 points = 100% weight
    pub async fn calculate_user_weight(&self, hotkey: &str) -> Result<f64> {
        let client = self.pool.get().await?;

        // Get user's total points (SUM of multipliers) in last 24h
        let user_row = client
            .query_one(
                "SELECT COALESCE(SUM(multiplier), 0) FROM resolved_issues 
                 WHERE hotkey = $1 AND resolved_at >= NOW() - INTERVAL '24 hours'",
                &[&hotkey],
            )
            .await?;
        let user_points: f64 = user_row.get::<_, f64>(0);

        // Calculate weight: points * 0.01, capped at 1.0 (100%)
        let weight = (user_points * WEIGHT_PER_POINT).min(1.0);
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

    /// Get pending issues count for a user (issues they created that are not yet validated)
    pub async fn get_user_pending_count(&self, github_username: &str) -> Result<i32> {
        let client = self.pool.get().await?;

        let row = client
            .query_one(
                "SELECT COUNT(*) FROM github_issues 
                 WHERE LOWER(github_username) = LOWER($1) 
                   AND state = 'open'
                   AND NOT 'valid' = ANY(labels) 
                   AND NOT 'invalid' = ANY(labels)",
                &[&github_username],
            )
            .await?;

        Ok(row.get::<_, i64>(0) as i32)
    }

    /// Get all users with their pending issues count
    pub async fn get_all_users_pending(&self) -> Result<std::collections::HashMap<String, i32>> {
        let client = self.pool.get().await?;

        let rows = client
            .query(
                "SELECT LOWER(github_username), COUNT(*) 
                 FROM github_issues 
                 WHERE state = 'open'
                   AND NOT 'valid' = ANY(labels) 
                   AND NOT 'invalid' = ANY(labels)
                 GROUP BY LOWER(github_username)",
                &[],
            )
            .await?;

        let mut map = std::collections::HashMap::new();
        for row in rows {
            let username: String = row.get(0);
            let count: i64 = row.get(1);
            map.insert(username, count as i32);
        }

        Ok(map)
    }

    /// Get extended leaderboard including users with pending issues
    pub async fn get_extended_leaderboard(&self, limit: i32) -> Result<Vec<LeaderboardEntry>> {
        let client = self.pool.get().await?;

        // Get all users from registrations, their valid issues, pending issues, and weights
        let rows = client
            .query(
                "WITH user_valid AS (
                    SELECT 
                        LOWER(github_username) as username,
                        COUNT(*) as valid_count,
                        MAX(resolved_at) as last_valid
                    FROM resolved_issues
                    WHERE resolved_at >= NOW() - INTERVAL '24 hours'
                    GROUP BY LOWER(github_username)
                ),
                user_pending AS (
                    SELECT 
                        LOWER(github_username) as username,
                        COUNT(*) as pending_count,
                        MAX(updated_at) as last_pending
                    FROM github_issues
                    WHERE state = 'open'
                      AND NOT 'valid' = ANY(labels)
                      AND NOT 'invalid' = ANY(labels)
                    GROUP BY LOWER(github_username)
                ),
                user_weights AS (
                    SELECT github_username, hotkey, weight, is_penalized
                    FROM current_weights
                )
                SELECT 
                    COALESCE(r.github_username, uv.username, up.username) as github_username,
                    r.hotkey,
                    COALESCE(uv.valid_count, 0)::INTEGER as valid_issues,
                    COALESCE(up.pending_count, 0)::INTEGER as pending_issues,
                    COALESCE(uw.weight, 0.0)::FLOAT8 as weight,
                    COALESCE(uw.is_penalized, false) as is_penalized,
                    GREATEST(uv.last_valid, up.last_pending) as last_activity
                FROM github_registrations r
                FULL OUTER JOIN user_valid uv ON LOWER(r.github_username) = uv.username
                FULL OUTER JOIN user_pending up ON LOWER(r.github_username) = up.username
                LEFT JOIN user_weights uw ON LOWER(r.github_username) = LOWER(uw.github_username)
                WHERE COALESCE(uv.valid_count, 0) > 0 OR COALESCE(up.pending_count, 0) > 0
                ORDER BY weight DESC, last_activity DESC NULLS LAST
                LIMIT $1",
                &[&(limit as i64)],
            )
            .await?;

        Ok(rows
            .iter()
            .map(|r| LeaderboardEntry {
                github_username: r.get(0),
                hotkey: r.get(1),
                valid_issues: r.get(2),
                pending_issues: r.get(3),
                weight: r.get(4),
                is_penalized: r.get(5),
                last_activity: r.get(6),
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

    // ========================================================================
    // STAR BONUS SYSTEM
    // ========================================================================

    /// Get list of repos to watch for stars
    pub async fn get_star_target_repos(&self) -> Result<Vec<StarTargetRepo>> {
        let client = self.pool.get().await?;

        let rows = client
            .query(
                "SELECT owner, repo FROM star_target_repos WHERE active = true",
                &[],
            )
            .await?;

        Ok(rows
            .iter()
            .map(|r| StarTargetRepo {
                owner: r.get(0),
                repo: r.get(1),
            })
            .collect())
    }

    /// Upsert a star (user starred a repo)
    pub async fn upsert_star(&self, github_username: &str, repo_owner: &str, repo_name: &str) -> Result<bool> {
        let client = self.pool.get().await?;

        let result = client
            .execute(
                "INSERT INTO github_stars (github_username, repo_owner, repo_name)
                 VALUES ($1, $2, $3)
                 ON CONFLICT (github_username, repo_owner, repo_name) DO NOTHING",
                &[&github_username.to_lowercase(), &repo_owner, &repo_name],
            )
            .await?;

        Ok(result > 0)
    }

    /// Update star sync timestamp for a repo
    pub async fn update_star_sync(&self, repo_owner: &str, repo_name: &str) -> Result<()> {
        let client = self.pool.get().await?;

        client
            .execute(
                "UPDATE star_target_repos SET last_synced_at = NOW() 
                 WHERE owner = $1 AND repo = $2",
                &[&repo_owner, &repo_name],
            )
            .await?;

        Ok(())
    }

    /// Get star count for a user
    pub async fn get_user_star_count(&self, github_username: &str) -> Result<i32> {
        let client = self.pool.get().await?;

        let row = client
            .query_one(
                "SELECT COUNT(*) FROM github_stars WHERE LOWER(github_username) = LOWER($1)",
                &[&github_username],
            )
            .await?;

        Ok(row.get::<_, i64>(0) as i32)
    }

    /// Get star bonus for a user (0.25 per star if >= 2 resolved issues)
    pub async fn get_user_star_bonus(&self, github_username: &str) -> Result<f64> {
        let client = self.pool.get().await?;

        let row = client
            .query_opt(
                "SELECT star_bonus FROM user_star_bonus WHERE LOWER(github_username) = LOWER($1)",
                &[&github_username],
            )
            .await?;

        Ok(row.map(|r| r.get::<_, f64>(0)).unwrap_or(0.0))
    }

    /// Get star stats
    pub async fn get_star_stats(&self) -> Result<StarStats> {
        let client = self.pool.get().await?;

        let row = client
            .query_one(
                "SELECT 
                    (SELECT COUNT(*) FROM github_stars) as total_stars,
                    (SELECT COUNT(DISTINCT github_username) FROM github_stars) as users_with_stars,
                    (SELECT COUNT(*) FROM user_star_bonus WHERE star_bonus > 0) as users_with_bonus",
                &[],
            )
            .await?;

        Ok(StarStats {
            total_stars: row.get::<_, i64>(0) as i32,
            users_with_stars: row.get::<_, i64>(1) as i32,
            users_with_bonus: row.get::<_, i64>(2) as i32,
        })
    }

    // ========================================================================
    // GITHUB ISSUES SYNC & CACHE
    // ========================================================================

    /// Get last sync time for a repo
    pub async fn get_last_sync(&self, repo_owner: &str, repo_name: &str) -> Result<Option<DateTime<Utc>>> {
        let client = self.pool.get().await?;

        let row = client
            .query_opt(
                "SELECT last_issue_updated_at FROM github_sync_state 
                 WHERE repo_owner = $1 AND repo_name = $2",
                &[&repo_owner, &repo_name],
            )
            .await?;

        Ok(row.and_then(|r| r.get(0)))
    }

    /// Update sync state for a repo
    pub async fn update_sync_state(&self, repo_owner: &str, repo_name: &str, issues_synced: i32) -> Result<()> {
        let client = self.pool.get().await?;

        client
            .execute(
                "INSERT INTO github_sync_state (repo_owner, repo_name, last_sync_at, issues_synced)
                 VALUES ($1, $2, NOW(), $3)
                 ON CONFLICT (repo_owner, repo_name) DO UPDATE SET
                    last_sync_at = NOW(),
                    issues_synced = github_sync_state.issues_synced + $3,
                    last_issue_updated_at = (
                        SELECT MAX(updated_at) FROM github_issues 
                        WHERE repo_owner = $1 AND repo_name = $2
                    )",
                &[&repo_owner, &repo_name, &issues_synced],
            )
            .await?;

        Ok(())
    }

    /// Upsert a GitHub issue and detect label changes
    pub async fn upsert_issue(&self, issue: &crate::github::GitHubIssue, repo_owner: &str, repo_name: &str) -> Result<LabelChange> {
        let client = self.pool.get().await?;

        let new_labels: Vec<String> = issue.label_names();
        let has_valid = new_labels.contains(&"valid".to_string());
        let has_invalid = new_labels.contains(&"invalid".to_string());

        // Check previous labels
        let prev_row = client
            .query_opt(
                "SELECT labels FROM github_issues WHERE repo_owner = $1 AND repo_name = $2 AND issue_id = $3",
                &[&repo_owner, &repo_name, &(issue.number as i64)],
            )
            .await?;

        let (had_valid, had_invalid) = match prev_row {
            Some(r) => {
                let prev_labels: Vec<String> = r.get(0);
                (prev_labels.contains(&"valid".to_string()), prev_labels.contains(&"invalid".to_string()))
            }
            None => (false, false),
        };

        // Detect changes
        let change = if has_invalid && !had_invalid {
            LabelChange::BecameInvalid
        } else if had_valid && !has_valid {
            LabelChange::LostValid
        } else if has_valid && !had_valid {
            LabelChange::BecameValid
        } else if has_valid && had_valid {
            // Already valid - check if already credited
            let already_credited = client
                .query_opt(
                    "SELECT 1 FROM resolved_issues WHERE repo_owner = $1 AND repo_name = $2 AND issue_id = $3",
                    &[&repo_owner, &repo_name, &(issue.number as i64)],
                )
                .await?
                .is_some();
            
            if !already_credited {
                LabelChange::BecameValid // Treat as new valid to trigger credit
            } else {
                LabelChange::None
            }
        } else {
            LabelChange::None
        };

        // Log significant changes and auto-credit valid issues
        match &change {
            LabelChange::BecameInvalid => {
                info!("Issue #{} in {}/{} marked as INVALID", issue.number, repo_owner, repo_name);
            }
            LabelChange::LostValid => {
                warn!("Issue #{} in {}/{} LOST valid label", issue.number, repo_owner, repo_name);
            }
            LabelChange::BecameValid => {
                // Get multiplier from project tag (cortex, vgrep, etc.)
                let multiplier = self.get_tag_multiplier(&new_labels).await.unwrap_or(1.0);
                
                // Find the project tag for logging
                let project_tag = new_labels.iter()
                    .find(|l| ["cortex", "vgrep", "term-challenge", "bounty-challenge"].contains(&l.to_lowercase().as_str()))
                    .map(|s| s.as_str())
                    .unwrap_or("unknown");
                
                info!("Issue #{} in {}/{} marked as VALID - auto-crediting to user @{} (tag={}, {}x)", 
                      issue.number, repo_owner, repo_name, issue.user.login, project_tag, multiplier);
                
                // Auto-credit: add to resolved_issues with tag-based multiplier
                let hotkey = self.get_hotkey_by_github(&issue.user.login).await.ok().flatten();
                let resolved_at = issue.closed_at.unwrap_or(issue.updated_at);
                
                client
                    .execute(
                        "INSERT INTO resolved_issues (issue_id, repo_owner, repo_name, github_username, hotkey, issue_url, issue_title, resolved_at, weight_attributed, multiplier)
                         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 0.01, $9)
                         ON CONFLICT (repo_owner, repo_name, issue_id) DO NOTHING",
                        &[
                            &(issue.number as i64),
                            &repo_owner,
                            &repo_name,
                            &issue.user.login.to_lowercase(),
                            &hotkey,
                            &issue.html_url,
                            &issue.title,
                            &resolved_at,
                            &(multiplier as f32),
                        ],
                    )
                    .await?;
            }
            LabelChange::None => {}
        }

        // Upsert the issue
        client
            .execute(
                "INSERT INTO github_issues (
                    issue_id, repo_owner, repo_name, github_username, title, body,
                    state, labels, created_at, updated_at, closed_at, issue_url, synced_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW())
                ON CONFLICT (repo_owner, repo_name, issue_id) DO UPDATE SET
                    title = $5,
                    body = $6,
                    state = $7,
                    labels = $8,
                    updated_at = $10,
                    closed_at = $11,
                    synced_at = NOW()",
                &[
                    &(issue.number as i64),
                    &repo_owner,
                    &repo_name,
                    &issue.user.login,
                    &issue.title,
                    &issue.body,
                    &issue.state,
                    &new_labels,
                    &issue.created_at,
                    &issue.updated_at,
                    &issue.closed_at,
                    &issue.html_url,
                ],
            )
            .await?;

        Ok(change)
    }

    /// Get all issues with optional filters
    pub async fn get_issues(
        &self,
        state: Option<&str>,
        label: Option<&str>,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<CachedIssue>> {
        let client = self.pool.get().await?;

        let limit_i64 = limit as i64;
        let offset_i64 = offset as i64;

        let query = match (state, label) {
            (Some(s), Some(l)) => {
                client.query(
                    "SELECT issue_id, repo_owner, repo_name, github_username, title, state, labels, 
                            created_at, updated_at, closed_at, issue_url
                     FROM github_issues 
                     WHERE state = $1 AND $2 = ANY(labels)
                     ORDER BY updated_at DESC
                     LIMIT $3 OFFSET $4",
                    &[&s, &l, &limit_i64, &offset_i64],
                ).await?
            }
            (Some(s), None) => {
                client.query(
                    "SELECT issue_id, repo_owner, repo_name, github_username, title, state, labels, 
                            created_at, updated_at, closed_at, issue_url
                     FROM github_issues 
                     WHERE state = $1
                     ORDER BY updated_at DESC
                     LIMIT $2 OFFSET $3",
                    &[&s, &limit_i64, &offset_i64],
                ).await?
            }
            (None, Some(l)) => {
                client.query(
                    "SELECT issue_id, repo_owner, repo_name, github_username, title, state, labels, 
                            created_at, updated_at, closed_at, issue_url
                     FROM github_issues 
                     WHERE $1 = ANY(labels)
                     ORDER BY updated_at DESC
                     LIMIT $2 OFFSET $3",
                    &[&l, &limit_i64, &offset_i64],
                ).await?
            }
            (None, None) => {
                client.query(
                    "SELECT issue_id, repo_owner, repo_name, github_username, title, state, labels, 
                            created_at, updated_at, closed_at, issue_url
                     FROM github_issues 
                     ORDER BY updated_at DESC
                     LIMIT $1 OFFSET $2",
                    &[&limit_i64, &offset_i64],
                ).await?
            }
        };

        Ok(query.iter().map(|r| CachedIssue {
            issue_id: r.get(0),
            repo_owner: r.get(1),
            repo_name: r.get(2),
            github_username: r.get(3),
            title: r.get(4),
            state: r.get(5),
            labels: r.get(6),
            created_at: r.get(7),
            updated_at: r.get(8),
            closed_at: r.get(9),
            issue_url: r.get(10),
        }).collect())
    }

    /// Get pending issues (closed but no valid/invalid label)
    pub async fn get_pending_issues(&self, limit: i32, offset: i32) -> Result<Vec<CachedIssue>> {
        let client = self.pool.get().await?;

        let limit_i64 = limit as i64;
        let offset_i64 = offset as i64;

        let rows = client
            .query(
                "SELECT issue_id, repo_owner, repo_name, github_username, title, state, labels, 
                        created_at, updated_at, closed_at, issue_url
                 FROM pending_issues
                 ORDER BY updated_at DESC
                 LIMIT $1 OFFSET $2",
                &[&limit_i64, &offset_i64],
            )
            .await?;

        Ok(rows.iter().map(|r| CachedIssue {
            issue_id: r.get(0),
            repo_owner: r.get(1),
            repo_name: r.get(2),
            github_username: r.get(3),
            title: r.get(4),
            state: r.get(5),
            labels: r.get(6),
            created_at: r.get(7),
            updated_at: r.get(8),
            closed_at: r.get(9),
            issue_url: r.get(10),
        }).collect())
    }

    /// Get issues count by status
    pub async fn get_issues_stats(&self) -> Result<IssuesStats> {
        let client = self.pool.get().await?;

        let row = client
            .query_one(
                "SELECT 
                    COUNT(*) as total,
                    COUNT(*) FILTER (WHERE state = 'open') as open_count,
                    COUNT(*) FILTER (WHERE state = 'closed') as closed_count,
                    COUNT(*) FILTER (WHERE 'valid' = ANY(labels)) as valid_count,
                    COUNT(*) FILTER (WHERE 'invalid' = ANY(labels)) as invalid_count,
                    COUNT(*) FILTER (WHERE state = 'closed' AND NOT 'valid' = ANY(labels) AND NOT 'invalid' = ANY(labels)) as pending_count
                 FROM github_issues",
                &[],
            )
            .await?;

        Ok(IssuesStats {
            total: row.get::<_, i64>(0) as i32,
            open: row.get::<_, i64>(1) as i32,
            closed: row.get::<_, i64>(2) as i32,
            valid: row.get::<_, i64>(3) as i32,
            invalid: row.get::<_, i64>(4) as i32,
            pending: row.get::<_, i64>(5) as i32,
        })
    }

    /// Get hotkey details
    pub async fn get_hotkey_details(&self, hotkey: &str) -> Result<Option<HotkeyDetails>> {
        let client = self.pool.get().await?;

        // Get registration info
        let reg = client
            .query_opt(
                "SELECT github_username, registered_at FROM github_registrations WHERE hotkey = $1",
                &[&hotkey],
            )
            .await?;

        let (github_username, registered_at): (String, DateTime<Utc>) = match reg {
            Some(r) => (r.get(0), r.get(1)),
            None => return Ok(None),
        };

        // Get balance info
        let balance = self.get_user_balance(hotkey).await?.unwrap_or(UserBalance {
            github_username: github_username.clone(),
            hotkey: hotkey.to_string(),
            valid_count: 0,
            invalid_count: 0,
            balance: 0,
            is_penalized: false,
        });

        // Get weight
        let weight = self.calculate_user_weight(hotkey).await.unwrap_or(0.0);

        // Get recent issues from this user
        let issues = client
            .query(
                "SELECT issue_id, repo_owner, repo_name, title, state, labels, updated_at, issue_url
                 FROM github_issues 
                 WHERE LOWER(github_username) = LOWER($1)
                 ORDER BY updated_at DESC
                 LIMIT 20",
                &[&github_username],
            )
            .await?;

        let recent_issues: Vec<_> = issues.iter().map(|r| CachedIssueShort {
            issue_id: r.get(0),
            repo: format!("{}/{}", r.get::<_, String>(1), r.get::<_, String>(2)),
            title: r.get(3),
            state: r.get(4),
            labels: r.get(5),
            updated_at: r.get(6),
            issue_url: r.get(7),
        }).collect();

        Ok(Some(HotkeyDetails {
            hotkey: hotkey.to_string(),
            github_username,
            registered_at,
            valid_issues: balance.valid_count,
            invalid_issues: balance.invalid_count,
            balance: balance.balance,
            is_penalized: balance.is_penalized,
            weight,
            recent_issues,
        }))
    }

    /// Get GitHub user details by username
    pub async fn get_github_user_details(&self, username: &str) -> Result<Option<GitHubUserDetails>> {
        let client = self.pool.get().await?;

        // Get registration info
        let reg = client
            .query_opt(
                "SELECT hotkey, registered_at FROM github_registrations WHERE LOWER(github_username) = LOWER($1)",
                &[&username],
            )
            .await?;

        let (hotkey, registered_at): (Option<String>, Option<DateTime<Utc>>) = match reg {
            Some(r) => (Some(r.get(0)), Some(r.get(1))),
            None => (None, None),
        };

        // Count issues from this user
        let stats = client
            .query_one(
                "SELECT 
                    COUNT(*) as total,
                    COUNT(*) FILTER (WHERE 'valid' = ANY(labels)) as valid_count,
                    COUNT(*) FILTER (WHERE 'invalid' = ANY(labels)) as invalid_count,
                    COUNT(*) FILTER (WHERE state = 'open') as open_count
                 FROM github_issues 
                 WHERE LOWER(github_username) = LOWER($1)",
                &[&username],
            )
            .await?;

        let total: i64 = stats.get(0);
        if total == 0 && hotkey.is_none() {
            return Ok(None);
        }

        // Get recent issues
        let issues = client
            .query(
                "SELECT issue_id, repo_owner, repo_name, title, state, labels, updated_at, issue_url
                 FROM github_issues 
                 WHERE LOWER(github_username) = LOWER($1)
                 ORDER BY updated_at DESC
                 LIMIT 20",
                &[&username],
            )
            .await?;

        let recent_issues: Vec<_> = issues.iter().map(|r| CachedIssueShort {
            issue_id: r.get(0),
            repo: format!("{}/{}", r.get::<_, String>(1), r.get::<_, String>(2)),
            title: r.get(3),
            state: r.get(4),
            labels: r.get(5),
            updated_at: r.get(6),
            issue_url: r.get(7),
        }).collect();

        Ok(Some(GitHubUserDetails {
            github_username: username.to_string(),
            hotkey,
            registered_at,
            total_issues: total as i32,
            valid_issues: stats.get::<_, i64>(1) as i32,
            invalid_issues: stats.get::<_, i64>(2) as i32,
            open_issues: stats.get::<_, i64>(3) as i32,
            recent_issues,
        }))
    }

    /// Get sync status for all repos
    pub async fn get_sync_status(&self) -> Result<Vec<SyncStatus>> {
        let client = self.pool.get().await?;

        let rows = client
            .query(
                "SELECT repo_owner, repo_name, last_sync_at, last_issue_updated_at, issues_synced
                 FROM github_sync_state
                 ORDER BY last_sync_at DESC",
                &[],
            )
            .await?;

        Ok(rows.iter().map(|r| SyncStatus {
            repo_owner: r.get(0),
            repo_name: r.get(1),
            last_sync_at: r.get(2),
            last_issue_updated_at: r.get(3),
            issues_synced: r.get(4),
        }).collect())
    }
}

// ============================================================================
// DATA STRUCTURES FOR ISSUES API
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct CachedIssue {
    pub issue_id: i64,
    pub repo_owner: String,
    pub repo_name: String,
    pub github_username: String,
    pub title: String,
    pub state: String,
    pub labels: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub issue_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CachedIssueShort {
    pub issue_id: i64,
    pub repo: String,
    pub title: String,
    pub state: String,
    pub labels: Vec<String>,
    pub updated_at: DateTime<Utc>,
    pub issue_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IssuesStats {
    pub total: i32,
    pub open: i32,
    pub closed: i32,
    pub valid: i32,
    pub invalid: i32,
    pub pending: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct HotkeyDetails {
    pub hotkey: String,
    pub github_username: String,
    pub registered_at: DateTime<Utc>,
    pub valid_issues: i32,
    pub invalid_issues: i32,
    pub balance: i32,
    pub is_penalized: bool,
    pub weight: f64,
    pub recent_issues: Vec<CachedIssueShort>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GitHubUserDetails {
    pub github_username: String,
    pub hotkey: Option<String>,
    pub registered_at: Option<DateTime<Utc>>,
    pub total_issues: i32,
    pub valid_issues: i32,
    pub invalid_issues: i32,
    pub open_issues: i32,
    pub recent_issues: Vec<CachedIssueShort>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncStatus {
    pub repo_owner: String,
    pub repo_name: String,
    pub last_sync_at: DateTime<Utc>,
    pub last_issue_updated_at: Option<DateTime<Utc>>,
    pub issues_synced: i32,
}

// ============================================================================
// WEIGHT CALCULATION (standalone function)
// ============================================================================

/// Label change detection result
#[derive(Debug, Clone, PartialEq)]
pub enum LabelChange {
    None,
    BecameValid,
    BecameInvalid,
    LostValid,
}

#[derive(Debug, Clone, Serialize)]
pub struct StarTargetRepo {
    pub owner: String,
    pub repo: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StarStats {
    pub total_stars: i32,
    pub users_with_stars: i32,
    pub users_with_bonus: i32,
}

/// Calculate weight based on points
/// 
/// Points system:
/// - cortex: 5 points per issue
/// - term-challenge: 1 point per issue
/// - vgrep: 1 point per issue
/// - 100 points = 100% weight (capped)
/// 
/// Formula: weight = min(points * 0.01, 1.0)
pub fn calculate_weight_from_points(points: f64) -> f64 {
    (points * WEIGHT_PER_POINT).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weight_from_points_basic() {
        // 1 point = 1%
        assert!((calculate_weight_from_points(1.0) - 0.01).abs() < 0.0001);
        
        // 10 points = 10%
        assert!((calculate_weight_from_points(10.0) - 0.10).abs() < 0.0001);
        
        // 50 points = 50%
        assert!((calculate_weight_from_points(50.0) - 0.50).abs() < 0.0001);
    }

    #[test]
    fn test_weight_from_points_cortex() {
        // 7 cortex issues = 7 * 5 = 35 points = 35%
        let cortex_points = 7.0 * 5.0;
        assert!((calculate_weight_from_points(cortex_points) - 0.35).abs() < 0.0001);
        
        // 20 cortex issues = 20 * 5 = 100 points = 100%
        let cortex_points = 20.0 * 5.0;
        assert!((calculate_weight_from_points(cortex_points) - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_weight_from_points_vgrep() {
        // 7 vgrep issues = 7 * 1 = 7 points = 7%
        let vgrep_points = 7.0 * 1.0;
        assert!((calculate_weight_from_points(vgrep_points) - 0.07).abs() < 0.0001);
        
        // 100 vgrep issues = 100 * 1 = 100 points = 100%
        let vgrep_points = 100.0 * 1.0;
        assert!((calculate_weight_from_points(vgrep_points) - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_weight_from_points_max_cap() {
        // 200 points should still be capped at 100%
        assert!((calculate_weight_from_points(200.0) - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_weight_from_points_zero() {
        assert_eq!(calculate_weight_from_points(0.0), 0.0);
    }
}
