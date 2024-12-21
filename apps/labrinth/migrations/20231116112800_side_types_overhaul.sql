CREATE INDEX version_fields_version_id ON version_fields (version_id);
CREATE INDEX hashes_file_id ON hashes (file_id);

INSERT INTO loader_fields (field, field_type, optional) SELECT 'singleplayer', 'boolean', false;
INSERT INTO loader_fields (field, field_type, optional) SELECT 'client_and_server', 'boolean', false;
INSERT INTO loader_fields (field, field_type, optional) SELECT 'client_only', 'boolean', false;
INSERT INTO loader_fields (field, field_type, optional) SELECT 'server_only', 'boolean', false;

-- Create 4 temporary columns for the four booleans (makes queries easier)
ALTER TABLE versions ADD COLUMN singleplayer boolean;
ALTER TABLE versions ADD COLUMN client_and_server boolean;
ALTER TABLE versions ADD COLUMN client_only boolean;
ALTER TABLE versions ADD COLUMN server_only boolean;

-- Set singleplayer to be true if either client_side or server_side is 'required' OR 'optional'
UPDATE versions v SET singleplayer = true
FROM version_fields vf
INNER JOIN loader_fields lf ON vf.field_id = lf.id
INNER JOIN loader_field_enum_values lfev ON lf.enum_type = lfev.enum_id AND vf.enum_value = lfev.id
WHERE v.id = vf.version_id 
AND (lf.field = 'client_side' OR lf.field = 'server_side') AND (lfev.value = 'required' OR lfev.value = 'optional');

-- Set client and server to be true if either client_side or server_side is 'required' OR 'optional'
UPDATE versions v SET client_and_server = true
FROM version_fields vf
INNER JOIN loader_fields lf ON vf.field_id = lf.id
INNER JOIN loader_field_enum_values lfev ON lf.enum_type = lfev.enum_id AND vf.enum_value = lfev.id
WHERE v.id = vf.version_id 
AND (lf.field = 'client_side' OR lf.field = 'server_side') AND (lfev.value = 'required' OR lfev.value = 'optional');

-- -- Set client_only to be true if client_side is 'required' or 'optional', and server_side is 'optional', 'unsupported', or 'unknown'
UPDATE versions v
SET client_only = true
WHERE EXISTS (
    SELECT 1
    FROM version_fields vf
    INNER JOIN loader_fields lf ON vf.field_id = lf.id
    INNER JOIN loader_field_enum_values lfev ON lf.enum_type = lfev.enum_id AND vf.enum_value = lfev.id
    WHERE v.id = vf.version_id
    AND lf.field = 'client_side' AND (lfev.value = 'required' OR lfev.value = 'optional')
)
AND EXISTS (
    SELECT 1
    FROM version_fields vf2
    INNER JOIN loader_fields lf2 ON vf2.field_id = lf2.id
    INNER JOIN loader_field_enum_values lfev2 ON lf2.enum_type = lfev2.enum_id AND vf2.enum_value = lfev2.id
    WHERE v.id = vf2.version_id
    AND lf2.field = 'server_side' AND (lfev2.value = 'optional' OR lfev2.value = 'unsupported' OR lfev2.value = 'unknown')
);

-- -- Set server_only to be true if server_side is 'required' or 'optional', and client_side is 'optional', 'unsupported', or 'unknown'
UPDATE versions v
SET server_only = true
WHERE EXISTS (
    SELECT 1
    FROM version_fields vf
    INNER JOIN loader_fields lf ON vf.field_id = lf.id
    INNER JOIN loader_field_enum_values lfev ON lf.enum_type = lfev.enum_id AND vf.enum_value = lfev.id
    WHERE v.id = vf.version_id
    AND lf.field = 'server_side' AND (lfev.value = 'required' OR lfev.value = 'optional')
)
AND EXISTS (
    SELECT 1
    FROM version_fields vf2
    INNER JOIN loader_fields lf2 ON vf2.field_id = lf2.id
    INNER JOIN loader_field_enum_values lfev2 ON lf2.enum_type = lfev2.enum_id AND vf2.enum_value = lfev2.id
    WHERE v.id = vf2.version_id
    AND lf2.field = 'client_side' AND (lfev2.value = 'optional' OR lfev2.value = 'unsupported' OR lfev2.value = 'unknown')
);

-- Insert the values into the version_fields table
INSERT INTO version_fields (version_id, field_id, int_value) 
SELECT v.id, lf.id, CASE WHEN v.singleplayer THEN 1 ELSE 0 END
FROM versions v
INNER JOIN loader_fields lf ON lf.field = 'singleplayer';

INSERT INTO version_fields (version_id, field_id, int_value)
SELECT v.id, lf.id, CASE WHEN v.client_and_server THEN 1 ELSE 0 END
FROM versions v
INNER JOIN loader_fields lf ON lf.field = 'client_and_server';

INSERT INTO version_fields (version_id, field_id, int_value)
SELECT v.id, lf.id, CASE WHEN v.client_only THEN 1 ELSE 0 END
FROM versions v
INNER JOIN loader_fields lf ON lf.field = 'client_only';

INSERT INTO version_fields (version_id, field_id, int_value)
SELECT v.id, lf.id, CASE WHEN v.server_only THEN 1 ELSE 0 END
FROM versions v
INNER JOIN loader_fields lf ON lf.field = 'server_only';

-- Drop the temporary columns
ALTER TABLE versions DROP COLUMN singleplayer;
ALTER TABLE versions DROP COLUMN client_and_server;
ALTER TABLE versions DROP COLUMN client_only;
ALTER TABLE versions DROP COLUMN server_only;

-- For each loader where loader_fields_loaders is 'client_side' or 'server_side', add the new fields
INSERT INTO loader_fields_loaders (loader_id, loader_field_id) 
SELECT lfl.loader_id, lf.id
FROM loader_fields_loaders lfl
CROSS JOIN loader_fields lf
WHERE lfl.loader_field_id IN (SELECT id FROM loader_fields WHERE field = 'client_side' OR field = 'server_side')
AND lf.field IN ('singleplayer', 'client_and_server', 'client_only', 'server_only')
ON CONFLICT DO NOTHING;

-- Drop the old loader_fields_loaders entries
DELETE FROM loader_fields_loaders WHERE loader_field_id IN (SELECT id FROM loader_fields WHERE field = 'client_side' OR field = 'server_side');

-- Drop client_side and server_side loader fields
DELETE FROM version_fields WHERE field_id IN (SELECT id FROM loader_fields WHERE field = 'client_side' OR field = 'server_side');
DELETE FROM loader_field_enum_values WHERE id IN (SELECT enum_type FROM loader_fields WHERE field = 'client_side' OR field = 'server_side');
DELETE FROM loader_fields WHERE field = 'client_side' OR field = 'server_side';
DELETE FROM loader_field_enums WHERE id IN (SELECT enum_type FROM loader_fields WHERE field = 'side_types');
