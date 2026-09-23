CREATE TABLE project_redirects (
	identifier TEXT PRIMARY KEY,
	target_project_id BIGINT NOT NULL REFERENCES mods(id) ON DELETE CASCADE
);

CREATE INDEX project_redirects_target_project_id
	ON project_redirects(target_project_id);
