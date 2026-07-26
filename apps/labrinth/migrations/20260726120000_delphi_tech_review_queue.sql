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
