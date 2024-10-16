
-- Originally:
-- ALTER TABLE users
-- ADD COLUMN github_id bigint NOT NULL default 0,
-- ADD COLUMN username varchar(255) NOT NULL default 'username',
-- ADD COLUMN name varchar(255) NOT NULL default 'John Doe',
-- ADD COLUMN email varchar(255) NULL default 'johndoe@modrinth.com',
-- ADD COLUMN avatar_url varchar(500) NOT NULL default '...',
-- ADD COLUMN bio varchar(160) NOT NULL default 'I make mods!',
-- ADD COLUMN created timestamptz default CURRENT_TIMESTAMP NOT NULL

-- We don't want garbage data when users are created incorrectly;
-- we just want it to fail.

ALTER TABLE users
ALTER COLUMN github_id DROP NOT NULL;
ALTER TABLE users
ALTER COLUMN github_id DROP DEFAULT;

ALTER TABLE users
ALTER COLUMN avatar_url DROP NOT NULL;
ALTER TABLE users
ALTER COLUMN avatar_url DROP DEFAULT;

ALTER TABLE users
ALTER COLUMN username DROP DEFAULT;
ALTER TABLE users
ALTER COLUMN name DROP DEFAULT;
ALTER TABLE users
ALTER COLUMN email DROP DEFAULT;

ALTER TABLE users
ALTER COLUMN bio DROP DEFAULT;
ALTER TABLE users
ALTER COLUMN bio DROP NOT NULL;
