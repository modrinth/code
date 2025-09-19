// Dummy data for the technical review queue, used when backend is unavailable

export type DelphiReportSeverity = 'LOW' | 'MEDIUM' | 'HIGH' | 'SEVERE'
export type DelphiReportIssueStatus = 'pending' | 'approved' | 'rejected'

export interface DelphiIssueJavaClass {
	id: number
	issue_id: number
	internal_class_name: string
	decompiled_source?: string | null
}

export interface DelphiReportSummary {
	id: number
	file_id?: number | null
	delphi_version: number
	artifact_url: string
	created: string // ISO date
	severity: DelphiReportSeverity
}

export interface DelphiIssueSummary {
	id: number
	report_id: number
	issue_type: string
	status: DelphiReportIssueStatus
}

export interface DelphiIssueResult {
	issue: DelphiIssueSummary
	report: DelphiReportSummary
	java_classes: DelphiIssueJavaClass[]
	project_id?: number | null
	project_published?: string | null
}

export const DUMMY_ISSUE_TYPES: string[] = [
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
]

export const DUMMY_ISSUES: DelphiIssueResult[] = [
	{
		issue: {
			id: 1001,
			report_id: 501,
			issue_type: 'suspicious_classes',
			status: 'pending',
		},
		report: {
			id: 501,
			file_id: 90001,
			delphi_version: 47,
			artifact_url: 'https://cdn.modrinth.com/data/abc/versions/1.0.0.jar',
			created: new Date(Date.now() - 3 * 24 * 3600 * 1000).toISOString(),
			severity: 'SEVERE',
		},
		java_classes: [
			{
				id: 7001,
				issue_id: 1001,
				internal_class_name: 'com/example/Suspect',
				decompiled_source: 'public class Suspect { /* ... */ }',
			},
		],
		project_id: 123456,
		project_published: new Date(Date.now() - 30 * 24 * 3600 * 1000).toISOString(),
	},
	{
		issue: {
			id: 1002,
			report_id: 502,
			issue_type: 'url_usage',
			status: 'pending',
		},
		report: {
			id: 502,
			file_id: 90002,
			delphi_version: 47,
			artifact_url: 'https://cdn.modrinth.com/data/def/versions/2.3.4.jar',
			created: new Date(Date.now() - 1 * 24 * 3600 * 1000).toISOString(),
			severity: 'HIGH',
		},
		java_classes: [],
		project_id: 789012,
		project_published: new Date(Date.now() - 45 * 24 * 3600 * 1000).toISOString(),
	},
]
