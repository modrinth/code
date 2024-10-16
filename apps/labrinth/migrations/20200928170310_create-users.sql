ALTER TABLE users
ADD COLUMN github_id bigint NOT NULL default 0,
ADD COLUMN username varchar(255) NOT NULL default 'username',
ADD COLUMN name varchar(255) NOT NULL default 'John Doe',
ADD COLUMN email varchar(255) NULL default 'johndoe@modrinth.com',
ADD COLUMN avatar_url varchar(500) NOT NULL default '...',
ADD COLUMN bio varchar(160) NOT NULL default 'I make mods!',
ADD COLUMN created timestamptz default CURRENT_TIMESTAMP NOT NULL