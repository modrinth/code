CREATE TABLE project_disclosures (
	project_id BIGINT NOT NULL REFERENCES mods(id) ON DELETE CASCADE,
	type TEXT NOT NULL,
	metadata JSONB NOT NULL,
	updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
	updated_by BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	set_by_moderator BOOLEAN NOT NULL,
	PRIMARY KEY (project_id, type)
);
