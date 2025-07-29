DO LANGUAGE plpgsql $$
DECLARE
    VAR_env_field_id INT;
    VAR_env_field_enum_id INT := 4; -- Known available ID for a new enum type
BEGIN

-- Define a new loader field for environment
INSERT INTO loader_field_enums (id, enum_name, ordering, hidable)
    VALUES (VAR_env_field_enum_id, 'environment', NULL, TRUE);

INSERT INTO loader_field_enum_values (enum_id, value, ordering, created, metadata)
    VALUES
        -- Must be installed on both client and (integrated) server
        (VAR_env_field_enum_id, 'client_and_server', NULL, NOW(), NULL),
        -- Must be installed only on the client
        (VAR_env_field_enum_id, 'client_only', NULL, NOW(), NULL),
        -- Must be installed on the client, may be installed on a (integrated) server. To be displayed as a
        -- client mod
        (VAR_env_field_enum_id, 'client_only_server_optional', NULL, NOW(), NULL),
        -- Must be installed only on the integrated singleplayer server. To be displayed as a server mod for
        -- singleplayer exclusively
        (VAR_env_field_enum_id, 'singleplayer_only', NULL, NOW(), NULL),
        -- Must be installed only on a (integrated) server
        (VAR_env_field_enum_id, 'server_only', NULL, NOW(), NULL),
        -- Must be installed on the server, may be installed on the client. To be displayed as a
        -- singleplayer-compatible server mod
        (VAR_env_field_enum_id, 'server_only_client_optional', NULL, NOW(), NULL),
        -- Must be installed only on a dedicated multiplayer server (not the integrated singleplayer server).
        -- To be displayed as an server mod for multiplayer exclusively
        (VAR_env_field_enum_id, 'dedicated_server_only', NULL, NOW(), NULL),
        -- Can be installed on both client and server, with no strong preference for either. To be displayed
        -- as both a client and server mod
        (VAR_env_field_enum_id, 'client_or_server', NULL, NOW(), NULL),
        -- Can be installed on both client and server, with a preference for being installed on both. To be
        -- displayed as a client and server mod
        (VAR_env_field_enum_id, 'client_or_server_prefers_both', NULL, NOW(), NULL),
        (VAR_env_field_enum_id, 'unknown', NULL, NOW(), NULL);

INSERT INTO loader_fields (field, field_type, enum_type, optional)
    VALUES ('environment', 'enum', VAR_env_field_enum_id, FALSE)
    RETURNING id INTO VAR_env_field_id;

-- Update version_fields to have the new environment field, initializing it from the
-- values of the previous fields
INSERT INTO version_fields (version_id, field_id, enum_value)
    SELECT vf.version_id, VAR_env_field_id, (
        SELECT id
        FROM loader_field_enum_values
        WHERE enum_id = VAR_env_field_enum_id
        AND value = (
            CASE jsonb_object_agg(lf.field, vf.int_value)
                WHEN '{ "server_only": 0, "singleplayer": 0, "client_and_server": 0, "client_only": 1 }'::jsonb THEN 'client_only'
                WHEN '{ "server_only": 0, "singleplayer": 0, "client_and_server": 1, "client_only": 0 }'::jsonb THEN 'client_and_server'
                WHEN '{ "server_only": 0, "singleplayer": 0, "client_and_server": 1, "client_only": 1 }'::jsonb THEN 'client_only_server_optional'
                WHEN '{ "server_only": 0, "singleplayer": 1, "client_and_server": 0, "client_only": 0 }'::jsonb THEN 'singleplayer_only'
                WHEN '{ "server_only": 0, "singleplayer": 1, "client_and_server": 0, "client_only": 1 }'::jsonb THEN 'client_only'
                WHEN '{ "server_only": 0, "singleplayer": 1, "client_and_server": 1, "client_only": 0 }'::jsonb THEN 'client_and_server'
                WHEN '{ "server_only": 0, "singleplayer": 1, "client_and_server": 1, "client_only": 1 }'::jsonb THEN 'client_only_server_optional'
                WHEN '{ "server_only": 1, "singleplayer": 0, "client_and_server": 0, "client_only": 0 }'::jsonb THEN 'server_only'
                WHEN '{ "server_only": 1, "singleplayer": 0, "client_and_server": 0, "client_only": 1 }'::jsonb THEN 'client_or_server'
                WHEN '{ "server_only": 1, "singleplayer": 0, "client_and_server": 1, "client_only": 0 }'::jsonb THEN 'server_only_client_optional'
                WHEN '{ "server_only": 1, "singleplayer": 0, "client_and_server": 1, "client_only": 1 }'::jsonb THEN 'client_or_server_prefers_both'
                WHEN '{ "server_only": 1, "singleplayer": 1, "client_and_server": 0, "client_only": 0 }'::jsonb THEN 'server_only'
                WHEN '{ "server_only": 1, "singleplayer": 1, "client_and_server": 0, "client_only": 1 }'::jsonb THEN 'client_or_server'
                WHEN '{ "server_only": 1, "singleplayer": 1, "client_and_server": 1, "client_only": 0 }'::jsonb THEN 'server_only_client_optional'
                WHEN '{ "server_only": 1, "singleplayer": 1, "client_and_server": 1, "client_only": 1 }'::jsonb THEN 'client_or_server_prefers_both'
                ELSE 'unknown'
            END
        )
    )
    FROM version_fields vf
    JOIN loader_fields lf ON vf.field_id = lf.id
    WHERE lf.field IN ('server_only', 'singleplayer', 'client_and_server', 'client_only')
    GROUP BY vf.version_id
    HAVING COUNT(DISTINCT lf.field) = 4;

-- Clean up old fields from the project versions
DELETE FROM version_fields
    WHERE field_id IN (
        SELECT id
        FROM loader_fields
        WHERE field IN ('server_only', 'singleplayer', 'client_and_server', 'client_only')
    );

-- Switch loader fields definitions on the available loaders to use the new environment field
ALTER TABLE loader_fields_loaders DROP CONSTRAINT unique_loader_field;
ALTER TABLE loader_fields_loaders DROP CONSTRAINT loader_fields_loaders_pkey;
ALTER TABLE loader_fields_loaders REPLICA IDENTITY FULL; -- Required due to temporary PK removal for replica sync in production

UPDATE loader_fields_loaders
    SET loader_field_id = VAR_env_field_id
    WHERE loader_field_id IN (
        SELECT id
        FROM loader_fields
        WHERE field IN ('server_only', 'singleplayer', 'client_and_server', 'client_only')
    );

-- Remove duplicate (loader_id, loader_field_id) pairs that may have been created due to several
-- old fields being converted to a single new field
DELETE FROM loader_fields_loaders
    WHERE ctid NOT IN (
        SELECT MIN(ctid)
        FROM loader_fields_loaders
        GROUP BY loader_id, loader_field_id
    );

-- Having both a PK and UNIQUE constraint for the same columns is redundant, so only restore the PK
ALTER TABLE loader_fields_loaders ADD PRIMARY KEY (loader_id, loader_field_id);
ALTER TABLE loader_fields_loaders REPLICA IDENTITY DEFAULT;

-- Finally, remove the old loader fields
DELETE FROM loader_fields
    WHERE field IN ('server_only', 'singleplayer', 'client_and_server', 'client_only');

-- Add a field to the projects table to track whether the new environment field value has been
-- reviewed to be appropriate after automated migration
ALTER TABLE mods
    ADD COLUMN side_types_migration_review_status VARCHAR(64) NOT NULL DEFAULT 'reviewed'
    CHECK (side_types_migration_review_status IN ('reviewed', 'pending'));

UPDATE mods SET side_types_migration_review_status = 'pending';

END;
$$
