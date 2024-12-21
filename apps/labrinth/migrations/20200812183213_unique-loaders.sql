ALTER TABLE game_versions
ADD UNIQUE(version);

ALTER TABLE loaders
ADD UNIQUE(loader);
