CREATE TABLE instance_quarantines (
	instance_id TEXT PRIMARY KEY NOT NULL,
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE
);
