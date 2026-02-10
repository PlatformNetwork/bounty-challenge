-- Migration 014: Dynamic Penalty System
-- Changes penalty from fixed 2-point per invalid issue to dynamic:
-- - No penalty if invalid_count <= valid_count
-- - Penalty = (invalid_count - valid_count) when invalid_count > valid_count

-- Drop and recreate current_weights view with dynamic penalty logic
DROP VIEW IF EXISTS current_weights;

CREATE OR REPLACE VIEW current_weights AS
WITH recent_valid AS (
    SELECT 
        github_username,
        hotkey,
        COUNT(*) as issues_resolved_24h,
        COUNT(*) as valid_count
    FROM resolved_issues
    WHERE resolved_at >= NOW() - INTERVAL '24 hours'
      AND hotkey IS NOT NULL
    GROUP BY github_username, hotkey
),
recent_invalid AS (
    SELECT 
        hotkey,
        COUNT(*) as invalid_count
    FROM invalid_issues
    WHERE recorded_at >= NOW() - INTERVAL '24 hours'
      AND hotkey IS NOT NULL
    GROUP BY hotkey
),
total_stats AS (
    SELECT COUNT(*) as total_issues_24h 
    FROM resolved_issues 
    WHERE resolved_at >= NOW() - INTERVAL '24 hours'
),
penalty_status AS (
    SELECT hotkey, is_penalized
    FROM user_balance
)
SELECT 
    r.github_username,
    r.hotkey,
    r.issues_resolved_24h,
    t.total_issues_24h,
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

-- Update user_balance view with dynamic penalty logic
DROP VIEW IF EXISTS user_balance;

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
    -- Dynamic balance: valid - max(0, invalid - valid) = valid if invalid <= valid, else 2*valid - invalid
    COALESCE(v.valid_count, 0) - GREATEST(0, COALESCE(i.invalid_count, 0) - COALESCE(v.valid_count, 0)) as balance,
    CASE 
        -- Penalized only if dynamic penalty makes balance negative
        WHEN COALESCE(v.valid_count, 0) - GREATEST(0, COALESCE(i.invalid_count, 0) - COALESCE(v.valid_count, 0)) < 0 THEN true
        ELSE false
    END as is_penalized
FROM valid_counts v
FULL OUTER JOIN invalid_counts i ON v.hotkey = i.hotkey;

-- Record this migration
INSERT INTO schema_migrations (version) VALUES (14) ON CONFLICT DO NOTHING;
