-- Migration 005: Repo-specific reward multipliers
-- Different repos give different points based on complexity/importance

-- ============================================================================
-- ADD MULTIPLIER COLUMN TO TARGET REPOS
-- ============================================================================
ALTER TABLE target_repos ADD COLUMN IF NOT EXISTS multiplier REAL NOT NULL DEFAULT 1.0;

-- Set multipliers for each repo
UPDATE target_repos SET multiplier = 4.0 WHERE owner = 'CortexLM' AND repo = 'cortex';
UPDATE target_repos SET multiplier = 1.0 WHERE owner = 'CortexLM' AND repo = 'vgrep';
UPDATE target_repos SET multiplier = 0.5 WHERE owner = 'PlatformNetwork' AND repo = 'platform';
UPDATE target_repos SET multiplier = 0.5 WHERE owner = 'PlatformNetwork' AND repo = 'term-challenge';
UPDATE target_repos SET multiplier = 1.0 WHERE owner = 'PlatformNetwork' AND repo = 'bounty-challenge';

-- Insert missing repos with their multipliers
INSERT INTO target_repos (owner, repo, multiplier) VALUES 
    ('CortexLM', 'cortex', 4.0),
    ('CortexLM', 'vgrep', 1.0)
ON CONFLICT (owner, repo) DO UPDATE SET multiplier = EXCLUDED.multiplier;

-- ============================================================================
-- ADD MULTIPLIER COLUMN TO RESOLVED ISSUES (for historical tracking)
-- ============================================================================
ALTER TABLE resolved_issues ADD COLUMN IF NOT EXISTS multiplier REAL NOT NULL DEFAULT 1.0;

-- ============================================================================
-- UPDATE CURRENT WEIGHTS VIEW (include multiplier in calculation)
-- ============================================================================
DROP VIEW IF EXISTS current_weights CASCADE;

CREATE OR REPLACE VIEW current_weights AS
WITH recent_issues AS (
    SELECT 
        r.github_username,
        r.hotkey,
        SUM(r.multiplier) as weighted_issues_24h,
        COUNT(*) as issues_resolved_24h
    FROM resolved_issues r
    WHERE r.resolved_at >= NOW() - INTERVAL '24 hours'
      AND r.hotkey IS NOT NULL
    GROUP BY r.github_username, r.hotkey
),
total_stats AS (
    SELECT 
        SUM(multiplier) as total_weighted_24h,
        COUNT(*) as total_issues_24h 
    FROM resolved_issues 
    WHERE resolved_at >= NOW() - INTERVAL '24 hours'
),
star_bonus AS (
    SELECT github_username, star_bonus
    FROM user_star_bonus
    WHERE star_bonus > 0
)
SELECT 
    r.github_username,
    r.hotkey,
    r.issues_resolved_24h,
    COALESCE(t.total_issues_24h, 0) as total_issues_24h,
    -- Weight calculation with multiplier:
    -- Base weight per issue = 0.01, but now multiplied by repo multiplier
    LEAST(
        r.weighted_issues_24h * 
        CASE 
            WHEN COALESCE(t.total_weighted_24h, 0) > 100 THEN 0.01 * (100.0 / t.total_weighted_24h)
            ELSE 0.01
        END,
        LEAST(COALESCE(t.total_weighted_24h, 0) / 250.0, 1.0)
    ) + COALESCE(sb.star_bonus, 0) as weight,
    false as is_penalized
FROM recent_issues r
CROSS JOIN total_stats t
LEFT JOIN star_bonus sb ON LOWER(r.github_username) = sb.github_username
ORDER BY weight DESC;

-- Record migration
INSERT INTO schema_migrations (version, name) VALUES (5, 'repo_multipliers')
ON CONFLICT DO NOTHING;
