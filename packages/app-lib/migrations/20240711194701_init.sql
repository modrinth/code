CREATE TABLE settings (
    id INTEGER NOT NULL CHECK (id = 0),

    theme TEXT NOT NULL DEFAULT 'dark',
    default_page TEXT NOT NULL DEFAULT 'home',
    collapsed_navigation INTEGER NOT NULL DEFAULT TRUE,
    discord_rpc INTEGER NOT NULL DEFAULT TRUE,
    hide_on_process_start INTEGER NOT NULL DEFAULT FALSE,
    native_decorations INTEGER NOT NULL DEFAULT FALSE,
    developer_mode INTEGER NOT NULL DEFAULT FALSE,
    telemetry INTEGER NOT NULL DEFAULT TRUE,
    advanced_rendering INTEGER NOT NULL DEFAULT TRUE,
    onboarded INTEGER NOT NULL DEFAULT FALSE,

    mc_memory_max INTEGER NOT NULL DEFAULT 2048,
    mc_force_fullscreen INTEGER NOT NULL DEFAULT FALSE,
    mc_game_resolution_x INTEGER NOT NULL DEFAULT 854,
    mc_game_resolution_y INTEGER NOT NULL DEFAULT 480,
    -- array of strings
    custom_launch_args JSONB NOT NULL DEFAULT jsonb_array(),
    -- array of (string, string)
    custom_env_args JSONB NOT NULL DEFAULT jsonb_array(),

    hook_pre_launch TEXT NULL,
    hook_wrapper TEXT NULL,
    hook_post_exit TEXT NULL,

    PRIMARY KEY (id)
);

INSERT INTO settings (id) VALUES (0);

CREATE TABLE java_versions (
    major_version INTEGER NOT NULL,
    full_version TEXT NOT NULL,
    architecture TEXT NOT NULL,
    path TEXT NOT NULL,

    PRIMARY KEY (major_version)
);

CREATE TABLE minecraft_users (
    uuid BLOB NOT NULL,
    active INTEGER NOT NULL DEFAULT FALSE,
    username TEXT NOT NULL,
    access_token TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    expires INTEGER NOT NULL,

    PRIMARY KEY (uuid)
);
CREATE UNIQUE INDEX minecraft_users_active ON minecraft_users(active);

CREATE TABLE modrinth_users (
    id INTEGER NOT NULL,
    active INTEGER NOT NULL DEFAULT FALSE,
    session_id TEXT NOT NULL,
    expires INTEGER NOT NULL,

    PRIMARY KEY (id)
);
CREATE UNIQUE INDEX minecraft_users_active ON minecraft_users(active);

CREATE TABLE modrinth_cache (
    id INTEGER NOT NULL,
    data_type TEXT NOT NULL,
    alias TEXT NULL,

    data JSONB NOT NULL,
    expires INTEGER NOT NULL,

    UNIQUE (data_type, alias),
    PRIMARY KEY (id, data_type)
);

-- profiles
-- safe processes
-- file name hashes
-- hashes project id version id