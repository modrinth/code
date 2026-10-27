CREATE TABLE hidden_shared_instances (
	user_id TEXT NOT NULL,
	shared_instance_id TEXT NOT NULL,
	hidden_at INTEGER NOT NULL,

	PRIMARY KEY (user_id, shared_instance_id)
);
