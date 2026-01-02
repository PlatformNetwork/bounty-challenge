-- Initial schema for bounty-challenge

CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    applied_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS miner_registrations (
    miner_hotkey TEXT PRIMARY KEY,
    github_username TEXT NOT NULL,
    registered_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS validated_bounties (
    issue_number INTEGER PRIMARY KEY,
    github_username TEXT NOT NULL,
    miner_hotkey TEXT NOT NULL,
    validated_at TEXT NOT NULL,
    issue_url TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_bounties_miner ON validated_bounties(miner_hotkey);
CREATE INDEX IF NOT EXISTS idx_bounties_github ON validated_bounties(github_username);
