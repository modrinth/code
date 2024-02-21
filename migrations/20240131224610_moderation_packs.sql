CREATE TABLE moderation_external_licenses (
    id bigint PRIMARY KEY,
    title text not null,
    status text not null,
    link text null,
    exceptions text null,
    proof text null,
    flame_project_id integer null
);

CREATE TABLE moderation_external_files (
    sha1 bytea PRIMARY KEY,
    external_license_id bigint references moderation_external_licenses not null
);

ALTER TABLE files ADD COLUMN metadata jsonb NULL;

INSERT INTO users (id, username, name, email, avatar_url, bio, role, badges, balance)
VALUES (0, 'AutoMod', 'AutoMod', 'support@modrinth.com', 'https://cdn.modrinth.com/user/2REoufqX/6aabaf2d1fca2935662eca4ce451cd9775054c22.png', 'An automated account performing moderation utilities for Modrinth.', 'moderator', 0, 0)