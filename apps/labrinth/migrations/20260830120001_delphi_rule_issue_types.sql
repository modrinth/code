ALTER TABLE delphi_rules
	ADD COLUMN on_issue_types TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[];
