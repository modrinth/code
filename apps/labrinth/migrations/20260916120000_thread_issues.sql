CREATE TABLE threads_issues (
	id BIGINT PRIMARY KEY,
	thread_id BIGINT NOT NULL REFERENCES threads(id) ON DELETE CASCADE,
	created_by BIGINT NOT NULL REFERENCES users(id) ON UPDATE CASCADE,
	created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	what JSONB NOT NULL,
	why JSONB NOT NULL,
	user_addressed BOOLEAN NOT NULL DEFAULT FALSE,
	moderator_verified BOOLEAN NOT NULL DEFAULT FALSE,
	verdict VARCHAR(64) NOT NULL
);

CREATE INDEX threads_issues_thread_id_verdict
	ON threads_issues(thread_id, verdict);
