CREATE TABLE games (
	id int PRIMARY KEY, -- Only used in db
	name varchar(64),
  CONSTRAINT unique_game_name UNIQUE (name)
);
INSERT INTO games(id, name) VALUES (1, 'minecraft-java');
INSERT INTO games(id, name) VALUES (2, 'minecraft-bedrock');

ALTER TABLE loaders ADD CONSTRAINT unique_loader_name UNIQUE (loader);

CREATE TABLE loader_field_enums (
  id serial PRIMARY KEY,
  enum_name varchar(64) NOT NULL,
  ordering int NULL,
  hidable BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE loader_field_enum_values (
  id serial PRIMARY KEY,
  enum_id integer REFERENCES loader_field_enums NOT NULL,
  value varchar(64) NOT NULL,
  ordering int NULL,
  created timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
  -- metadata is json of all the extra data for this enum value
  metadata jsonb NULL,
	
  original_id integer, -- This is for mapping only- it is dropped before the end of the migration

  CONSTRAINT unique_variant_per_enum UNIQUE (enum_id, value)

);

CREATE TABLE loader_fields (
  id serial PRIMARY KEY,
  field varchar(64) UNIQUE NOT NULL,
  -- "integer", "text", "enum", "bool", 
  -- "array_integer", "array_text", "array_enum", "array_bool"
  field_type varchar(64) NOT NULL,
  -- only for enum
  enum_type integer REFERENCES loader_field_enums NULL,
  optional BOOLEAN NOT NULL DEFAULT true,
  -- for int- min/max val, for text- min len, for enum- min items, for bool- nothing
  min_val integer NULL,
  max_val integer NULL
);

CREATE TABLE loader_fields_loaders (
  loader_id integer REFERENCES loaders NOT NULL,
  loader_field_id integer REFERENCES loader_fields NOT NULL,
  CONSTRAINT unique_loader_field UNIQUE (loader_id, loader_field_id)
);

ALTER TABLE loaders ADD COLUMN hidable boolean NOT NULL default false;

CREATE TABLE version_fields (
  version_id bigint REFERENCES versions  NOT NULL,
  field_id integer REFERENCES loader_fields  NOT NULL,
  -- for int/bool values
  int_value integer NULL,
  enum_value integer REFERENCES loader_field_enum_values  NULL,
  string_value text NULL
);

-- Convert side_types
INSERT INTO loader_field_enums (id, enum_name, hidable) VALUES (1, 'side_types', true);
INSERT INTO loader_field_enum_values (original_id, enum_id, value) SELECT id, 1, name FROM side_types st;

INSERT INTO loader_fields (field, field_type, enum_type, optional, min_val, max_val) SELECT 'client_side', 'enum', 1, false, 1, 1;
INSERT INTO loader_fields ( field, field_type, enum_type, optional, min_val, max_val) SELECT 'server_side', 'enum', 1, false, 1, 1;

INSERT INTO loader_fields_loaders (loader_id, loader_field_id) SELECT l.id, lf.id FROM loaders l CROSS JOIN loader_fields lf  WHERE lf.field = 'client_side' AND l.loader = ANY( ARRAY['forge', 'fabric', 'quilt', 'modloader','rift','liteloader', 'neoforge']);
INSERT INTO loader_fields_loaders (loader_id, loader_field_id) SELECT l.id, lf.id FROM loaders l CROSS JOIN loader_fields lf  WHERE lf.field = 'server_side' AND l.loader = ANY( ARRAY['forge', 'fabric', 'quilt', 'modloader','rift','liteloader', 'neoforge']);

INSERT INTO version_fields (version_id, field_id, enum_value) 
SELECT v.id, lf.id, lfev.id  -- Note: bug fix/edited 2023-11-27
FROM versions v
INNER JOIN mods m ON v.mod_id = m.id
INNER JOIN loader_field_enum_values lfev ON m.client_side = lfev.original_id
CROSS JOIN loader_fields lf
WHERE client_side IS NOT NULL AND lfev.enum_id = 1 AND lf.field = 'client_side' AND NOT (ARRAY['vanilla', 'minecraft', 'optifine', 'iris', 'canvas', 'bukkit', 'folia', 'paper', 'purpur', 'spigot', 'sponge', 'datapack', 'bungeecord', 'velocity', 'waterfall'] @> m.loaders::text[]);;

INSERT INTO version_fields (version_id, field_id, enum_value) 
SELECT v.id, lf.id, lfev.id   -- Note: bug fix/edited 2023-11-27
FROM versions v
INNER JOIN mods m ON v.mod_id = m.id
INNER JOIN loader_field_enum_values lfev ON m.server_side = lfev.original_id
CROSS JOIN loader_fields lf
WHERE server_side IS NOT NULL AND lfev.enum_id = 1 AND lf.field = 'server_side' AND NOT (ARRAY['vanilla', 'minecraft', 'optifine', 'iris', 'canvas', 'bukkit', 'folia', 'paper', 'purpur', 'spigot', 'sponge', 'datapack', 'bungeecord', 'velocity', 'waterfall'] @> m.loaders::text[]);

ALTER TABLE mods DROP COLUMN client_side;
ALTER TABLE mods DROP COLUMN server_side;
DROP TABLE side_types;

-- Convert game_versions
INSERT INTO loader_field_enums (id, enum_name, hidable) VALUES (2, 'game_versions', true);
INSERT INTO loader_field_enum_values (original_id, enum_id, value, created, metadata)
SELECT id, 2, version, created, json_build_object('type', type, 'major', major) FROM game_versions;

INSERT INTO loader_fields (field, field_type, enum_type, optional, min_val) VALUES('game_versions', 'array_enum', 2, false, 0);
INSERT INTO loader_fields_loaders (loader_id, loader_field_id) SELECT l.id, lf.id FROM loaders l CROSS JOIN loader_fields lf  WHERE lf.field = 'game_versions' AND l.loader = ANY( ARRAY['forge', 'fabric', 'quilt', 'modloader','rift','liteloader', 'neoforge']);

-- remove dangling game versions
DELETE FROM game_versions_versions
WHERE joining_version_id NOT IN (SELECT id FROM versions);

INSERT INTO version_fields(version_id, field_id, enum_value)
SELECT gvv.joining_version_id, lf.id, lfev.id
FROM game_versions_versions gvv INNER JOIN loader_field_enum_values lfev ON gvv.game_version_id = lfev.original_id
CROSS JOIN loader_fields lf
WHERE lf.field = 'game_versions' AND lfev.enum_id = 2;

ALTER TABLE mods DROP COLUMN loaders;
ALTER TABLE mods DROP COLUMN game_versions;
DROP TABLE game_versions_versions;
DROP TABLE game_versions;

-- Convert project types
-- we are creating a new loader type- 'mrpack'- for minecraft modpacks
SELECT setval('loaders_id_seq', (SELECT MAX(id) FROM loaders) + 1, false);
INSERT INTO loaders (loader) VALUES ('mrpack');

-- For the loader 'mrpack', we create loader fields for every loader
-- That way we keep information like "this modpack is a fabric modpack"
INSERT INTO loader_field_enums (id, enum_name, hidable) VALUES (3, 'mrpack_loaders', true);
INSERT INTO loader_field_enum_values (original_id, enum_id, value) SELECT id, 3, loader FROM loaders WHERE loader != 'mrpack';
INSERT INTO loader_fields (field, field_type, enum_type, optional, min_val) VALUES('mrpack_loaders', 'array_enum', 3, false, 0);
INSERT INTO loader_fields_loaders (loader_id, loader_field_id) 
SELECT l.id, lf.id FROM loaders l CROSS JOIN loader_fields lf  WHERE lf.field = 'mrpack_loaders' AND l.loader = 'mrpack';

INSERT INTO version_fields(version_id, field_id, enum_value)
SELECT v.id, lf.id, lfev.id
FROM versions v
INNER JOIN mods m ON v.mod_id = m.id
INNER JOIN loaders_versions lv ON v.id = lv.version_id
INNER JOIN loaders l ON lv.loader_id = l.id
CROSS JOIN loader_fields lf
LEFT JOIN loader_field_enum_values lfev ON lf.enum_type = lfev.enum_id AND lfev.original_id = l.id
WHERE m.project_type = (SELECT id FROM project_types WHERE name = 'modpack') AND lf.field = 'mrpack_loaders';

INSERT INTO loaders_project_types (joining_loader_id, joining_project_type_id) SELECT DISTINCT l.id, pt.id FROM loaders l CROSS JOIN project_types pt WHERE pt.name = 'modpack' AND l.loader = 'mrpack';

-- Set those versions to mrpack as their version
INSERT INTO loaders_versions (version_id, loader_id)
SELECT DISTINCT vf.version_id, l.id
FROM version_fields vf
LEFT JOIN loader_fields lf ON lf.id = vf.field_id
CROSS JOIN loaders l
WHERE lf.field = 'mrpack_loaders'
AND l.loader = 'mrpack'
ON CONFLICT DO NOTHING;

--  Delete the old versions that had mrpack added to them
DELETE FROM loaders_versions lv
WHERE lv.loader_id != (SELECT id FROM loaders WHERE loader = 'mrpack')
AND lv.version_id IN (
    SELECT version_id
    FROM loaders_versions
    WHERE loader_id = (SELECT id FROM loaders WHERE loader = 'mrpack')
);


--- Non-mrpack loaders no longer support modpacks
DELETE FROM loaders_project_types WHERE joining_loader_id != (SELECT id FROM loaders WHERE loader = 'mrpack') AND joining_project_type_id = (SELECT id FROM project_types WHERE name = 'modpack');

CREATE TABLE loaders_project_types_games (
  loader_id integer REFERENCES loaders NOT NULL,
  project_type_id integer REFERENCES project_types NOT NULL,
  game_id integer REFERENCES games NOT NULL,
  PRIMARY KEY (loader_id, project_type_id, game_id)
);

-- all past loader_project_types are minecraft-java as the only game before this migration is minecraft-java
INSERT INTO loaders_project_types_games (loader_id, project_type_id, game_id) SELECT joining_loader_id, joining_project_type_id, 1 FROM loaders_project_types;

-- Now that loaders are inferred, we can drop the project_type column from mods
ALTER TABLE mods DROP COLUMN project_type;


-- Drop original_id columns
ALTER TABLE loader_field_enum_values DROP COLUMN original_id;