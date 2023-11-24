
-- Adds missing fields to loader_fields_loaders
INSERT INTO loader_fields_loaders (loader_id, loader_field_id) 
SELECT l.id, lf.id FROM loaders l CROSS JOIN loader_fields lf  WHERE lf.field = 'game_versions' 
AND l.loader = ANY( ARRAY['forge', 'fabric', 'quilt', 'modloader','rift','liteloader', 'neoforge'])
ON CONFLICT (loader_id, loader_field_id) DO NOTHING;

-- Fixes mrpack variants being added to the wrong enum
-- Luckily, mrpack variants are the only ones set to 2 without metadata
UPDATE loader_field_enum_values SET enum_id = 3 WHERE enum_id = 2 AND metadata IS NULL;

-- Because it was mislabeled, version_fields for mrpack_loaders were set to null.
-- 1) Update version_fields corresponding to mrpack_loaders to the correct enum_value
UPDATE version_fields vf
SET enum_value = subquery.lfev_id
FROM (
    SELECT vf.version_id, vf.field_id, lfev.id AS lfev_id
    FROM version_fields vf
    LEFT JOIN versions v ON v.id = vf.version_id
    LEFT JOIN loaders_versions lv ON v.id = lv.version_id
    LEFT JOIN loaders l ON l.id = lv.loader_id
    LEFT JOIN loader_fields lf ON lf.id = vf.field_id
    LEFT JOIN loader_field_enum_values lfev ON lfev.value = l.loader AND lf.enum_type = lfev.enum_id
    WHERE lf.field = 'mrpack_loaders' AND vf.enum_value IS NULL
) AS subquery
WHERE vf.version_id = subquery.version_id AND vf.field_id = subquery.field_id;

-- 2) Set those versions to mrpack as their version
INSERT INTO loaders_versions (version_id, loader_id)
SELECT DISTINCT vf.version_id, l.id
FROM version_fields vf
LEFT JOIN loader_fields lf ON lf.id = vf.field_id
CROSS JOIN loaders l
WHERE lf.field = 'mrpack_loaders'
AND l.loader = 'mrpack'
ON CONFLICT DO NOTHING;

-- 3) Delete the old versions that had mrpack added to them
DELETE FROM loaders_versions lv
WHERE lv.loader_id != (SELECT id FROM loaders WHERE loader = 'mrpack')
AND lv.version_id IN (
    SELECT version_id
    FROM loaders_versions
    WHERE loader_id = (SELECT id FROM loaders WHERE loader = 'mrpack')
);
