-- Migration 013: Duplicate Issues Tracking
-- Track issues marked with 'duplicate' label for 0.5 point penalty (less severe than invalid's 2.0 penalty)

-- ============================================================================
-- DUPLICATE ISSUES TABLE
-- ============================================================================
CREATE TABLE IF NOT EXISTS duplicate_issues (
    id SERIAL PRIMARY KEY,
    issue_id BIGINT NOT NULL,
    repo_owner TEXT NOT NULL,
    repo_name TEXT NOT NULL,
    github_username TEXT NOT NULL,
    hotkey TEXT,
    issue_url TEXT NOT NULL,
    issue_title TEXT,
    reason TEXT DEFAULT 'Marked as duplicate',
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_owner, repo_name, issue_id)
);

CREATE INDEX IF NOT EXISTS idx_duplicate_username ON duplicate_issues(LOWER(github_username));
CREATE INDEX IF NOT EXISTS idx_duplicate_hotkey ON duplicate_issues(hotkey);
CREATE INDEX IF NOT EXISTS idx_duplicate_recorded_at ON duplicate_issues(recorded_at);

-- Record this migration
INSERT INTO schema_migrations (version) VALUES (13) ON CONFLICT DO NOTHING;
