CREATE TYPE delphi_file_scan_outcome AS ENUM (
	'succeeded',
	'submission_failed',
	'timed_out'
);

CREATE TABLE delphi_file_scans (
	id UUID PRIMARY KEY,
	file_id BIGINT NOT NULL,
	delphi_version INTEGER,
	report_id BIGINT REFERENCES delphi_reports(id)
		ON DELETE SET NULL
		ON UPDATE CASCADE,
	outcome DELPHI_FILE_SCAN_OUTCOME NOT NULL,
	completed_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	error TEXT,
	CONSTRAINT delphi_file_scans_result_consistency CHECK (
		(
			outcome = 'succeeded'
			AND delphi_version IS NOT NULL
			AND error IS NULL
		)
		OR (
			outcome IN ('submission_failed', 'timed_out')
			AND report_id IS NULL
			AND error IS NOT NULL
		)
	)
);

CREATE INDEX delphi_file_scans_file_id_completed_at
	ON delphi_file_scans (file_id, completed_at DESC);
