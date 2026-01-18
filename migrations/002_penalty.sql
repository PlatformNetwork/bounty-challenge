-- Migration 003: Penalty System
-- Tracks invalid issues and applies penalties when invalid > valid

-- ============================================================================
-- INVALID ISSUES (track issues closed without 'valid' label)
-- ============================================================================
CREATE TABLE IF NOT EXISTS invalid_issues (
    id SERIAL PRIMARY KEY,
    issue_id BIGINT NOT NULL,
    repo_owner TEXT NOT NULL,
    repo_name TEXT NOT NULL,
    github_username TEXT NOT NULL,
    hotkey TEXT,
    issue_url TEXT NOT NULL,
    issue_title TEXT,
    reason TEXT,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_owner, repo_name, issue_id)
);

CREATE INDEX IF NOT EXISTS idx_invalid_username ON invalid_issues(LOWER(github_username));
CREATE INDEX IF NOT EXISTS idx_invalid_hotkey ON invalid_issues(hotkey);
CREATE INDEX IF NOT EXISTS idx_invalid_recorded_at ON invalid_issues(recorded_at);

-- ============================================================================
-- USER BALANCE VIEW (valid - invalid issues)
-- ============================================================================
CREATE OR REPLACE VIEW user_balance AS
WITH valid_counts AS (
    SELECT 
        hotkey,
        github_username,
        COUNT(*) as valid_count
    FROM resolved_issues
    WHERE hotkey IS NOT NULL
    GROUP BY hotkey, github_username
),
invalid_counts AS (
    SELECT 
        hotkey,
        github_username,
        COUNT(*) as invalid_count
    FROM invalid_issues
    WHERE hotkey IS NOT NULL
    GROUP BY hotkey, github_username
)
SELECT 
    COALESCE(v.hotkey, i.hotkey) as hotkey,
    COALESCE(v.github_username, i.github_username) as github_username,
    COALESCE(v.valid_count, 0) as valid_count,
    COALESCE(i.invalid_count, 0) as invalid_count,
    COALESCE(v.valid_count, 0) - COALESCE(i.invalid_count, 0) as balance,
    CASE 
        WHEN COALESCE(v.valid_count, 0) - COALESCE(i.invalid_count, 0) < 0 THEN true
        ELSE false
    END as is_penalized
FROM valid_counts v
FULL OUTER JOIN invalid_counts i ON v.hotkey = i.hotkey;

-- ============================================================================
-- UPDATED CURRENT WEIGHTS VIEW (with penalty system)
-- ============================================================================
DROP VIEW IF EXISTS current_weights;

CREATE OR REPLACE VIEW current_weights AS
WITH recent_valid AS (
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
),
-- Get penalty status from all-time balance
penalty_status AS (
    SELECT hotkey, is_penalized
    FROM user_balance
)
SELECT 
    r.github_username,
    r.hotkey,
    r.issues_resolved_24h,
    t.total_issues_24h,
    -- Apply penalty: if penalized, weight = 0
    CASE 
        WHEN COALESCE(p.is_penalized, false) = true THEN 0.0
        ELSE LEAST(
            r.issues_resolved_24h * 
            CASE 
                WHEN t.total_issues_24h > 100 THEN 0.01 * (100.0 / t.total_issues_24h)
                ELSE 0.01
            END,
            LEAST(t.total_issues_24h / 250.0, 1.0)
        )
    END as weight,
    COALESCE(p.is_penalized, false) as is_penalized
FROM recent_valid r
CROSS JOIN total_stats t
LEFT JOIN penalty_status p ON r.hotkey = p.hotkey
ORDER BY weight DESC;

-- ============================================================================
-- SCHEMA MIGRATION RECORD
-- ============================================================================
INSERT INTO schema_migrations (version, name) VALUES (3, 'penalty_system')
ON CONFLICT DO NOTHING;
