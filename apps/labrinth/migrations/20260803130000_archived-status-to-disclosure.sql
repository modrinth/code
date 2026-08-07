INSERT INTO project_disclosures (project_id, type, metadata, updated_by, set_by_moderator)
SELECT id, 'archived', '{"note": null}'::jsonb, 0, FALSE
FROM mods
WHERE status = 'archived'
ON CONFLICT (project_id, type) DO NOTHING;

UPDATE mods
SET status = 'approved'
WHERE status = 'archived';
