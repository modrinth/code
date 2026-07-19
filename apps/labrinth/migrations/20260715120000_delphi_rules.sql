CREATE TABLE delphi_rules (
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	name TEXT NOT NULL CHECK (BTRIM(name) <> ''),
	priority INTEGER NOT NULL DEFAULT 0,
	created TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	created_by BIGINT REFERENCES users (id)
		ON DELETE SET NULL
		ON UPDATE CASCADE,
	updated_by BIGINT REFERENCES users (id)
		ON DELETE SET NULL
		ON UPDATE CASCADE
);

CREATE TABLE delphi_rule_revisions (
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	rule_id BIGINT NOT NULL REFERENCES delphi_rules (id)
		ON DELETE CASCADE
		ON UPDATE CASCADE,
	expression TEXT NOT NULL CHECK (BTRIM(expression) <> ''),
	active BOOLEAN NOT NULL DEFAULT TRUE,
	created TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	created_by BIGINT REFERENCES users (id)
		ON DELETE SET NULL
		ON UPDATE CASCADE
);

CREATE UNIQUE INDEX delphi_rule_revisions_active
	ON delphi_rule_revisions (rule_id)
	WHERE active;

CREATE INDEX delphi_rule_revisions_rule_id
	ON delphi_rule_revisions (rule_id, created DESC);

CREATE TABLE delphi_rule_effects (
	rule_revision_id BIGINT NOT NULL REFERENCES delphi_rule_revisions (id)
		ON DELETE CASCADE
		ON UPDATE CASCADE,
	issue_detail_id BIGINT NOT NULL REFERENCES delphi_report_issue_details (id)
		ON DELETE CASCADE
		ON UPDATE CASCADE,
	severity delphi_severity,
	hidden BOOLEAN NOT NULL DEFAULT FALSE,
	created TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	PRIMARY KEY (rule_revision_id, issue_detail_id),
	CHECK (severity IS NOT NULL OR hidden)
);

CREATE INDEX delphi_rule_effects_issue_detail_id
	ON delphi_rule_effects (issue_detail_id);
