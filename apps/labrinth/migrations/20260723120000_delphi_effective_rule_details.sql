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
