-- For every loader that has a loaders_project_types entry that connects it to the project_types 'plugin',
-- remove all non plugin project_types entries for that loader.
-- This is to ensure that the plugin project_types is the only one that is used for the plugin loaders

--plugin
DELETE FROM loaders_project_types
WHERE joining_loader_id IN (
    SELECT DISTINCT l.id
    FROM loaders l
    LEFT JOIN loaders_project_types lpt ON lpt.joining_loader_id = l.id
    LEFT JOIN project_types pt ON pt.id = lpt.joining_project_type_id
    WHERE pt.name = 'plugin'
)
AND joining_project_type_id NOT IN (
    SELECT id FROM project_types
    WHERE name = 'plugin'
);

--datapack
DELETE FROM loaders_project_types
WHERE joining_loader_id IN (
    SELECT DISTINCT l.id
    FROM loaders l
    LEFT JOIN loaders_project_types lpt ON lpt.joining_loader_id = l.id
    LEFT JOIN project_types pt ON pt.id = lpt.joining_project_type_id
    WHERE pt.name = 'datapack'
)
AND joining_project_type_id NOT IN (
    SELECT id FROM project_types
    WHERE name = 'datapack'
);