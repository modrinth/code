-- Adds loader_fields_loaders entries for all loaders
-- (at this point, they are all Minecraft loaders, and thus have the same fields)
-- These are loaders such as bukkit, minecraft, vanilla, waterfall, velocity... etc
-- This also allows v2 routes (which have things such as client_side to remain to work with these loaders)
INSERT INTO loader_fields_loaders
SELECT l.id, lf.id FROM loaders l CROSS JOIN loader_fields lf
WHERE lf.field=ANY(ARRAY['client_and_server','server_only','client_only','singleplayer'])
AND
l.loader NOT IN ('vanilla', 'minecraft', 'optifine', 'iris', 'canvas', 'bukkit', 'folia', 'paper', 'purpur', 'spigot', 'sponge', 'datapack', 'bungeecord', 'velocity', 'waterfall')
ON CONFLICT DO NOTHING;

INSERT INTO loader_fields_loaders
SELECT l.id, lf.id FROM loaders l CROSS JOIN loader_fields lf
WHERE lf.field=ANY(ARRAY['game_versions'])
ON CONFLICT DO NOTHING;

-- All existing loader_project_types so far should have a games entry as minecraft
INSERT INTO loaders_project_types_games
SELECT lpt.joining_loader_id, lpt.joining_project_type_id, g.id FROM loaders_project_types lpt CROSS JOIN games g
WHERE g.name='minecraft-java'
ON CONFLICT DO NOTHING;
