CREATE TYPE delphi_report_issue_status AS ENUM ('pending', 'approved', 'rejected');

CREATE TYPE delphi_report_issue_type AS ENUM (
	'reflection_indirection',
	'xor_obfuscation',
	'included_libraries',
	'suspicious_binaries',
	'corrupt_classes',
	'suspicious_classes',
	'url_usage',
	'classloader_usage',
	'processbuilder_usage',
	'runtime_exec_usage',
	'jni_usage',
	'main_method',
	'native_loading',
	'malformed_jar',
	'nested_jar_too_deep',
	'failed_decompilation',
	'analysis_failure',
	'malware_easyforme',
	'malware_simplyloader',
	'unknown'
);

-- A Delphi analysis report for a project version
CREATE TABLE delphi_reports (
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	file_id BIGINT REFERENCES files (id)
		ON DELETE SET NULL
		ON UPDATE CASCADE,
	delphi_version INTEGER NOT NULL,
	artifact_url VARCHAR(2048) NOT NULL,
	created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
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
	issue_type DELPHI_REPORT_ISSUE_TYPE NOT NULL,
	status DELPHI_REPORT_ISSUE_STATUS NOT NULL,
	UNIQUE (report_id, issue_type)
);
CREATE INDEX delphi_report_issue_by_status_and_type ON delphi_report_issues (status, issue_type);

-- A Java class affected by a Delphi report issue. Every affected
-- Java class belongs to a specific issue, and an issue can have zero,
-- one, or more affected classes. (Some issues may be artifact-wide,
-- or otherwise not really specific to any particular class.)
CREATE TABLE delphi_report_issue_java_classes (
	id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	issue_id BIGINT NOT NULL REFERENCES delphi_report_issues (id)
		ON DELETE CASCADE
		ON UPDATE CASCADE,
	internal_class_name TEXT NOT NULL,
	decompiled_source TEXT NOT NULL,
	UNIQUE (issue_id, internal_class_name)
);
