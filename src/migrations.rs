//! Database migrations system

use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};
use std::fs;
use std::path::Path;
use tracing::{debug, info};

#[derive(Debug)]
struct Migration {
    version: u32,
    name: String,
    sql: String,
}

pub struct Migrator {
    migrations_dir: &'static str,
}

impl Default for Migrator {
    fn default() -> Self {
        Self::new()
    }
}

impl Migrator {
    pub fn new() -> Self {
        Self {
            migrations_dir: concat!(env!("CARGO_MANIFEST_DIR"), "/migrations"),
        }
    }

    pub fn run(&self, conn: &Connection) -> Result<()> {
        self.ensure_migrations_table(conn)?;

        let applied = self.get_applied_versions(conn)?;
        let migrations = self.load_migrations()?;

        for migration in migrations {
            if applied.contains(&migration.version) {
                debug!("Migration {} already applied", migration.name);
                continue;
            }

            info!("Applying migration: {}", migration.name);
            self.apply_migration(conn, &migration)?;
        }

        Ok(())
    }

    fn ensure_migrations_table(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    fn get_applied_versions(&self, conn: &Connection) -> Result<Vec<u32>> {
        let mut stmt = conn.prepare("SELECT version FROM schema_migrations ORDER BY version")?;
        let versions = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<u32>, _>>()?;
        Ok(versions)
    }

    fn load_migrations(&self) -> Result<Vec<Migration>> {
        let dir = Path::new(self.migrations_dir);

        if !dir.exists() {
            return Ok(vec![]);
        }

        let mut migrations = Vec::new();
        let mut entries: Vec<_> = fs::read_dir(dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "sql")
                    .unwrap_or(false)
            })
            .collect();

        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let filename = entry.file_name();
            let name = filename.to_string_lossy();

            let version = name
                .split('_')
                .next()
                .and_then(|v| v.parse::<u32>().ok())
                .context(format!("Invalid migration filename: {}", name))?;

            let sql = fs::read_to_string(entry.path())
                .context(format!("Failed to read migration: {}", name))?;

            migrations.push(Migration {
                version,
                name: name.to_string(),
                sql,
            });
        }

        Ok(migrations)
    }

    fn apply_migration(&self, conn: &Connection, migration: &Migration) -> Result<()> {
        conn.execute_batch(&migration.sql)
            .context(format!("Failed to apply migration: {}", migration.name))?;

        conn.execute(
            "INSERT INTO schema_migrations (version, name, applied_at) VALUES (?1, ?2, ?3)",
            params![migration.version, migration.name, Utc::now().to_rfc3339()],
        )?;

        info!("Applied migration: {}", migration.name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrator_in_memory() {
        let conn = Connection::open_in_memory().unwrap();
        let migrator = Migrator::new();
        migrator.run(&conn).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert!(count >= 1);
    }
}
