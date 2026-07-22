CREATE TABLE delphi_rules (
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	created_by BIGINT REFERENCES users(id)
		ON DELETE SET NULL,
	updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_by BIGINT REFERENCES users(id)
		ON DELETE SET NULL,
	name VARCHAR(256) NOT NULL,
	rule VARCHAR(65536) NOT NULL,
	revision BIGINT NOT NULL DEFAULT 0,
	delete_on_next_revision BOOL NOT NULL DEFAULT FALSE
);

CREATE TABLE delphi_rule_revisions (
	revision BIGINT PRIMARY KEY
);

INSERT INTO delphi_rule_revisions (revision)
VALUES (1);

CREATE TABLE delphi_rule_effects (
	revision BIGINT NOT NULL,
	detail_id BIGINT NOT NULL REFERENCES delphi_report_issue_details(id)
		ON DELETE CASCADE
		ON UPDATE CASCADE,
	rule_id BIGINT NOT NULL REFERENCES delphi_rules(id)
		ON UPDATE CASCADE,
	severity delphi_severity,
	hidden BOOLEAN NOT NULL DEFAULT FALSE,
	PRIMARY KEY (revision, detail_id)
);

CREATE INDEX delphi_rule_effects_rule_id ON delphi_rule_effects(rule_id);
