CREATE TABLE settings (
    id INTEGER NOT NULL CHECK (id = 0),

    max_concurrent_downloads INTEGER NOT NULL DEFAULT 10,
    max_concurrent_writes INTEGER NOT NULL DEFAULT 10,

    theme TEXT NOT NULL DEFAULT 'dark',
    default_page TEXT NOT NULL DEFAULT 'home',
    collapsed_navigation INTEGER NOT NULL DEFAULT TRUE,
    advanced_rendering INTEGER NOT NULL DEFAULT TRUE,
    native_decorations INTEGER NOT NULL DEFAULT FALSE,

    telemetry INTEGER NOT NULL DEFAULT TRUE,
    discord_rpc INTEGER NOT NULL DEFAULT TRUE,
    developer_mode INTEGER NOT NULL DEFAULT FALSE,

    onboarded INTEGER NOT NULL DEFAULT FALSE,

    -- array of strings
    extra_launch_args JSONB NOT NULL,
    -- array of (string, string)
    custom_env_vars JSONB NOT NULL,
    mc_memory_max INTEGER NOT NULL DEFAULT 2048,
    mc_force_fullscreen INTEGER NOT NULL DEFAULT FALSE,
    mc_game_resolution_x INTEGER NOT NULL DEFAULT 854,
    mc_game_resolution_y INTEGER NOT NULL DEFAULT 480,

    hide_on_process_start INTEGER NOT NULL DEFAULT FALSE,

    hook_pre_launch TEXT NULL,
    hook_wrapper TEXT NULL,
    hook_post_exit TEXT NULL,

    custom_dir TEXT NULL,
    prev_custom_dir TEXT NULL,
    migrated INTEGER NOT NULL DEFAULT FALSE,

    PRIMARY KEY (id)
);

INSERT INTO settings (id, extra_launch_args, custom_env_vars) VALUES (0, jsonb_array(), jsonb_array());

CREATE TABLE java_versions (
    major_version INTEGER NOT NULL,
    full_version TEXT NOT NULL,
    architecture TEXT NOT NULL,
    path TEXT NOT NULL,

    PRIMARY KEY (major_version)
);

CREATE TABLE minecraft_users (
    uuid TEXT NOT NULL,
    active INTEGER NOT NULL DEFAULT FALSE,
    username TEXT NOT NULL,
    access_token TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    expires INTEGER NOT NULL,

    PRIMARY KEY (uuid)
);
CREATE UNIQUE INDEX minecraft_users_active ON minecraft_users(active);

CREATE TABLE minecraft_device_tokens (
    id INTEGER NOT NULL CHECK (id = 0),

    uuid TEXT NOT NULL,
    private_key TEXT NOT NULL,
    x TEXT NOT NULL,
    y TEXT NOT NULL,
    issue_instant INTEGER NOT NULL,
    not_after INTEGER NOT NULL,
    token TEXT NOT NULL,
    display_claims JSONB NOT NULL,

    PRIMARY KEY (id)
);

CREATE TABLE modrinth_users (
    id TEXT NOT NULL,
    active INTEGER NOT NULL DEFAULT FALSE,
    session_id TEXT NOT NULL,
    expires INTEGER NOT NULL,

    PRIMARY KEY (id)
);
CREATE UNIQUE INDEX modrinth_users_active ON modrinth_users(active);

CREATE TABLE cache (
    id TEXT NOT NULL,
    data_type TEXT NOT NULL,
    alias TEXT NULL,

    data JSONB NULL,
    expires INTEGER NOT NULL,

    UNIQUE (data_type, alias),
    PRIMARY KEY (id, data_type)
);

CREATE TABLE profiles (
    path TEXT NOT NULL,
    install_stage TEXT NOT NULL,

    name TEXT NOT NULL,
    icon_path TEXT NULL,

    game_version TEXT NOT NULL,
    mod_loader TEXT NOT NULL,
    mod_loader_version TEXT NULL,

    -- array of strings
    groups JSONB NOT NULL,

    linked_project_id TEXT NULL,
    linked_version_id TEXT NULL,
    locked INTEGER NULL,

    created INTEGER NOT NULL,
    modified INTEGER NOT NULL,
    last_played INTEGER NULL,

    submitted_time_played INTEGER NOT NULL DEFAULT 0,
    recent_time_played INTEGER NOT NULL DEFAULT 0,

    override_java_path TEXT NULL,

    -- array of strings
    override_extra_launch_args JSONB NOT NULL,
    -- array of (string, string)
    override_custom_env_vars JSONB NOT NULL,

    override_mc_memory_max INTEGER NULL,
    override_mc_force_fullscreen INTEGER NULL,
    override_mc_game_resolution_x INTEGER NULL,
    override_mc_game_resolution_y INTEGER NULL,

    override_hook_pre_launch TEXT NULL,
    override_hook_wrapper TEXT NULL,
    override_hook_post_exit TEXT NULL,


    PRIMARY KEY (path)
);

CREATE TABLE processes (
    pid INTEGER NOT NULL,
    start_time INTEGER NOT NULL,
    name TEXT NOT NULL,
    executable TEXT NOT NULL,
    profile_path TEXT NOT NULL,
    post_exit_command TEXT NULL,

    UNIQUE (pid),
    PRIMARY KEY (pid),
    FOREIGN KEY (profile_path) REFERENCES profiles(path)
);
CREATE INDEX processes_profile_path ON processes(profile_path);
