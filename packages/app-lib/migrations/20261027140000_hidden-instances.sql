CREATE TABLE hidden_instances (
	user_id TEXT NOT NULL,
	instance_id TEXT NOT NULL,
	hidden_at INTEGER NOT NULL,

	PRIMARY KEY (user_id, instance_id)
);
