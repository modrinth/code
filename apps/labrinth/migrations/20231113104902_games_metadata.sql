ALTER TABLE games ADD COLUMN slug varchar(64);
ALTER TABLE games ADD COLUMN icon_url varchar(2048) NULL;
ALTER TABLE games ADD COLUMN banner_url varchar(2048) NULL;

-- 'minecraft-java' and 'minecraft-bedrock' are the only games- both slug and names (names are for translations)
UPDATE games SET slug = name;
ALTER TABLE games ALTER COLUMN slug SET NOT NULL;
ALTER TABLE games ALTER COLUMN name SET NOT NULL;
ALTER TABLE games ADD CONSTRAINT unique_game_slug UNIQUE (slug);