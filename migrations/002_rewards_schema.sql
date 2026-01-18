-- Migration 002: Rewards Schema for Cortex Issue Bounties
-- Supports multi-repo tracking and adaptive weight calculation

-- ============================================================================
-- GITHUB REGISTRATIONS (username <-> hotkey mapping)
-- ============================================================================
CREATE TABLE IF NOT EXISTS github_registrations (
    id SERIAL PRIMARY KEY,
    github_username TEXT NOT NULL UNIQUE,
    hotkey TEXT NOT NULL UNIQUE,
    registered_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_registrations_username ON github_registrations(LOWER(github_username));
CREATE INDEX IF NOT EXISTS idx_registrations_hotkey ON github_registrations(hotkey);

-- ============================================================================
-- TARGET REPOSITORIES (multi-repo support)
-- ============================================================================
CREATE TABLE IF NOT EXISTS target_repos (
    id SERIAL PRIMARY KEY,
    owner TEXT NOT NULL,
    repo TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(owner, repo)
);

-- Insert default repos
INSERT INTO target_repos (owner, repo) VALUES 
    ('CortexLM', 'cortex'),
    ('CortexLM', 'fabric')
ON CONFLICT DO NOTHING;

-- ============================================================================
-- RESOLVED ISSUES (track each validated issue)
-- ============================================================================
CREATE TABLE IF NOT EXISTS resolved_issues (
    id SERIAL PRIMARY KEY,
    issue_id BIGINT NOT NULL,
    repo_owner TEXT NOT NULL,
    repo_name TEXT NOT NULL,
    github_username TEXT NOT NULL,
    hotkey TEXT,
    issue_url TEXT NOT NULL,
    issue_title TEXT,
    resolved_at TIMESTAMPTZ NOT NULL,
    weight_attributed REAL NOT NULL DEFAULT 0.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_owner, repo_name, issue_id)
);

CREATE INDEX IF NOT EXISTS idx_resolved_username ON resolved_issues(LOWER(github_username));
CREATE INDEX IF NOT EXISTS idx_resolved_hotkey ON resolved_issues(hotkey);
CREATE INDEX IF NOT EXISTS idx_resolved_at ON resolved_issues(resolved_at);
CREATE INDEX IF NOT EXISTS idx_resolved_repo ON resolved_issues(repo_owner, repo_name);

-- ============================================================================
-- REWARD SNAPSHOTS (historical weight snapshots at instant T)
-- ============================================================================
CREATE TABLE IF NOT EXISTS reward_snapshots (
    id SERIAL PRIMARY KEY,
    snapshot_at TIMESTAMPTZ NOT NULL,
    github_username TEXT NOT NULL,
    hotkey TEXT NOT NULL,
    issues_resolved_24h INTEGER NOT NULL,
    total_issues_24h INTEGER NOT NULL,
    weight REAL NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_snapshots_at ON reward_snapshots(snapshot_at DESC);
CREATE INDEX IF NOT EXISTS idx_snapshots_hotkey ON reward_snapshots(hotkey);
CREATE INDEX IF NOT EXISTS idx_snapshots_username ON reward_snapshots(github_username);

-- ============================================================================
-- DAILY STATS (aggregated daily statistics)
-- ============================================================================
CREATE TABLE IF NOT EXISTS daily_stats (
    id SERIAL PRIMARY KEY,
    date DATE NOT NULL UNIQUE,
    total_issues_opened INTEGER NOT NULL DEFAULT 0,
    total_issues_resolved INTEGER NOT NULL DEFAULT 0,
    unique_contributors INTEGER NOT NULL DEFAULT 0,
    total_weight_distributed REAL NOT NULL DEFAULT 0.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_daily_stats_date ON daily_stats(date DESC);

-- ============================================================================
-- CURRENT WEIGHTS VIEW (computed from last 24h)
-- ============================================================================
CREATE OR REPLACE VIEW current_weights AS
WITH recent_issues AS (
    SELECT 
        github_username,
        hotkey,
        COUNT(*) as issues_resolved_24h
    FROM resolved_issues
    WHERE resolved_at >= NOW() - INTERVAL '24 hours'
      AND hotkey IS NOT NULL
    GROUP BY github_username, hotkey
),
total_stats AS (
    SELECT COUNT(*) as total_issues_24h 
    FROM resolved_issues 
    WHERE resolved_at >= NOW() - INTERVAL '24 hours'
)
SELECT 
    r.github_username,
    r.hotkey,
    r.issues_resolved_24h,
    t.total_issues_24h,
    -- Adaptive weight calculation:
    -- Base: 0.01 per issue, max 1.0 total weight
    -- If > 100 issues in 24h, weight per issue decreases proportionally
    LEAST(
        r.issues_resolved_24h * 
        CASE 
            WHEN t.total_issues_24h > 100 THEN 0.01 * (100.0 / t.total_issues_24h)
            ELSE 0.01
        END,
        LEAST(t.total_issues_24h / 250.0, 1.0)
    ) as weight
FROM recent_issues r
CROSS JOIN total_stats t
ORDER BY weight DESC;

-- ============================================================================
-- SCHEMA MIGRATIONS TABLE
-- ============================================================================
CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO schema_migrations (version, name) VALUES (2, 'rewards_schema')
ON CONFLICT DO NOTHING;
