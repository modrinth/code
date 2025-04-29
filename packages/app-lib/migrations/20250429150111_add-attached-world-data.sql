CREATE TABLE attached_world_data (
    profile_path TEXT NOT NULL,
    world_type TEXT CHECK ( world_type in ('singleplayer', 'server') ) NOT NULL,
    world_id TEXT NOT NULL,
    display_status TEXT NOT NULL DEFAULT 'normal',

    PRIMARY KEY (profile_path, world_type, world_id),
    FOREIGN KEY (profile_path) REFERENCES profiles(path) ON DELETE CASCADE
);
CREATE INDEX attached_world_data_profile_path ON attached_world_data(profile_path);
