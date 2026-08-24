CREATE TABLE instance_content_locks (
	file_id TEXT NOT NULL,

	PRIMARY KEY (file_id),
	FOREIGN KEY (file_id) REFERENCES instance_files(id) ON DELETE CASCADE
);
