//! Local storage for bounty tracking

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;

use crate::migrations::Migrator;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedBounty {
    pub issue_number: u32,
    pub github_username: String,
    pub miner_hotkey: String,
    pub validated_at: DateTime<Utc>,
    pub issue_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerScore {
    pub miner_hotkey: String,
    pub github_username: String,
    pub valid_issues_count: u32,
    pub last_updated: DateTime<Utc>,
}

pub struct BountyStorage {
    conn: Mutex<Connection>,
}

impl BountyStorage {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path)?;
        let storage = Self {
            conn: Mutex::new(conn),
        };
        storage.run_migrations()?;
        Ok(storage)
    }

    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let storage = Self {
            conn: Mutex::new(conn),
        };
        storage.run_migrations()?;
        Ok(storage)
    }

    fn run_migrations(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        Migrator::new().run(&conn)
    }

    pub fn register_miner(&self, hotkey: &str, github_username: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO miner_registrations (miner_hotkey, github_username, registered_at) VALUES (?1, ?2, ?3)",
            params![hotkey, github_username, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn get_github_username(&self, hotkey: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT github_username FROM miner_registrations WHERE miner_hotkey = ?1",
        )?;
        let result = stmt.query_row(params![hotkey], |row| row.get(0)).ok();
        Ok(result)
    }

    pub fn get_hotkey_by_github(&self, github_username: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT miner_hotkey FROM miner_registrations WHERE LOWER(github_username) = LOWER(?1)",
        )?;
        let result = stmt.query_row(params![github_username], |row| row.get(0)).ok();
        Ok(result)
    }

    pub fn record_bounty(&self, bounty: &ValidatedBounty) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO validated_bounties (issue_number, github_username, miner_hotkey, validated_at, issue_url) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                bounty.issue_number,
                bounty.github_username,
                bounty.miner_hotkey,
                bounty.validated_at.to_rfc3339(),
                bounty.issue_url,
            ],
        )?;
        Ok(())
    }

    pub fn is_issue_claimed(&self, issue_number: u32) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM validated_bounties WHERE issue_number = ?1",
            params![issue_number],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn get_miner_bounties(&self, hotkey: &str) -> Result<Vec<ValidatedBounty>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT issue_number, github_username, miner_hotkey, validated_at, issue_url 
             FROM validated_bounties WHERE miner_hotkey = ?1 ORDER BY validated_at DESC",
        )?;

        let bounties = stmt
            .query_map(params![hotkey], |row| {
                Ok(ValidatedBounty {
                    issue_number: row.get(0)?,
                    github_username: row.get(1)?,
                    miner_hotkey: row.get(2)?,
                    validated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .unwrap()
                        .with_timezone(&Utc),
                    issue_url: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(bounties)
    }

    pub fn get_all_scores(&self) -> Result<Vec<MinerScore>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT 
                miner_hotkey,
                github_username,
                COUNT(*) as valid_count,
                MAX(validated_at) as last_updated
            FROM validated_bounties
            GROUP BY miner_hotkey
            ORDER BY valid_count DESC
            "#,
        )?;

        let scores = stmt
            .query_map([], |row| {
                Ok(MinerScore {
                    miner_hotkey: row.get(0)?,
                    github_username: row.get(1)?,
                    valid_issues_count: row.get(2)?,
                    last_updated: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .unwrap()
                        .with_timezone(&Utc),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(scores)
    }

    pub fn get_total_bounties(&self) -> Result<u32> {
        let conn = self.conn.lock().unwrap();
        let count: u32 = conn.query_row(
            "SELECT COUNT(*) FROM validated_bounties",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_in_memory() {
        let storage = BountyStorage::in_memory().unwrap();
        
        storage.register_miner("hotkey1", "github_user").unwrap();
        let username = storage.get_github_username("hotkey1").unwrap();
        assert_eq!(username, Some("github_user".to_string()));
    }

    #[test]
    fn test_record_bounty() {
        let storage = BountyStorage::in_memory().unwrap();
        
        let bounty = ValidatedBounty {
            issue_number: 123,
            github_username: "testuser".to_string(),
            miner_hotkey: "hotkey1".to_string(),
            validated_at: Utc::now(),
            issue_url: "https://github.com/test/repo/issues/123".to_string(),
        };

        storage.record_bounty(&bounty).unwrap();
        assert!(storage.is_issue_claimed(123).unwrap());
        assert!(!storage.is_issue_claimed(999).unwrap());
    }
}
