CREATE TABLE store_blobs (
	sha512 TEXT PRIMARY KEY NOT NULL,
	sha1 TEXT NOT NULL,
	size INTEGER NOT NULL CHECK (size >= 0),
	relative_path TEXT NOT NULL UNIQUE,
	status TEXT NOT NULL DEFAULT 'ready' CHECK (status IN ('ready', 'quarantined', 'deleting')),
	modified_at_ns INTEGER NOT NULL,
	created_at INTEGER NOT NULL DEFAULT (unixepoch()),
	last_used_at INTEGER NOT NULL DEFAULT (unixepoch()),
	verified_at INTEGER NOT NULL DEFAULT (unixepoch()),
	sources TEXT NOT NULL DEFAULT '[]'
);

CREATE INDEX store_blobs_sha1_size ON store_blobs (sha1, size);

CREATE TABLE store_instance_files (
	file_id TEXT PRIMARY KEY NOT NULL REFERENCES instance_files(id) ON DELETE CASCADE,
	blob_sha512 TEXT NOT NULL REFERENCES store_blobs(sha512) ON DELETE RESTRICT,
	materialization_kind TEXT NOT NULL CHECK (materialization_kind IN ('symlink', 'copy'))
);

CREATE INDEX store_instance_files_blob ON store_instance_files(blob_sha512);

CREATE TABLE store_retained_refs (
	owner_kind TEXT NOT NULL,
	owner_id TEXT NOT NULL,
	blob_sha512 TEXT NOT NULL REFERENCES store_blobs(sha512) ON DELETE RESTRICT,
	PRIMARY KEY (owner_kind, owner_id, blob_sha512)
);

CREATE TABLE store_operations (
	id TEXT PRIMARY KEY NOT NULL,
	instance_id TEXT NOT NULL,
	payload TEXT NOT NULL,
	created_at INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE INDEX store_operations_instance ON store_operations(instance_id);

INSERT INTO app_metadata (key, value, updated_at)
VALUES ('store_cache_limit_bytes', '5368709120', unixepoch());
