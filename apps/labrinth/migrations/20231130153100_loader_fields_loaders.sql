-- Adds missing loader_fields_loaders entries for mrpack loader
INSERT INTO loader_fields_loaders
SELECT l.id, lf.id FROM loaders l CROSS JOIN loader_fields lf
WHERE l.loader='mrpack' AND lf.field=ANY(ARRAY['game_versions','client_and_server','server_only','client_only','singleplayer'])
ON CONFLICT DO NOTHING;