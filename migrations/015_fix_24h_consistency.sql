-- Migration 015: Fix 24h Consistency in Weight Calculation
-- 
-- This migration fixes critical inconsistencies between SQL views and Rust code:
-- 1. user_balance view now uses 24h windows for valid/invalid counts (was all-time)
-- 2. current_weights view uses 24h-based penalty calculation (not all-time user_balance)
-- 3. Weight formula aligned with Rust: points * 0.02 (was 0.01)
-- 4. Star bonus (0.25 per star) included in current_weights
-- 5. Duplicate penalty (0.5 per duplicate) included in current_weights
--
-- Formula (matching Rust calculate_user_weight):
--   net_points = valid_count + star_points - invalid_penalty - duplicate_penalty
--   weight = net_points * 0.02 (WEIGHT_PER_POINT constant)
--   invalid_penalty = max(0, invalid_count - valid_count) [dynamic penalty]
--   duplicate_penalty = duplicate_count * 0.5

-- ============================================================================
-- DROP AND RECREATE USER_BALANCE VIEW (now 24h based)
-- ============================================================================
DROP VIEW IF EXISTS user_balance CASCADE;

CREATE OR REPLACE VIEW user_balance AS
WITH valid_counts_24h AS (
    SELECT 
        hotkey,
        github_username,
        COUNT(*) as valid_count
    FROM resolved_issues
    WHERE hotkey IS NOT NULL
      AND resolved_at >= NOW() - INTERVAL '24 hours'
    GROUP BY hotkey, github_username
),
invalid_counts_24h AS (
    SELECT 
        hotkey,
        github_username,
        COUNT(*) as invalid_count
    FROM invalid_issues
    WHERE hotkey IS NOT NULL
      AND recorded_at >= NOW() - INTERVAL '24 hours'
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
FROM valid_counts_24h v
FULL OUTER JOIN invalid_counts_24h i ON v.hotkey = i.hotkey;

-- ============================================================================
-- DROP AND RECREATE CURRENT_WEIGHTS VIEW (aligned with Rust formula)
-- ============================================================================
DROP VIEW IF EXISTS current_weights CASCADE;

CREATE OR REPLACE VIEW current_weights AS
WITH recent_valid AS (
    -- Valid issues in last 24h: 1 point per issue
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
    -- Invalid issues in last 24h (for dynamic penalty)
    SELECT 
        hotkey,
        COUNT(*) as invalid_count
    FROM invalid_issues
    WHERE recorded_at >= NOW() - INTERVAL '24 hours'
      AND hotkey IS NOT NULL
    GROUP BY hotkey
),
recent_duplicate AS (
    -- Duplicate issues in last 24h: 0.5 penalty per duplicate
    SELECT 
        hotkey,
        COUNT(*) * 0.5 as duplicate_penalty
    FROM duplicate_issues
    WHERE recorded_at >= NOW() - INTERVAL '24 hours'
      AND hotkey IS NOT NULL
    GROUP BY hotkey
),
user_stars AS (
    -- Star bonus: 0.25 points per starred repo (no time limit on stars)
    SELECT 
        LOWER(github_username) as github_username,
        COUNT(*) * 0.25 as star_points
    FROM github_stars
    GROUP BY LOWER(github_username)
),
total_stats AS (
    SELECT COUNT(*) as total_issues_24h 
    FROM resolved_issues 
    WHERE resolved_at >= NOW() - INTERVAL '24 hours'
),
admin_bonus AS (
    -- Admin bonuses (active and not expired)
    SELECT 
        hotkey,
        SUM(bonus_weight) as total_admin_bonus
    FROM admin_bonuses
    WHERE active = true AND expires_at > NOW()
    GROUP BY hotkey
),
user_net_points AS (
    -- Calculate net points for each user
    SELECT 
        v.github_username,
        v.hotkey,
        v.issues_resolved_24h,
        v.valid_count,
        COALESCE(i.invalid_count, 0) as invalid_count,
        COALESCE(d.duplicate_penalty, 0) as duplicate_penalty,
        COALESCE(s.star_points, 0) as star_points,
        -- Dynamic invalid penalty: max(0, invalid_count - valid_count)
        GREATEST(0, COALESCE(i.invalid_count, 0) - v.valid_count) as invalid_penalty,
        -- Net points = valid + stars - invalid_penalty - duplicate_penalty
        v.valid_count + COALESCE(s.star_points, 0) 
            - GREATEST(0, COALESCE(i.invalid_count, 0) - v.valid_count)
            - COALESCE(d.duplicate_penalty, 0) as net_points
    FROM recent_valid v
    LEFT JOIN recent_invalid i ON v.hotkey = i.hotkey
    LEFT JOIN recent_duplicate d ON v.hotkey = d.hotkey
    LEFT JOIN user_stars s ON LOWER(v.github_username) = s.github_username
)
SELECT 
    u.github_username,
    u.hotkey,
    u.issues_resolved_24h,
    COALESCE(t.total_issues_24h, 0) as total_issues_24h,
    -- Weight = net_points * 0.02 + admin bonus
    -- Matches Rust WEIGHT_PER_POINT = 0.02
    -- If net_points <= 0, only admin bonus applies (can be 0)
    CASE 
        WHEN u.net_points <= 0 THEN GREATEST(0, COALESCE(ab.total_admin_bonus, 0))
        ELSE u.net_points * 0.02 + COALESCE(ab.total_admin_bonus, 0)
    END as weight,
    -- Penalized if net_points <= 0
    u.net_points <= 0 as is_penalized
FROM user_net_points u
CROSS JOIN total_stats t
LEFT JOIN admin_bonus ab ON u.hotkey = ab.hotkey
ORDER BY weight DESC;

-- ============================================================================
-- RECORD MIGRATION
-- ============================================================================
INSERT INTO schema_migrations (version) VALUES (15) ON CONFLICT DO NOTHING;
