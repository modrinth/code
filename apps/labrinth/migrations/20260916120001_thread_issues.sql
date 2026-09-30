CREATE TABLE threads_issues (
	id BIGINT PRIMARY KEY,
	thread_id BIGINT NOT NULL REFERENCES threads(id) ON DELETE CASCADE,
	created_by BIGINT NOT NULL REFERENCES users(id) ON UPDATE CASCADE,
	created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	why JSONB NOT NULL,
	user_addressed BOOLEAN NOT NULL DEFAULT FALSE,
	moderator_verified BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX threads_issues_thread_id
	ON threads_issues(thread_id);

CREATE TABLE threads_issue_facets (
	id BIGINT PRIMARY KEY,
	issue_id BIGINT NOT NULL REFERENCES threads_issues(id) ON DELETE CASCADE,
	what JSONB NOT NULL,
	verdict VARCHAR(64) NOT NULL
);

CREATE INDEX threads_issue_facets_issue_id_id
	ON threads_issue_facets(issue_id, id);
