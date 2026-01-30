ALTER TABLE delphi_report_issue_details
DROP COLUMN status;

CREATE TABLE delphi_issue_detail_verdicts (
	project_id BIGINT                     REFERENCES mods(id)
		ON DELETE SET NULL
		ON UPDATE CASCADE,
	detail_key TEXT                       NOT NULL,
	verdict    delphi_report_issue_status NOT NULL,
	PRIMARY KEY (project_id, detail_key)
);

CREATE VIEW delphi_issue_details_with_statuses AS
SELECT
	drid.*,
	m.id AS project_id,
	COALESCE(didv.verdict, 'pending') AS status
FROM delphi_report_issue_details drid
INNER JOIN delphi_report_issues dri ON dri.id = drid.issue_id
INNER JOIN delphi_reports dr ON dr.id = dri.report_id
INNER JOIN files f ON f.id = dr.file_id
INNER JOIN versions v ON v.id = f.version_id
INNER JOIN mods m ON m.id = v.mod_id
LEFT JOIN delphi_issue_detail_verdicts didv
	ON m.id = didv.project_id
	AND drid.key = didv.detail_key;
