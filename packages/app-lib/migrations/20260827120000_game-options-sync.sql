-- Game-settings sync starts off. The user must choose which instance supplies
-- the initial values before it can be enabled.
INSERT INTO sync_feature_settings
	(feature, globally_enabled, new_instance_default)
VALUES ('game_options', 0, 1);

INSERT INTO instance_sync_preferences (instance_id, feature, enabled)
SELECT id, 'game_options', 0
FROM instances;

-- Tracks changes to the shared values. The catalog revision changes when we add
-- or update support for Minecraft versions.
CREATE TABLE synced_game_option_state (
	singleton INTEGER PRIMARY KEY NOT NULL CHECK (singleton = 1),
	revision INTEGER NOT NULL CHECK (revision >= 0),
	catalog_revision INTEGER NOT NULL CHECK (catalog_revision >= 1)
);

-- Stores the value shared for each setting, including settings added by mods.
-- A NULL value means the setting was found but has no shared value yet.
CREATE TABLE synced_game_option_values (
	option_id TEXT PRIMARY KEY NOT NULL,
	kind TEXT NOT NULL CHECK (kind IN ('vanilla', 'external')),
	raw_key TEXT,
	canonical_type TEXT NOT NULL,
	canonical_value_json TEXT,
	value_codec TEXT NOT NULL,
	seeded INTEGER NOT NULL CHECK (seeded IN (0, 1)),
	revision INTEGER NOT NULL CHECK (revision >= 0),
	origin TEXT NOT NULL CHECK (origin IN ('app_editor', 'instance', 'source_seed')),
	source_game_version TEXT,
	source_instance_id TEXT,
	updated_at INTEGER NOT NULL
);

-- Stores which settings the user chose to sync. This is separate from the values
-- so turning a setting off does not forget its last value.
CREATE TABLE synced_game_option_preferences (
	option_id TEXT PRIMARY KEY NOT NULL,
	enabled INTEGER NOT NULL CHECK (enabled IN (0, 1)),
	source TEXT NOT NULL CHECK (source IN ('catalog_default', 'discovery_default', 'user')),
	revision INTEGER NOT NULL CHECK (revision >= 0)
);

-- Keeps the options.txt supplied by the installed modpack. After a pack update,
-- the user's shared settings are applied to this file again.
CREATE TABLE instance_game_option_pack_bases (
	instance_id TEXT PRIMARY KEY NOT NULL,
	pack_version_id TEXT,
	source TEXT NOT NULL CHECK (source IN ('client_overrides', 'overrides', 'none')),
	sha1 TEXT,
	encoding TEXT,
	document BLOB,
	FOREIGN KEY (instance_id) REFERENCES instances (id) ON DELETE CASCADE
);

-- Temporarily keeps the local options.txt while the installer removes the old
-- pack. It is restored when the new pack does not include its own options.txt.
CREATE TABLE instance_game_option_update_state (
	instance_id TEXT PRIMARY KEY NOT NULL,
	had_file INTEGER NOT NULL CHECK (had_file IN (0, 1)),
	sha1 TEXT,
	document BLOB,
	FOREIGN KEY (instance_id) REFERENCES instances (id) ON DELETE CASCADE
);
