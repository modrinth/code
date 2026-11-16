export interface SourceView {
	reportId: string
	path?: string
	jar?: string
	lines?: { start: number; end: number }
}

export interface SourceLinkTarget extends SourceView {
	path: string
}

export function sourceQueryKey(query: Record<string, unknown>) {
	return JSON.stringify([
		query.tech_review_report,
		query.tech_review_path,
		query.tech_review_jar,
		query.lines,
	])
}

export function parseSourceLines(value: unknown): SourceLinkTarget['lines'] {
	if (typeof value !== 'string' || !/^\d+-\d+$/.test(value)) return
	const [start, end] = value.split('-').map(Number)
	if (!Number.isSafeInteger(start) || !Number.isSafeInteger(end) || start < 1 || end < start) return
	return { start, end }
}

export function matchesSourceLink(target: SourceLinkTarget, path: string, jar: string | null) {
	return target.path === path && (target.jar === undefined || target.jar === (jar ?? ''))
}
