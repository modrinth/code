ALTER TABLE delphi_issue_detail_verdicts
	ADD COLUMN updated_at TIMESTAMPTZ;

ALTER TABLE delphi_global_detail_verdicts
	ADD COLUMN updated_at TIMESTAMPTZ;

CREATE INDEX delphi_report_issue_details_issue_id
	ON delphi_report_issue_details (issue_id);

CREATE INDEX threads_messages_thread_id_created_id
	ON threads_messages (thread_id, created DESC, id DESC);
