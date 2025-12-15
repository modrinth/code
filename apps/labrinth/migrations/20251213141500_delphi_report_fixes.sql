ALTER TABLE delphi_reports
DROP COLUMN status;

ALTER TABLE delphi_report_issues
DROP COLUMN status;

ALTER TABLE delphi_report_issue_details
ADD COLUMN status DELPHI_REPORT_ISSUE_STATUS NOT NULL DEFAULT 'pending';
