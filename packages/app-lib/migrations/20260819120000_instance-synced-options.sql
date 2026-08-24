CREATE TABLE instance_synced_options (
	instance_id TEXT NOT NULL,
	option TEXT NOT NULL,
	enabled INTEGER NOT NULL CHECK (enabled IN (0, 1)),

	PRIMARY KEY (instance_id, option),
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE
);

CREATE INDEX instance_synced_options_option_enabled
	ON instance_synced_options(option, enabled);

INSERT INTO instance_synced_options (instance_id, option, enabled)
SELECT id, 'screenshots', 1
FROM instances;
