-- Core's schema. Written with IF NOT EXISTS so a database created by the older, longer
-- migration history adopts it and keeps its servers (see `infrastructure::db::migrate`).

CREATE TABLE IF NOT EXISTS core_identity (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    core_id TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL
);

-- Temporary until pairing exists: the first account to connect owns this Core.
CREATE TABLE IF NOT EXISTS core_owner (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    user_id TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS java_installations (
    version INTEGER PRIMARY KEY,
    path TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS server_installations (
    id TEXT PRIMARY KEY,
    game_version TEXT NOT NULL,
    loader TEXT NOT NULL,
    loader_version TEXT,
    status TEXT NOT NULL DEFAULT 'installing',
    error TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS instances (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    game_version TEXT,
    loader TEXT,
    port INTEGER,
    memory_min INTEGER,
    memory_max INTEGER,
    status TEXT DEFAULT 'offline',
    data_dir TEXT,
    loader_version TEXT,
    java_version INTEGER,
    updated_at TEXT NOT NULL DEFAULT '1970-01-01T00:00:00+00:00',
    install_status TEXT NOT NULL DEFAULT 'ready',
    jvm_args TEXT,
    server_args TEXT,
    total_uptime_seconds INTEGER NOT NULL DEFAULT 0,
    installation_id TEXT,
    path TEXT
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_instances_path ON instances(path);

CREATE TABLE IF NOT EXISTS mods (
    id TEXT PRIMARY KEY,
    instance_id TEXT NOT NULL REFERENCES instances(id) ON DELETE CASCADE,
    filename TEXT NOT NULL,
    display_name TEXT,
    modrinth_project_id TEXT,
    modrinth_version_id TEXT,
    version_number TEXT,
    client_side TEXT,
    server_side TEXT,
    sha512 TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    installed_at TEXT NOT NULL,
    UNIQUE(instance_id, filename)
);

CREATE TABLE IF NOT EXISTS backups (
    id TEXT PRIMARY KEY,
    instance_id TEXT NOT NULL REFERENCES instances(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    size_bytes INTEGER NOT NULL DEFAULT 0,
    locked INTEGER NOT NULL DEFAULT 0,
    trigger TEXT NOT NULL DEFAULT 'manual',
    created_at TEXT NOT NULL,
    hot INTEGER NOT NULL DEFAULT 0,
    consistency TEXT NOT NULL DEFAULT 'offline'
);

CREATE TABLE IF NOT EXISTS server_sources (
    instance_id TEXT PRIMARY KEY REFERENCES instances(id) ON DELETE CASCADE,
    -- ServerSource JSON.
    source_json TEXT NOT NULL,
    desired_version TEXT NOT NULL,
    installed_version TEXT,
    -- Version whose files are downloaded and verified in Core's staging directory.
    staged_version TEXT,
    state TEXT NOT NULL,
    error TEXT,
    -- Stop a running server, apply, and start it again.
    restart_requested INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS server_source_files (
    instance_id TEXT NOT NULL REFERENCES server_sources(instance_id) ON DELETE CASCADE,
    -- Path relative to the instance data dir, e.g. `mods/a.jar` or `world/datapacks/b.zip`.
    path TEXT NOT NULL,
    kind TEXT NOT NULL,
    sha1 TEXT NOT NULL,
    client_only INTEGER NOT NULL,
    project_id TEXT,
    version_id TEXT,
    PRIMARY KEY (instance_id, path)
);

-- Tables of the removed second API, left behind in databases from the older history.
DROP TABLE IF EXISTS sync_events;
DROP TABLE IF EXISTS sync_snapshots;
DROP TABLE IF EXISTS sync_profiles;
DROP TABLE IF EXISTS scheduled_tasks;
DROP TABLE IF EXISTS backup_schedules;
DROP TABLE IF EXISTS instance_members;
DROP TABLE IF EXISTS activity_log;
DROP TABLE IF EXISTS core_invitations;
DROP TABLE IF EXISTS core_role_settings;
DROP TABLE IF EXISTS core_roles;
DROP TABLE IF EXISTS core_group_bans;
DROP TABLE IF EXISTS core_members;
DROP TABLE IF EXISTS core_metadata;
DROP TABLE IF EXISTS core_config;
DROP TABLE IF EXISTS events;
DROP TABLE IF EXISTS paseto_key;
DROP TABLE IF EXISTS users;
