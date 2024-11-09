CREATE TABLE shared_instances (
    id bigint PRIMARY KEY,
    creator_id bigint REFERENCES users NOT NULL,
    icon_url text NOT NULL,
    name text NOT NULL,
    status text NOT NULL,
    created timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL,
    include_paths text[] NOT NULL,
    exclude_paths text[] NOT NULL,
);

CREATE TABLE shared_instance_invites (
    id bigint PRIMARY KEY,
    creator_id bigint REFERENCES users NOT NULL,
    shared_instance_id bigint REFERENCES shared_instances NOT NULL,

    created timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL,
    expires timestamptz NULL,
    last_used timestamptz NOT NULL,
    max_users bigint NULL,
    uses integer NOT NULL DEFAULT 0,
)

CREATE TABLE shared_instances_users (
    shared_instance_id bigint REFERENCES shared_instances NOT NULL,
    user_id bigint REFERENCES users NOT NULL,

    PRIMARY KEY (shared_instance_id, user_id),
);

CREATE TABLE shared_instances_files (
    id bigint PRIMARY KEY,
    shared_instance_id bigint REFERENCES shared_instances NOT NULL,
    install_path text not null,
    side_type text not null,

    override_id bigint REFERENCES shared_instances_overrides,
    file_id bigint REFERENCES files,
);

CREATE TABLE shared_instances_overrides (
    id bigint PRIMARY KEY,
    size integer NOT NULL,
    -- blake3 hash of file for lookup
    hash bytea NOT NULL,
);