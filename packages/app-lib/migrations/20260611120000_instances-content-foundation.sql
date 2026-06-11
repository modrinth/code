CREATE TABLE instances (
	id TEXT NOT NULL,
	path TEXT NOT NULL,
	applied_content_set_id TEXT NULL,

	install_stage TEXT NOT NULL,
	launcher_feature_version TEXT NOT NULL,
	update_channel TEXT NOT NULL DEFAULT 'release',

	name TEXT NOT NULL,
	icon_path TEXT NULL,

	created INTEGER NOT NULL,
	modified INTEGER NOT NULL,
	last_played INTEGER NULL,

	submitted_time_played INTEGER NOT NULL DEFAULT 0,
	recent_time_played INTEGER NOT NULL DEFAULT 0,

	PRIMARY KEY (id),
	UNIQUE (path)
);

CREATE INDEX instances_path ON instances(path);
CREATE INDEX instances_applied_content_set_id
	ON instances(applied_content_set_id);

CREATE TABLE instance_links (
	instance_id TEXT NOT NULL,
	link_kind TEXT NOT NULL,

	modrinth_project_id TEXT NULL,
	modrinth_version_id TEXT NULL,

	server_project_id TEXT NULL,

	content_project_id TEXT NULL,
	content_version_id TEXT NULL,

	hosting_server_id TEXT NULL,
	hosting_instance_ids JSONB NULL,
	hosting_active_instance_id TEXT NULL,

	shared_instance_id TEXT NULL,

	PRIMARY KEY (instance_id),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE
);

CREATE INDEX instance_links_link_kind ON instance_links(link_kind);
CREATE INDEX instance_links_modrinth_project_id
	ON instance_links(modrinth_project_id);
CREATE INDEX instance_links_server_project_id
	ON instance_links(server_project_id);
CREATE INDEX instance_links_hosting_server_id
	ON instance_links(hosting_server_id);
CREATE INDEX instance_links_hosting_active_instance_id
	ON instance_links(hosting_active_instance_id);
CREATE INDEX instance_links_shared_instance_id
	ON instance_links(shared_instance_id);

CREATE TABLE instance_groups (
	instance_id TEXT NOT NULL,
	group_name TEXT NOT NULL,

	PRIMARY KEY (instance_id, group_name),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE
);

CREATE INDEX instance_groups_group_name ON instance_groups(group_name);

CREATE TABLE instance_launch_overrides (
	instance_id TEXT NOT NULL,

	java_path TEXT NULL,
	extra_launch_args JSONB NULL,
	custom_env_vars JSONB NULL,

	memory INTEGER NULL,
	force_fullscreen INTEGER NULL,
	game_resolution_x INTEGER NULL,
	game_resolution_y INTEGER NULL,

	hook_pre_launch TEXT NULL,
	hook_wrapper TEXT NULL,
	hook_post_exit TEXT NULL,

	PRIMARY KEY (instance_id),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE
);

CREATE TABLE instance_content_sets (
	id TEXT NOT NULL,
	instance_id TEXT NOT NULL,

	name TEXT NOT NULL,
	source_kind TEXT NOT NULL,
	status TEXT NOT NULL,

	game_version TEXT NOT NULL,
	protocol_version INTEGER NULL,
	loader TEXT NOT NULL,
	loader_version TEXT NULL,

	created INTEGER NOT NULL,
	modified INTEGER NOT NULL,

	PRIMARY KEY (id),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE
);

CREATE INDEX instance_content_sets_instance_id
	ON instance_content_sets(instance_id);

CREATE TABLE instance_content_set_remote_refs (
	content_set_id TEXT NOT NULL,
	ref_type TEXT NOT NULL,
	ref_id TEXT NOT NULL,

	PRIMARY KEY (content_set_id, ref_type),
	FOREIGN KEY (content_set_id)
		REFERENCES instance_content_sets(id)
		ON DELETE CASCADE
);

CREATE INDEX instance_content_set_remote_refs_ref
	ON instance_content_set_remote_refs(ref_type, ref_id);

CREATE TABLE instance_content_set_sync_state (
	content_set_id TEXT NOT NULL,
	provider TEXT NOT NULL,

	applied_update_id TEXT NULL,
	latest_available_update_id TEXT NULL,
	checked_at INTEGER NULL,
	status TEXT NOT NULL,

	PRIMARY KEY (content_set_id),
	FOREIGN KEY (content_set_id)
		REFERENCES instance_content_sets(id)
		ON DELETE CASCADE
);

CREATE INDEX instance_content_set_sync_state_provider
	ON instance_content_set_sync_state(provider);

CREATE INDEX instance_content_set_sync_state_latest_available_update_id
	ON instance_content_set_sync_state(latest_available_update_id);

CREATE TABLE instance_files (
	id TEXT NOT NULL,
	instance_id TEXT NOT NULL,

	relative_path TEXT NOT NULL,
	file_name TEXT NOT NULL,
	enabled INTEGER NOT NULL,

	sha1 TEXT NOT NULL,
	size INTEGER NOT NULL,
	missing INTEGER NOT NULL DEFAULT 0,

	added_at INTEGER NOT NULL,
	modified_at INTEGER NOT NULL,

	PRIMARY KEY (id),
	UNIQUE (instance_id, relative_path),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE
);

CREATE INDEX instance_files_instance_id ON instance_files(instance_id);
CREATE INDEX instance_files_sha1 ON instance_files(sha1);
CREATE INDEX instance_files_missing ON instance_files(missing);

CREATE TABLE instance_content_entries (
	id TEXT NOT NULL,
	instance_id TEXT NOT NULL,
	content_set_id TEXT NOT NULL,
	file_id TEXT NULL,

	project_type TEXT NOT NULL,
	project_id TEXT NULL,
	version_id TEXT NULL,

	source_kind TEXT NOT NULL,
	server_requirement TEXT NOT NULL,
	client_requirement TEXT NOT NULL,
	enabled INTEGER NOT NULL DEFAULT 1,

	added_at INTEGER NOT NULL,
	modified_at INTEGER NOT NULL,

	PRIMARY KEY (id),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE,
	FOREIGN KEY (content_set_id)
		REFERENCES instance_content_sets(id)
		ON DELETE CASCADE,
	FOREIGN KEY (file_id) REFERENCES instance_files(id) ON DELETE SET NULL
);

CREATE INDEX instance_content_entries_instance_id
	ON instance_content_entries(instance_id);
CREATE INDEX instance_content_entries_content_set_id
	ON instance_content_entries(content_set_id);
CREATE INDEX instance_content_entries_file_id
	ON instance_content_entries(file_id);
CREATE INDEX instance_content_entries_project_id
	ON instance_content_entries(project_id);
CREATE INDEX instance_content_entries_version_id
	ON instance_content_entries(version_id);
CREATE INDEX instance_content_entries_source_kind
	ON instance_content_entries(source_kind);

CREATE TABLE instance_content_update_checks (
	content_entry_id TEXT NOT NULL,

	update_channel TEXT NOT NULL,
	update_version_id TEXT NULL,
	checked_at INTEGER NOT NULL,

	PRIMARY KEY (content_entry_id),
	FOREIGN KEY (content_entry_id)
		REFERENCES instance_content_entries(id)
		ON DELETE CASCADE
);

CREATE INDEX instance_content_update_checks_update_version_id
	ON instance_content_update_checks(update_version_id);

INSERT INTO instances (
	id,
	path,
	applied_content_set_id,
	install_stage,
	launcher_feature_version,
	update_channel,
	name,
	icon_path,
	created,
	modified,
	last_played,
	submitted_time_played,
	recent_time_played
)
SELECT
	'legacy:' || path,
	path,
	'legacy:' || path || ':default',
	install_stage,
	launcher_feature_version,
	preferred_update_channel,
	name,
	icon_path,
	created,
	modified,
	last_played,
	submitted_time_played,
	recent_time_played
FROM profiles;

INSERT INTO instance_content_sets (
	id,
	instance_id,
	name,
	source_kind,
	status,
	game_version,
	protocol_version,
	loader,
	loader_version,
	created,
	modified
)
SELECT
	'legacy:' || path || ':default',
	'legacy:' || path,
	'Default',
	CASE
		WHEN linked_project_id IS NOT NULL
			AND linked_version_id IS NOT NULL
			AND linked_version_id != ''
			THEN 'modrinth_modpack'
		WHEN linked_project_id IS NOT NULL
			AND (linked_version_id IS NULL OR linked_version_id = '')
			THEN 'server_project'
		ELSE 'local'
	END,
	'available',
	game_version,
	protocol_version,
	mod_loader,
	mod_loader_version,
	created,
	modified
FROM profiles;

INSERT INTO instance_links (
	instance_id,
	link_kind,
	modrinth_project_id,
	modrinth_version_id,
	server_project_id,
	content_project_id,
	content_version_id,
	hosting_server_id,
	hosting_instance_ids,
	hosting_active_instance_id,
	shared_instance_id
)
SELECT
	'legacy:' || path,
	CASE
		WHEN linked_project_id IS NULL
			THEN 'unmanaged'
		WHEN linked_version_id IS NULL OR linked_version_id = ''
			THEN 'server_project'
		ELSE 'modrinth_modpack'
	END,
	CASE
		WHEN linked_version_id IS NOT NULL AND linked_version_id != ''
			THEN linked_project_id
		ELSE NULL
	END,
	CASE
		WHEN linked_version_id IS NOT NULL AND linked_version_id != ''
			THEN linked_version_id
		ELSE NULL
	END,
	CASE
		WHEN linked_project_id IS NOT NULL
			AND (linked_version_id IS NULL OR linked_version_id = '')
			THEN linked_project_id
		ELSE NULL
	END,
	NULL,
	NULL,
	NULL,
	NULL,
	NULL,
	NULL
FROM profiles;

INSERT OR IGNORE INTO instance_groups (instance_id, group_name)
SELECT
	'legacy:' || profiles.path,
	json_each.value
FROM profiles, json_each(profiles.groups);

INSERT INTO instance_launch_overrides (
	instance_id,
	java_path,
	extra_launch_args,
	custom_env_vars,
	memory,
	force_fullscreen,
	game_resolution_x,
	game_resolution_y,
	hook_pre_launch,
	hook_wrapper,
	hook_post_exit
)
SELECT
	'legacy:' || path,
	override_java_path,
	CASE
		WHEN json_type(override_extra_launch_args) = 'null' THEN NULL
		ELSE override_extra_launch_args
	END,
	CASE
		WHEN json_type(override_custom_env_vars) = 'null' THEN NULL
		ELSE override_custom_env_vars
	END,
	override_mc_memory_max,
	override_mc_force_fullscreen,
	override_mc_game_resolution_x,
	override_mc_game_resolution_y,
	override_hook_pre_launch,
	override_hook_wrapper,
	override_hook_post_exit
FROM profiles;
