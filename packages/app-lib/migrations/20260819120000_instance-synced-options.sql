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
SELECT id, 'screenshots', 1
FROM instances;

CREATE TABLE global_synced_options_overrides (
	option TEXT PRIMARY KEY NOT NULL,
	enabled INTEGER NOT NULL CHECK (enabled IN (0, 1))
);

INSERT INTO global_synced_options_overrides (option, enabled)
VALUES
	('command_history', CASE WHEN EXISTS(SELECT 1 FROM instances) THEN 0 ELSE 1 END),
	('multiplayer_servers', CASE WHEN EXISTS(SELECT 1 FROM instances) THEN 0 ELSE 1 END),
	('creative_hotbars', CASE WHEN EXISTS(SELECT 1 FROM instances) THEN 0 ELSE 1 END),
	('screenshots', CASE WHEN EXISTS(SELECT 1 FROM instances) THEN 0 ELSE 1 END);

CREATE TABLE synced_option_materializations (
	instance_id TEXT NOT NULL,
	option TEXT NOT NULL,
	family TEXT NOT NULL DEFAULT '',
	expected_sha1 TEXT,
	link_mode TEXT NOT NULL DEFAULT 'copy',
	PRIMARY KEY (instance_id, option, family),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE
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
