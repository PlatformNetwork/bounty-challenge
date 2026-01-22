-- Migration 010: Cleanup false invalid issues
-- Remove issues that were wrongly marked as invalid just because they were closed without 'valid' label
-- Only issues with the actual 'invalid' label from maintainers should be in this table

DELETE FROM invalid_issues 
WHERE reason = 'Closed without valid label';

-- Record this migration
INSERT INTO schema_migrations (version) VALUES (10) ON CONFLICT DO NOTHING;
