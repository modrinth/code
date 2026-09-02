CREATE TABLE install_jobs (
	id TEXT NOT NULL PRIMARY KEY,
	instance_id TEXT NULL,
	kind TEXT NOT NULL,
	
	status TEXT NOT NULL,
	state JSONB NOT NULL,

	created INTEGER NOT NULL,
	modified INTEGER NOT NULL,
	finished INTEGER NULL,

	dismissed INTEGER NOT NULL DEFAULT 0,
	FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE SET NULL
);

CREATE INDEX install_jobs_instance_id ON install_jobs(instance_id);
CREATE INDEX install_jobs_status ON install_jobs(status);
CREATE INDEX install_jobs_finished ON install_jobs(finished);
