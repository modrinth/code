CREATE TABLE join_log (
    profile_path TEXT NOT NULL,
    host TEXT NOT NULL,
    port INTEGER NOT NULL,
    join_time INTEGER NOT NULL,

    PRIMARY KEY (profile_path, host, port),
    FOREIGN KEY (profile_path) REFERENCES profiles(path) ON DELETE CASCADE
);
CREATE INDEX join_log_profile_path ON join_log(profile_path);
