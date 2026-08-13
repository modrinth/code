-- Development fixtures for the Delphi trace-rule editor and scanner.
-- This file is idempotent and only replaces rows using the fixture namespace.

BEGIN;

DELETE FROM delphi_rule_effects
WHERE rule_id IN (
	SELECT id
	FROM delphi_rules
	WHERE name LIKE '[DEV trace-rule fixture]%'
);

DELETE FROM delphi_reports
WHERE artifact_url = 'https://example.invalid/fixtures/delphi-trace-rules.json';

DELETE FROM delphi_rules
WHERE name LIKE '[DEV trace-rule fixture]%';

WITH fixture_report AS (
	INSERT INTO delphi_reports (
		file_id,
		delphi_version,
		artifact_url,
		severity
	)
	VALUES (
		NULL,
		18,
		'https://example.invalid/fixtures/delphi-trace-rules.json',
		'malware'
	)
	RETURNING id
), fixture_issues AS (
	INSERT INTO delphi_report_issues (report_id, issue_type)
	SELECT fixture_report.id, fixture_issue.issue_type
	FROM fixture_report
	CROSS JOIN (
		VALUES
			('OBFUSCATED_NAMES'),
			('SUSPICIOUS_NETWORK_ACCESS'),
			('RUNTIME_EXEC_USAGE'),
			('BUNDLED_LIBRARY'),
			('HARDCODED_URL'),
			('CRYPTO_MINING_SIGNATURE'),
			('CLASSLOADER_USAGE'),
			('NATIVE_LIBRARY_LOAD')
	) AS fixture_issue(issue_type)
	RETURNING id, issue_type
)
INSERT INTO delphi_report_issue_details (
	issue_id,
	key,
	jar,
	file_path,
	decompiled_source,
	data,
	severity
)
SELECT
	fixture_issues.id,
	fixture_detail.key,
	fixture_detail.jar,
	fixture_detail.file_path,
	fixture_detail.decompiled_source,
	fixture_detail.data,
	fixture_detail.severity::delphi_severity
FROM fixture_issues
INNER JOIN (
	VALUES
		(
			'OBFUSCATED_NAMES',
			'dev-trace-rules/obfuscation/known-bootstrap',
			'META-INF/jars/bootstrap.jar',
			'com/example/bootstrap/Bootstrap.class',
			'class Bootstrap { /* generated fixture */ }',
			'{"confidence": 0.99, "symbol_count": 58}'::jsonb,
			'high'
		),
		(
			'OBFUSCATED_NAMES',
			'dev-trace-rules/obfuscation/unknown-library',
			NULL,
			'com/example/internal/A.class',
			'class A { /* generated fixture */ }',
			'{"confidence": 0.71, "symbol_count": 240}'::jsonb,
			'high'
		),
		(
			'SUSPICIOUS_NETWORK_ACCESS',
			'dev-trace-rules/network/malware-host',
			NULL,
			'com/example/network/Backdoor.class',
			'class Backdoor { /* generated fixture */ }',
			'{"host": "evil.example", "port": 4444, "protocol": "tcp"}'::jsonb,
			'medium'
		),
		(
			'SUSPICIOUS_NETWORK_ACCESS',
			'dev-trace-rules/network/telemetry-host',
			NULL,
			'com/example/telemetry/TelemetryClient.class',
			'class TelemetryClient { /* generated fixture */ }',
			'{"host": "telemetry.example.com", "port": 443, "protocol": "https"}'::jsonb,
			'medium'
		),
		(
			'SUSPICIOUS_NETWORK_ACCESS',
			'dev-trace-rules/network/unmatched-host',
			NULL,
			'com/example/update/UpdateChecker.class',
			'class UpdateChecker { /* generated fixture */ }',
			'{"host": "api.github.com", "port": 443, "protocol": "https"}'::jsonb,
			'low'
		),
		(
			'RUNTIME_EXEC_USAGE',
			'dev-trace-rules/runtime/powershell',
			NULL,
			'com/example/platform/WindowsHelper.class',
			'class WindowsHelper { /* generated fixture */ }',
			'{"command": "powershell.exe", "arguments": ["-EncodedCommand", "fixture"]}'::jsonb,
			'high'
		),
		(
			'RUNTIME_EXEC_USAGE',
			'dev-trace-rules/runtime/desktop-opener',
			NULL,
			'com/example/platform/DesktopBrowser.class',
			'class DesktopBrowser { /* generated fixture */ }',
			'{"command": "xdg-open", "arguments": ["https://example.com/docs"]}'::jsonb,
			'medium'
		),
		(
			'RUNTIME_EXEC_USAGE',
			'dev-trace-rules/runtime/unmatched-java',
			NULL,
			'com/example/launcher/JavaLauncher.class',
			'class JavaLauncher { /* generated fixture */ }',
			'{"command": "java", "arguments": ["-version"]}'::jsonb,
			'low'
		),
		(
			'BUNDLED_LIBRARY',
			'dev-trace-rules/library/example-helper',
			'META-INF/jars/example-helper.jar',
			'com/example/helper/Helper.class',
			NULL,
			'{"library": "example-helper", "version": "1.0.0"}'::jsonb,
			'high'
		),
		(
			'BUNDLED_LIBRARY',
			'dev-trace-rules/library/kotlin-runtime',
			'META-INF/jars/kotlin-stdlib.jar',
			'kotlin/collections/CollectionsKt.class',
			NULL,
			'{"library": "kotlin-stdlib", "version": "2.1.0"}'::jsonb,
			'medium'
		),
		(
			'HARDCODED_URL',
			'dev-trace-rules/url/known-documentation',
			NULL,
			'com/example/help/DocumentationLink.class',
			'class DocumentationLink { /* generated fixture */ }',
			'{"url": "https://docs.example.com/help"}'::jsonb,
			'low'
		),
		(
			'HARDCODED_URL',
			'dev-trace-rules/url/unmatched-webhook',
			NULL,
			'com/example/integration/WebhookClient.class',
			'class WebhookClient { /* generated fixture */ }',
			'{"url": "https://hooks.example.net/incoming"}'::jsonb,
			'medium'
		),
		(
			'CRYPTO_MINING_SIGNATURE',
			'dev-trace-rules/crypto/randomx-miner',
			NULL,
			'com/example/worker/NativeWorker.class',
			'class NativeWorker { /* generated fixture */ }',
			'{"algorithm": "randomx", "pool": "pool.example"}'::jsonb,
			'severe'
		),
		(
			'CLASSLOADER_USAGE',
			'dev-trace-rules/classloader/plugin-loader',
			'META-INF/jars/plugin-api.jar',
			'com/example/plugin/PluginLoader.class',
			'class PluginLoader { /* generated fixture */ }',
			'{"purpose": "plugin-discovery", "parent_first": true}'::jsonb,
			'high'
		),
		(
			'NATIVE_LIBRARY_LOAD',
			'dev-trace-rules/native/unmatched-jna',
			'META-INF/jars/jna.jar',
			'com/sun/jna/Native.class',
			NULL,
			'{"library": "jnidispatch", "platform": "linux-x86_64"}'::jsonb,
			'high'
		)
) AS fixture_detail(
	issue_type,
	key,
	jar,
	file_path,
	decompiled_source,
	data,
	severity
) USING (issue_type);

INSERT INTO delphi_rules (name, rule, priority, revision)
VALUES
	(
		'[DEV trace-rule fixture] Escalate known malware host',
		'trace.issue_type == "SUSPICIOUS_NETWORK_ACCESS" && trace.data.host == "evil.example" ? {"severity": "malware"} : null',
		1000,
		(SELECT revision FROM delphi_rule_revisions LIMIT 1)
	),
	(
		'[DEV trace-rule fixture] Escalate crypto-mining signatures',
		'trace.issue_type == "CRYPTO_MINING_SIGNATURE" && trace.data.algorithm == "randomx" ? {"severity": "malware"} : null',
		900,
		(SELECT revision FROM delphi_rule_revisions LIMIT 1)
	),
	(
		'[DEV trace-rule fixture] Escalate encoded PowerShell',
		'trace.issue_type == "RUNTIME_EXEC_USAGE" && trace.data.command == "powershell.exe" ? {"severity": "severe"} : null',
		800,
		(SELECT revision FROM delphi_rule_revisions LIMIT 1)
	),
	(
		'[DEV trace-rule fixture] Hide known-safe obfuscation',
		'trace.issue_type == "OBFUSCATED_NAMES" && trace.data.confidence >= 0.95 ? {"severity": "hidden"} : null',
		700,
		(SELECT revision FROM delphi_rule_revisions LIMIT 1)
	),
	(
		'[DEV trace-rule fixture] Downgrade known telemetry',
		'trace.issue_type == "SUSPICIOUS_NETWORK_ACCESS" && trace.data.host == "telemetry.example.com" ? {"severity": "low"} : null',
		600,
		(SELECT revision FROM delphi_rule_revisions LIMIT 1)
	),
	(
		'[DEV trace-rule fixture] Downgrade desktop URL openers',
		'trace.issue_type == "RUNTIME_EXEC_USAGE" && trace.data.command == "xdg-open" ? {"severity": "low"} : null',
		500,
		(SELECT revision FROM delphi_rule_revisions LIMIT 1)
	),
	(
		'[DEV trace-rule fixture] Downgrade bundled libraries',
		'trace.issue_type == "BUNDLED_LIBRARY" ? {"severity": "low"} : null',
		400,
		(SELECT revision FROM delphi_rule_revisions LIMIT 1)
	),
	(
		'[DEV trace-rule fixture] Hide documentation URLs',
		'trace.issue_type == "HARDCODED_URL" && trace.data.url == "https://docs.example.com/help" ? {"severity": "hidden"} : null',
		300,
		(SELECT revision FROM delphi_rule_revisions LIMIT 1)
	),
	(
		'[DEV trace-rule fixture] Downgrade plugin classloaders',
		'trace.issue_type == "CLASSLOADER_USAGE" && trace.data.purpose == "plugin-discovery" ? {"severity": "low"} : null',
		200,
		(SELECT revision FROM delphi_rule_revisions LIMIT 1)
	),
	(
		'[DEV trace-rule fixture] Pending native-library exception',
		'trace.issue_type == "NATIVE_LIBRARY_LOAD" && trace.data.library == "jnidispatch" ? {"severity": "low"} : null',
		100,
		(SELECT revision + 1 FROM delphi_rule_revisions LIMIT 1)
	);

INSERT INTO delphi_rule_effects (revision, detail_id, rule_id, severity)
SELECT
	(SELECT revision FROM delphi_rule_revisions LIMIT 1),
	detail.id,
	rule.id,
	fixture_effect.severity::delphi_severity
FROM (
	VALUES
		(
			'dev-trace-rules/network/malware-host',
			'[DEV trace-rule fixture] Escalate known malware host',
			'malware'
		),
		(
			'dev-trace-rules/crypto/randomx-miner',
			'[DEV trace-rule fixture] Escalate crypto-mining signatures',
			'malware'
		),
		(
			'dev-trace-rules/runtime/powershell',
			'[DEV trace-rule fixture] Escalate encoded PowerShell',
			'severe'
		),
		(
			'dev-trace-rules/obfuscation/known-bootstrap',
			'[DEV trace-rule fixture] Hide known-safe obfuscation',
			'hidden'
		),
		(
			'dev-trace-rules/network/telemetry-host',
			'[DEV trace-rule fixture] Downgrade known telemetry',
			'low'
		),
		(
			'dev-trace-rules/runtime/desktop-opener',
			'[DEV trace-rule fixture] Downgrade desktop URL openers',
			'low'
		),
		(
			'dev-trace-rules/library/example-helper',
			'[DEV trace-rule fixture] Downgrade bundled libraries',
			'low'
		),
		(
			'dev-trace-rules/library/kotlin-runtime',
			'[DEV trace-rule fixture] Downgrade bundled libraries',
			'low'
		),
		(
			'dev-trace-rules/url/known-documentation',
			'[DEV trace-rule fixture] Hide documentation URLs',
			'hidden'
		),
		(
			'dev-trace-rules/classloader/plugin-loader',
			'[DEV trace-rule fixture] Downgrade plugin classloaders',
			'low'
		)
) AS fixture_effect(detail_key, rule_name, severity)
INNER JOIN delphi_report_issue_details detail
	ON detail.key = fixture_effect.detail_key
INNER JOIN delphi_rules rule
	ON rule.name = fixture_effect.rule_name;

COMMIT;
