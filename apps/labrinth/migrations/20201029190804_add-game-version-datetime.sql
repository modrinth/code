
ALTER TABLE game_versions
ADD COLUMN created timestamptz NOT NULL DEFAULT timezone('utc', now());
