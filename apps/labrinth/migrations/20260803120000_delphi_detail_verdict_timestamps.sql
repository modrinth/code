ALTER TABLE delphi_issue_detail_verdicts
	ADD COLUMN updated_at TIMESTAMPTZ;

ALTER TABLE delphi_global_detail_verdicts
	ADD COLUMN updated_at TIMESTAMPTZ;
