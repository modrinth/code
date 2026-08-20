CREATE TABLE instance_icon_configs (
	instance_id TEXT NOT NULL,
	background TEXT NOT NULL,
	symbol TEXT NOT NULL,

	PRIMARY KEY (instance_id),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE
);

CREATE TABLE recent_instance_icon_configs (
	background TEXT NOT NULL,
	symbol TEXT NOT NULL,
	used_at INTEGER NOT NULL,

	PRIMARY KEY (background, symbol)
);

CREATE INDEX recent_instance_icon_configs_used_at
	ON recent_instance_icon_configs(used_at DESC);
