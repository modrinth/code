CREATE TABLE app_analytics (
	id INTEGER NOT NULL CHECK (id = 0),
	installation_id TEXT NOT NULL,
	PRIMARY KEY (id)
);
