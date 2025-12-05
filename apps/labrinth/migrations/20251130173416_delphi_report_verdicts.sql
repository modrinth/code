ALTER TABLE delphi_reports
ADD COLUMN status delphi_report_issue_status NOT NULL DEFAULT 'pending';
