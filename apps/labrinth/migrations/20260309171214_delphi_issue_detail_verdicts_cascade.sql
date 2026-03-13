ALTER TABLE delphi_issue_detail_verdicts
	DROP CONSTRAINT IF EXISTS delphi_issue_detail_verdicts_project_id_fkey;

ALTER TABLE delphi_issue_detail_verdicts
	ADD CONSTRAINT delphi_issue_detail_verdicts_project_id_fkey
	FOREIGN KEY (project_id)
	REFERENCES mods(id)
	ON DELETE CASCADE
	ON UPDATE CASCADE;
