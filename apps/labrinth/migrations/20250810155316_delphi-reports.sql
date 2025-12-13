CREATE TYPE delphi_severity AS ENUM ('low', 'medium', 'high', 'severe');

CREATE TYPE delphi_report_issue_status AS ENUM ('pending', 'safe', 'unsafe');

-- A Delphi analysis report for a project version
CREATE TABLE delphi_reports (
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	file_id BIGINT REFERENCES files (id)
		ON DELETE SET NULL
		ON UPDATE CASCADE,
	delphi_version INTEGER NOT NULL,
	artifact_url VARCHAR(2048) NOT NULL,
	created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
	severity DELPHI_SEVERITY NOT NULL,
	UNIQUE (file_id, delphi_version)
);
CREATE INDEX delphi_version ON delphi_reports (delphi_version);

-- An issue found in a Delphi report. Every issue belongs to a report,
-- and a report can have zero, one, or more issues attached to it
CREATE TABLE delphi_report_issues (
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	report_id BIGINT NOT NULL REFERENCES delphi_reports (id)
		ON DELETE CASCADE
		ON UPDATE CASCADE,
	issue_type TEXT NOT NULL,
	status DELPHI_REPORT_ISSUE_STATUS NOT NULL,
	UNIQUE (report_id, issue_type)
);
CREATE INDEX delphi_report_issue_by_status_and_type ON delphi_report_issues (status, issue_type);

-- The details of a Delphi report issue, which contain data about a
-- Java class affected by it. Every Delphi report issue details object
-- belongs to a specific issue, and an issue can have zero, one, or
-- more details attached to it. (Some issues may be artifact-wide,
-- or otherwise not really specific to any particular class.)
CREATE TABLE delphi_report_issue_details (
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	issue_id BIGINT NOT NULL REFERENCES delphi_report_issues (id)
		ON DELETE CASCADE
		ON UPDATE CASCADE,
	key TEXT NOT NULL,
	file_path TEXT NOT NULL,
	decompiled_source TEXT,
	data JSONB NOT NULL,
	severity DELPHI_SEVERITY NOT NULL
);
