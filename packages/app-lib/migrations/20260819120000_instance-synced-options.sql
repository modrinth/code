CREATE TABLE instance_synced_options (
	instance_id TEXT NOT NULL,
	option TEXT NOT NULL,
	enabled INTEGER NOT NULL CHECK (enabled IN (0, 1)),

	PRIMARY KEY (instance_id, option),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE
);

CREATE INDEX instance_synced_options_option_enabled
	ON instance_synced_options(option, enabled);

INSERT INTO instance_synced_options (instance_id, option, enabled)
SELECT instances.id, options.option, options.enabled
FROM instances
CROSS JOIN (
	SELECT 'command_history' AS option, 0 AS enabled
	UNION ALL SELECT 'multiplayer_servers', 0
	UNION ALL SELECT 'creative_hotbars', 0
	UNION ALL SELECT 'screenshots', 1
) AS options;

CREATE TABLE global_synced_options_overrides (
	option TEXT PRIMARY KEY NOT NULL,
	enabled INTEGER NOT NULL CHECK (enabled IN (0, 1)),
	default_enabled INTEGER NOT NULL CHECK (default_enabled IN (0, 1))
);

INSERT INTO global_synced_options_overrides
	(option, enabled, default_enabled)
VALUES
	('command_history', 1, 1),
	('multiplayer_servers', 1, 1),
	('creative_hotbars', 1, 1),
	('screenshots', 1, 1);

CREATE TABLE synced_option_materializations (
	instance_id TEXT NOT NULL,
	option TEXT NOT NULL,
	family TEXT NOT NULL DEFAULT '',
	expected_sha1 TEXT,
	baseline BLOB,
	canonical_revision INTEGER NOT NULL DEFAULT 0,
	pending INTEGER NOT NULL DEFAULT 0 CHECK (pending IN (0, 1)),
	link_mode TEXT NOT NULL DEFAULT 'copy',
	PRIMARY KEY (instance_id, option, family),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE
);

CREATE TABLE synced_option_revisions (
	option TEXT PRIMARY KEY NOT NULL,
	revision INTEGER NOT NULL DEFAULT 0,
	initialized INTEGER NOT NULL DEFAULT 0 CHECK (initialized IN (0, 1))
);

INSERT INTO synced_option_revisions (option, revision, initialized)
VALUES
	('multiplayer_servers', 0, 0);

CREATE TABLE synced_server_entries (
	id TEXT PRIMARY KEY NOT NULL,
	nbt BLOB NOT NULL,
	position INTEGER NOT NULL
);

CREATE INDEX synced_server_entries_position
	ON synced_server_entries(position);

CREATE TABLE synced_hotbar_state (
	singleton INTEGER PRIMARY KEY NOT NULL CHECK (singleton = 1),
	nbt BLOB NOT NULL
);

CREATE TABLE instance_server_entries (
	instance_id TEXT NOT NULL,
	id TEXT NOT NULL,
	source TEXT NOT NULL,
	canonical_id TEXT,
	nbt BLOB NOT NULL,
	position INTEGER NOT NULL,
	PRIMARY KEY (instance_id, id),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE
);

CREATE INDEX instance_server_entries_source
	ON instance_server_entries(instance_id, source, position);

CREATE TABLE instance_server_snapshots (
	instance_id TEXT NOT NULL,
	server_id TEXT NOT NULL,
	source TEXT NOT NULL,
	nbt BLOB NOT NULL,
	position INTEGER NOT NULL,
	PRIMARY KEY (instance_id, server_id),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE
);

CREATE TABLE instance_server_baselines (
	instance_id TEXT PRIMARY KEY NOT NULL,
	version_id TEXT,
	reconstructed INTEGER NOT NULL CHECK (reconstructed IN (0, 1)),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE
);
