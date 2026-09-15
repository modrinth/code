INSERT INTO project_disclosures (project_id, type, metadata, updated_at, updated_by, set_by_moderator, deleted_at, lock_status)
SELECT
	project_id,
	'ai_functionality',
	'{"note":null}'::jsonb,
	now(),
	0,
	set_by_moderator,
	deleted_at,
	lock_status
FROM project_disclosures
WHERE type = 'ai_content'
	AND metadata -> 'uses' ? 'functionality'
ON CONFLICT (project_id, type) DO NOTHING;

DELETE FROM project_disclosures
WHERE type = 'ai_content'
	AND metadata -> 'uses' ? 'functionality'
	AND jsonb_array_length(metadata -> 'uses') = 1;

UPDATE project_disclosures
SET metadata = jsonb_set(metadata, '{uses}', (metadata -> 'uses') - 'functionality')
WHERE type = 'ai_content'
	AND metadata -> 'uses' ? 'functionality';
