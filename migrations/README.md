# Database Migrations

This directory contains SQLite migrations for the bounty-challenge database.

## Migration Files

Migrations are numbered sequentially and run in order:

- `001_initial.sql` - Base schema (registrations, bounties, indexes)

## Running Migrations

Migrations are applied automatically when the server starts via `BountyStorage::new()`.

The migration runner:
1. Creates a `schema_migrations` table to track applied migrations
2. Reads all `.sql` files from this directory
3. Sorts by filename (numeric prefix)
4. Runs each migration that hasn't been applied
5. Records the migration version and timestamp

## Creating New Migrations

1. Create a new file: `NNN_description.sql` where NNN is the next number
2. Write idempotent SQL (use `IF NOT EXISTS`, etc.)
3. Add comments explaining the purpose
4. Test locally before deploying

Example:
```sql
-- 002_add_issue_title.sql
-- Add title column for better display

ALTER TABLE validated_bounties ADD COLUMN issue_title TEXT;
```

## Schema Overview

### miner_registrations
Links miner hotkeys to GitHub usernames.

| Column | Type | Description |
|--------|------|-------------|
| miner_hotkey | TEXT | Primary key, SS58 address |
| github_username | TEXT | GitHub username |
| registered_at | TEXT | ISO 8601 timestamp |

### validated_bounties
Records claimed and validated bounties.

| Column | Type | Description |
|--------|------|-------------|
| issue_number | INTEGER | Primary key, GitHub issue number |
| github_username | TEXT | Issue author's GitHub username |
| miner_hotkey | TEXT | Claiming miner's hotkey |
| validated_at | TEXT | ISO 8601 timestamp |
| issue_url | TEXT | Full GitHub issue URL |

### schema_migrations
Tracks applied migrations.

| Column | Type | Description |
|--------|------|-------------|
| version | INTEGER | Primary key, migration number |
| name | TEXT | Migration filename |
| applied_at | TEXT | ISO 8601 timestamp |

## Manual Operations

### Check Applied Migrations
```bash
sqlite3 bounty.db "SELECT * FROM schema_migrations ORDER BY version;"
```

### View Bounty Stats
```bash
sqlite3 bounty.db "
SELECT miner_hotkey, COUNT(*) as bounties 
FROM validated_bounties 
GROUP BY miner_hotkey 
ORDER BY bounties DESC;
"
```

### Export Data
```bash
sqlite3 -header -csv bounty.db "SELECT * FROM validated_bounties;" > bounties.csv
```
