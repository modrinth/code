CREATE TABLE instance_servers_new (
	instance_id TEXT NOT NULL,
	id TEXT NOT NULL,
	source TEXT NOT NULL CHECK (source IN ('modpack', 'linked_server_project', 'local_desynced')),
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

INSERT INTO instance_servers_new
	(instance_id, id, source, excluded_synced_server_id, nbt, position)
SELECT instance_id, id, source, excluded_synced_server_id, nbt, position
FROM instance_servers;

DROP TABLE instance_servers;

ALTER TABLE instance_servers_new RENAME TO instance_servers;
