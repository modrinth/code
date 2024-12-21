ALTER TABLE users DROP COLUMN kratos_id;

ALTER TABLE states ADD COLUMN provider varchar(64) NOT NULL default 'github';

ALTER TABLE users ADD COLUMN discord_id bigint;
ALTER TABLE users ADD COLUMN gitlab_id bigint;
ALTER TABLE users ADD COLUMN google_id varchar(256);
ALTER TABLE users ADD COLUMN steam_id bigint;
ALTER TABLE users ADD COLUMN microsoft_id varchar(256);

CREATE INDEX users_discord_id
    ON users (discord_id);
CREATE INDEX users_gitlab_id
    ON users (gitlab_id);
CREATE INDEX users_google_id
    ON users (google_id);
CREATE INDEX users_steam_id
    ON users (steam_id);
CREATE INDEX users_microsoft_id
    ON users (microsoft_id);

ALTER TABLE users ALTER COLUMN avatar_url TYPE varchar(1024);
ALTER TABLE users ADD COLUMN password TEXT NULL;
ALTER TABLE users ADD COLUMN email_verified BOOLEAN NOT NULL DEFAULT FALSE;

CREATE TABLE sessions (
    id bigint NOT NULL PRIMARY KEY,
    session varchar(64) NOT NULL UNIQUE,
    user_id BIGINT NOT NULL REFERENCES users(id),
    created timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL,
    last_login timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL,
    expires timestamptz DEFAULT CURRENT_TIMESTAMP + interval '14 days' NOT NULL,
    refresh_expires timestamptz DEFAULT CURRENT_TIMESTAMP + interval '60 days' NOT NULL,

    city varchar(256) NULL,
    country varchar(256) NULL,
    ip varchar(512) NOT NULL,

    os varchar(256) NULL,
    platform varchar(256) NULL,
    user_agent varchar(1024) NOT NULL
);

CREATE INDEX sessions_user_id
    ON sessions (user_id);

ALTER TABLE mods DROP COLUMN game_versions;
ALTER TABLE mods DROP COLUMN loaders;
