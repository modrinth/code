-- Global switches and defaults for each sync feature.
CREATE TABLE sync_feature_settings (
	feature TEXT PRIMARY KEY NOT NULL,
	globally_enabled INTEGER NOT NULL CHECK (globally_enabled IN (0, 1)),
	new_instance_default INTEGER NOT NULL CHECK (new_instance_default IN (0, 1))
);

INSERT INTO sync_feature_settings
	(feature, globally_enabled, new_instance_default)
VALUES
	('command_history', 1, 1),
	('multiplayer_servers', 1, 1),
	('creative_hotbars', 1, 1),
	('screenshots', 1, 1);

-- Each instance's switches for the sync features.
CREATE TABLE instance_sync_preferences (
	instance_id TEXT NOT NULL,
	feature TEXT NOT NULL,
	enabled INTEGER NOT NULL CHECK (enabled IN (0, 1)),
	PRIMARY KEY (instance_id, feature),
	FOREIGN KEY (instance_id) REFERENCES instances (id)
		ON DELETE CASCADE,
	FOREIGN KEY (feature) REFERENCES sync_feature_settings (feature)
		ON DELETE CASCADE
);

CREATE INDEX instance_sync_preferences_feature_enabled ON instance_sync_preferences (
	feature,
	enabled
);

-- Existing instances pre-update will have screenshots on, everything else off.
INSERT INTO instance_sync_preferences (instance_id, feature, enabled)
SELECT instances.id, features.feature, features.enabled
FROM
	instances
	CROSS JOIN (
		SELECT 'command_history' AS feature, 0 AS enabled
		UNION ALL
		SELECT 'multiplayer_servers', 0
		UNION ALL
		SELECT 'creative_hotbars', 0
		UNION ALL
		SELECT 'screenshots', 1
	) AS features;

-- The file each instance should have, and whether writing it finished.
-- For hotbars, merge_base is the last file we generated.
CREATE TABLE instance_sync_checkpoints (
	instance_id TEXT NOT NULL,
	feature TEXT NOT NULL,
	variant TEXT NOT NULL CHECK (variant IN ('default', 'legacy', 'components')),
	expected_sha1 TEXT NOT NULL,
	merge_base BLOB,
	source_revision INTEGER NOT NULL CHECK (source_revision >= 0),
	status TEXT NOT NULL CHECK (status IN ('pending', 'ready')),
	link_mode TEXT CHECK (link_mode IS NULL OR link_mode IN ('copy', 'hard', 'symbolic')),
	PRIMARY KEY (instance_id, feature, variant),
	FOREIGN KEY (instance_id, feature) REFERENCES instance_sync_preferences (instance_id, feature)
		ON DELETE CASCADE
);

-- The shared hotbars. No row means they have not been set up yet.
CREATE TABLE synced_hotbar_state (
	singleton INTEGER PRIMARY KEY NOT NULL CHECK (singleton = 1),
	schema_version INTEGER NOT NULL CHECK (schema_version >= 1),
	revision INTEGER NOT NULL CHECK (revision >= 0),
	nbt BLOB NOT NULL
);

-- The shared server-list revision after server sync has been set up.
-- This row still exists when the shared server list is empty.
CREATE TABLE synced_server_state (
	singleton INTEGER PRIMARY KEY NOT NULL CHECK (singleton = 1),
	revision INTEGER NOT NULL CHECK (revision >= 0)
);

-- The shared multiplayer server list.
CREATE TABLE synced_servers (
	id TEXT PRIMARY KEY NOT NULL,
	position INTEGER NOT NULL UNIQUE CHECK (position >= 0),
	nbt BLOB NOT NULL
);

-- Servers that only belong to one instance.
-- excluded_synced_server_id is the shared server replaced by a local copy.
CREATE TABLE instance_servers (
	instance_id TEXT NOT NULL,
	id TEXT NOT NULL,
	source TEXT NOT NULL CHECK (source IN ('modpack', 'local_desynced')),
	excluded_synced_server_id TEXT,
	nbt BLOB NOT NULL,
	position INTEGER NOT NULL CHECK (position >= 0),
	PRIMARY KEY (instance_id, id),
	UNIQUE (instance_id, position),
	FOREIGN KEY (instance_id) REFERENCES instances (id)
		ON DELETE CASCADE,
	FOREIGN KEY (excluded_synced_server_id) REFERENCES synced_servers (id)
		ON DELETE SET NULL
);

-- The servers.dat we last wrote for each instance.
-- Used to work out what changed the next time we read it.
CREATE TABLE instance_server_projection_entries (
	instance_id TEXT NOT NULL,
	owner TEXT NOT NULL CHECK (owner IN ('synced', 'instance')),
	server_id TEXT NOT NULL,
	nbt BLOB NOT NULL,
	position INTEGER NOT NULL CHECK (position >= 0),
	PRIMARY KEY (instance_id, owner, server_id),
	UNIQUE (instance_id, position),
	FOREIGN KEY (instance_id) REFERENCES instances (id)
		ON DELETE CASCADE
);

-- Tracks whether we have read the modpack's server list for this instance.
CREATE TABLE instance_server_pack_state (
	instance_id TEXT PRIMARY KEY NOT NULL,
	version_id TEXT,
	FOREIGN KEY (instance_id) REFERENCES instances (id)
		ON DELETE CASCADE
);
