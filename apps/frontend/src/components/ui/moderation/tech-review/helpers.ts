import type { Labrinth } from '@modrinth/api-client'

import type { DetailDecision, FlattenedFileReport } from './types'

export const severityOrder: Record<Labrinth.TechReview.Internal.DelphiSeverity, number> = {
	severe: 3,
	high: 2,
	medium: 1,
	low: 0,
}

export function getSeverityBadgeColor(
	severity: Labrinth.TechReview.Internal.DelphiSeverity,
): string {
	switch (severity) {
		case 'severe':
			return 'border-red/60 border bg-highlight-red text-red'
		case 'high':
			return 'border-orange/60 border bg-highlight-orange text-orange'
		case 'medium':
			return 'border-green/60 border bg-highlight-green text-green'
		case 'low':
		default:
			return 'border-blue/60 border bg-highlight-blue text-blue'
	}
}

export function truncateMiddle(str: string, maxLength = 120): string {
	if (str.length <= maxLength) return str
	const keep = maxLength - 3
	const front = Math.ceil(keep / 3)
	return str.slice(0, front) + '...' + str.slice(front - keep)
}

export function getFileHighestSeverity(
	file: FlattenedFileReport,
): Labrinth.TechReview.Internal.DelphiSeverity {
	let highest: Labrinth.TechReview.Internal.DelphiSeverity = 'low'
	for (const issue of file.issues) {
		for (const detail of issue.details) {
			if (severityOrder[detail.severity] > severityOrder[highest]) {
				highest = detail.severity
			}
		}
	}
	return highest
}

export function getFileDetailCount(file: FlattenedFileReport): number {
	return file.issues.reduce((sum, issue) => sum + issue.details.length, 0)
}

export function flattenFileReports(
	versions: Labrinth.TechReview.Internal.VersionReport[],
): FlattenedFileReport[] {
	return versions.flatMap((version) =>
		version.files.map((file) => ({
			...file,
			id: file.report_id,
			version_id: version.version_id,
			version_number: version.version_number,
		})),
	)
}

export function getVersionLabel(file: FlattenedFileReport): string {
	return file.version_number || file.version_id
}

export function getVersionPageHref(
	project: { id: string; slug?: string; project_types: string[] },
	versionId: string,
): string {
	return `/${project.project_types[0] ?? 'project'}/${project.slug ?? project.id}/version/${versionId}`
}

export function verdictToDecision(
	verdict: Labrinth.TechReview.Internal.DelphiReportIssueStatus,
): DetailDecision {
	if (verdict === 'safe') return 'safe'
	if (verdict === 'unsafe') return 'malware'
	return 'pending'
}

export function decisionToVerdict(
	decision: Exclude<DetailDecision, 'pending'>,
): Labrinth.TechReview.Internal.DelphiReportIssueStatus {
	return decision === 'safe' ? 'safe' : 'unsafe'
}

export function statusMatchesDecision(
	status: Labrinth.TechReview.Internal.DelphiReportIssueStatus | null,
	decision: DetailDecision,
): boolean {
	if (status === 'safe') return decision === 'safe'
	if (status === 'unsafe') return decision === 'malware'
	return false
}

export function canUpdateGlobalDetail(
	detail: Labrinth.TechReview.Internal.ReportIssueDetail,
): boolean {
	return detail.key.length > 0 && !detail.key.startsWith('<no-key-')
}
