ALTER TABLE loaders ADD COLUMN metadata jsonb NOT NULL DEFAULT '{}'::jsonb;

-- Set 'platform' to 'true' for all plugin loaders
-- From knossos v2
    --   pluginLoaders: ['bukkit', 'spigot', 'paper', 'purpur', 'sponge', 'folia'],
    --   pluginPlatformLoaders: ['bungeecord', 'waterfall', 'velocity'],
    --   allPluginLoaders: [
    --     'bukkit',
    --     'spigot',
    --     'paper',
    --     'purpur',
    --     'sponge',
    --     'bungeecord',
    --     'waterfall',
    --     'velocity',
    --     'folia',
    --   ],
    --   dataPackLoaders: ['datapack'],
    --   modLoaders: ['forge', 'fabric', 'quilt', 'liteloader', 'modloader', 'rift', 'neoforge'],
UPDATE loaders SET metadata = jsonb_set(metadata, '{platform}', 'false'::jsonb) WHERE loader in ('bukkit', 'spigot', 'paper', 'purpur', 'sponge', 'folia');
UPDATE loaders SET metadata = jsonb_set(metadata, '{platform}', 'true'::jsonb) WHERE loader in ('bungeecord', 'waterfall', 'velocity');

INSERT INTO project_types (name) VALUES ('plugin');
INSERT INTO project_types (name) VALUES ('datapack');

INSERT INTO loaders_project_types (joining_loader_id, joining_project_type_id) 
SELECT l.id, pt.id
FROM loaders l
CROSS JOIN project_types pt
WHERE l.loader in ('datapack')
AND pt.name = 'datapack';

INSERT INTO loaders_project_types (joining_loader_id, joining_project_type_id)
SELECT l.id, pt.id
FROM loaders l
CROSS JOIN project_types pt
WHERE l.loader in ('bukkit', 'spigot', 'paper', 'purpur', 'sponge', 'bungeecord', 'waterfall', 'velocity', 'folia')
AND pt.name = 'plugin';

INSERT INTO loaders_project_types_games (loader_id, project_type_id, game_id)
SELECT joining_loader_id, joining_project_type_id, g.id
FROM loaders_project_types lpt
INNER JOIN project_types pt ON pt.id = lpt.joining_project_type_id
CROSS JOIN games g
WHERE g.name = 'minecraft'
AND pt.name in ('plugin', 'datapack');
