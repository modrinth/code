INSERT INTO project_types (id, name)
VALUES (8, 'map');

SELECT setval(
	pg_get_serial_sequence('project_types', 'id'),
	(SELECT MAX(id) FROM project_types)
);

INSERT INTO categories (header, category, project_type)
VALUES
	('genre', 'adventure', 8),
	('genre', 'survival', 8),
	('genre', 'puzzle', 8),
	('genre', 'parkour', 8),
	('genre', 'horror', 8),
	('genre', 'minigame', 8),
	('genre', 'pvp', 8),
	('genre', 'escape', 8),
	('genre', 'story', 8),
	('genre', 'building', 8),
	('genre', 'creation', 8),
	('play-style', 'singleplayer', 8),
	('play-style', 'multiplayer', 8),
	('play-style', 'co-op', 8),
	('play-style', 'competitive', 8),
	('audience', 'short', 8),
	('audience', 'medium', 8),
	('audience', 'long', 8),
	('audience', 'casual', 8),
	('audience', 'hardcore', 8),
	('technical', 'vanilla', 8),
	('technical', 'command-blocks', 8),
	('technical', 'resource-pack-supported', 8),
	('format', 'world-template', 8),
	('format', 'playable-map', 8);
