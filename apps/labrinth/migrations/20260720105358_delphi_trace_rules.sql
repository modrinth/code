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
	priority INTEGER NOT NULL DEFAULT 0,
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

DROP VIEW delphi_issue_details_with_statuses;

CREATE VIEW delphi_issue_details_with_statuses AS
SELECT
	drid.id,
	drid.issue_id,
	drid.key,
	drid.jar,
	drid.file_path,
	drid.decompiled_source,
	drid.data,
	drid.severity AS original_severity,
	COALESCE(dre.severity, drid.severity) AS severity,
	COALESCE(dre.hidden, FALSE) AS hidden,
	m.id AS project_id,
	didv.verdict AS local_status,
	dgdv.verdict AS global_status,
	COALESCE(dgdv.verdict, didv.verdict, 'pending') AS status
FROM delphi_report_issue_details drid
INNER JOIN delphi_report_issues dri ON dri.id = drid.issue_id
INNER JOIN delphi_reports dr ON dr.id = dri.report_id
INNER JOIN files f ON f.id = dr.file_id
INNER JOIN versions v ON v.id = f.version_id
INNER JOIN mods m ON m.id = v.mod_id
LEFT JOIN delphi_global_detail_verdicts dgdv
	ON drid.key = dgdv.detail_key
LEFT JOIN delphi_issue_detail_verdicts didv
	ON m.id = didv.project_id
	AND drid.key = didv.detail_key
LEFT JOIN (
	SELECT revision
	FROM delphi_rule_revisions
	LIMIT 1
) drr ON TRUE
LEFT JOIN delphi_rule_effects dre
	ON dre.revision = drr.revision
	AND dre.detail_id = drid.id;

CREATE TABLE delphi_tech_review_queue (
	project_id BIGINT PRIMARY KEY REFERENCES mods(id)
		ON DELETE CASCADE
);

INSERT INTO delphi_tech_review_queue (project_id)
SELECT DISTINCT didws.project_id
FROM delphi_issue_details_with_statuses didws
INNER JOIN delphi_report_issues dri ON dri.id = didws.issue_id
WHERE
	(
		dri.issue_type = '__dummy'
		AND didws.status = 'pending'
	)
	OR (
		dri.issue_type != '__dummy'
		AND didws.status IN ('pending', 'unsafe')
		AND NOT didws.hidden
	);

DELETE FROM delphi_report_issue_details detail
USING delphi_report_issues issue
WHERE
	detail.issue_id = issue.id
	AND issue.issue_type = '__dummy';

DELETE FROM delphi_report_issues
WHERE issue_type = '__dummy';
