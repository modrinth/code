INSERT OR IGNORE INTO instance_groups (id, name)
VALUES ('group:favorites', 'Favorites');

UPDATE instance_groups
SET name = 'Favorites'
WHERE id = 'group:favorites';
