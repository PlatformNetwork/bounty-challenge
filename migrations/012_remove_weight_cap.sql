-- Migration 012: Remove per-user weight cap
-- Weight is now raw points * 0.02, normalized at API level
-- This ensures users with more points always have proportionally higher weight

-- ============================================================================
-- UPDATE CURRENT WEIGHTS VIEW - REMOVE CAP
-- ============================================================================
DROP VIEW IF EXISTS current_weights CASCADE;

CREATE OR REPLACE VIEW current_weights AS
WITH recent_valid AS (
    SELECT 
        r.github_username,
        r.hotkey,
        COUNT(*) as valid_count  -- 1 point per issue (flat rate)
    FROM resolved_issues r
    WHERE r.resolved_at >= NOW() - INTERVAL '24 hours'
      AND r.hotkey IS NOT NULL
    GROUP BY r.github_username, r.hotkey
),
recent_invalid AS (
    SELECT 
        i.github_username,
        i.hotkey,
        COUNT(*) * 2 as invalid_count  -- 2 penalty per invalid issue
    FROM invalid_issues i
    WHERE i.recorded_at >= NOW() - INTERVAL '24 hours'
      AND i.hotkey IS NOT NULL
    GROUP BY i.github_username, i.hotkey
),
user_stars AS (
    SELECT 
        LOWER(github_username) as github_username,
        COUNT(*) * 0.25 as star_points  -- 0.25 points per starred repo
    FROM github_stars
    GROUP BY LOWER(github_username)
),
user_net_points AS (
    SELECT 
        COALESCE(v.github_username, inv.github_username) as github_username,
        COALESCE(v.hotkey, inv.hotkey) as hotkey,
        COALESCE(v.valid_count, 0) as issues_resolved_24h,
        -- Net points = valid + stars - invalid
        COALESCE(v.valid_count, 0) + COALESCE(s.star_points, 0) - COALESCE(inv.invalid_count, 0) as net_points,
        COALESCE(inv.invalid_count, 0) as invalid_count
    FROM recent_valid v
    FULL OUTER JOIN recent_invalid inv ON v.hotkey = inv.hotkey
    LEFT JOIN user_stars s ON LOWER(COALESCE(v.github_username, inv.github_username)) = s.github_username
    WHERE COALESCE(v.hotkey, inv.hotkey) IS NOT NULL
),
admin_bonus AS (
    SELECT 
        hotkey,
        SUM(bonus_weight) as total_admin_bonus
    FROM admin_bonuses
    WHERE active = true AND expires_at > NOW()
    GROUP BY hotkey
)
SELECT 
    u.github_username,
    u.hotkey,
    u.issues_resolved_24h::bigint,
    (SELECT COUNT(*) FROM resolved_issues WHERE resolved_at >= NOW() - INTERVAL '24 hours')::int as total_issues_24h,
    -- Weight = net_points * 0.02 + admin bonus (NO CAP - normalized at API level)
    -- If net_points <= 0, only admin bonus applies
    CASE 
        WHEN u.net_points <= 0 THEN COALESCE(ab.total_admin_bonus, 0)
        ELSE u.net_points * 0.02 + COALESCE(ab.total_admin_bonus, 0)
    END as weight,
    u.net_points <= 0 as is_penalized
FROM user_net_points u
LEFT JOIN admin_bonus ab ON u.hotkey = ab.hotkey
ORDER BY weight DESC;

-- ============================================================================
-- SCHEMA MIGRATION RECORD
-- ============================================================================
INSERT INTO schema_migrations (version, name) VALUES (12, 'remove_weight_cap')
ON CONFLICT DO NOTHING;
