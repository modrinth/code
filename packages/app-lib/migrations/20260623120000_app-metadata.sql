CREATE TABLE app_metadata (
	key TEXT NOT NULL,
	value TEXT NOT NULL,
	updated_at INTEGER NOT NULL DEFAULT (unixepoch()),

	PRIMARY KEY (key)
);
