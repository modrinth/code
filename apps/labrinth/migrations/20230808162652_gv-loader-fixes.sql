ALTER TABLE mods ADD COLUMN loaders varchar(255)[] NOT NULL default array[]::varchar[];
ALTER TABLE mods ADD COLUMN game_versions varchar(255)[] NOT NULL default array[]::varchar[];

UPDATE mods
SET loaders = (
    SELECT COALESCE(ARRAY_AGG(DISTINCT l.loader) filter (where l.loader is not null), array[]::varchar[])
    FROM versions v
             INNER JOIN loaders_versions lv ON lv.version_id = v.id
             INNER JOIN loaders l on lv.loader_id = l.id
    WHERE v.mod_id = mods.id
);

UPDATE mods
SET game_versions = (
    SELECT COALESCE(ARRAY_AGG(DISTINCT gv.version) filter (where gv.version is not null), array[]::varchar[])
    FROM versions v
             INNER JOIN game_versions_versions gvv ON v.id = gvv.joining_version_id
             INNER JOIN game_versions gv on gvv.game_version_id = gv.id
    WHERE v.mod_id = mods.id
);
