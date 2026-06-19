CREATE TEMP TABLE finish_instance_profile_migration_validation (
	error_count INTEGER NOT NULL CHECK (error_count = 0)
);

INSERT INTO finish_instance_profile_migration_validation (error_count)
SELECT COUNT(*)
FROM profiles
LEFT JOIN instances ON instances.path = profiles.path
WHERE instances.id IS NULL;

INSERT INTO finish_instance_profile_migration_validation (error_count)
SELECT COUNT(*)
FROM instances
LEFT JOIN instance_content_sets
	ON instance_content_sets.id = instances.applied_content_set_id
	AND instance_content_sets.instance_id = instances.id
WHERE instances.applied_content_set_id IS NULL
	OR instance_content_sets.id IS NULL;

DROP TABLE finish_instance_profile_migration_validation;

DROP INDEX processes_profile_path;
ALTER TABLE processes RENAME TO processes_old;

CREATE TABLE processes (
	pid INTEGER NOT NULL,
	start_time INTEGER NOT NULL,
	name TEXT NOT NULL,
	executable TEXT NOT NULL,
	instance_id TEXT NOT NULL,
	post_exit_command TEXT NULL,

	UNIQUE (pid),
	PRIMARY KEY (pid),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE
);

INSERT INTO processes (
	pid,
	start_time,
	name,
	executable,
	instance_id,
	post_exit_command
)
SELECT
	processes_old.pid,
	processes_old.start_time,
	processes_old.name,
	processes_old.executable,
	instances.id,
	processes_old.post_exit_command
FROM processes_old
JOIN instances ON instances.path = processes_old.profile_path;

DROP TABLE processes_old;

CREATE INDEX processes_instance_id ON processes(instance_id);

DROP INDEX join_log_profile_path;
ALTER TABLE join_log RENAME TO join_log_old;

CREATE TABLE join_log (
	instance_id TEXT NOT NULL,
	host TEXT NOT NULL,
	port INTEGER NOT NULL,
	join_time INTEGER NOT NULL,

	PRIMARY KEY (instance_id, host, port),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE
);

INSERT INTO join_log (
	instance_id,
	host,
	port,
	join_time
)
SELECT
	instances.id,
	join_log_old.host,
	join_log_old.port,
	join_log_old.join_time
FROM join_log_old
JOIN instances ON instances.path = join_log_old.profile_path;

DROP TABLE join_log_old;

CREATE INDEX join_log_instance_id ON join_log(instance_id);

DROP INDEX attached_world_data_profile_path;
ALTER TABLE attached_world_data RENAME TO attached_world_data_old;

CREATE TABLE attached_world_data (
	instance_id TEXT NOT NULL,
	world_type TEXT CHECK ( world_type in ('singleplayer', 'server') ) NOT NULL,
	world_id TEXT NOT NULL,
	display_status TEXT NOT NULL DEFAULT 'normal',
	project_id TEXT NULL,
	content_kind TEXT NULL,

	PRIMARY KEY (instance_id, world_type, world_id),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE
);

INSERT INTO attached_world_data (
	instance_id,
	world_type,
	world_id,
	display_status,
	project_id,
	content_kind
)
SELECT
	instances.id,
	attached_world_data_old.world_type,
	attached_world_data_old.world_id,
	attached_world_data_old.display_status,
	attached_world_data_old.project_id,
	attached_world_data_old.content_kind
FROM attached_world_data_old
JOIN instances ON instances.path = attached_world_data_old.profile_path;

DROP TABLE attached_world_data_old;

CREATE INDEX attached_world_data_instance_id
	ON attached_world_data(instance_id);

DROP TABLE profiles;
