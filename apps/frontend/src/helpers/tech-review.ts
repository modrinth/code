import { DUMMY_ISSUE_TYPES, DUMMY_ISSUES, type DelphiIssueResult } from './tech-review.dummy'

// TODO: @modrinth/api-client package

export type OrderBy =
	| 'created_asc'
	| 'created_desc'
	| 'pending_status_first'
	| 'severity_asc'
	| 'severity_desc'

export interface FetchIssuesParams {
	type?: string | null
	status?: 'pending' | 'approved' | 'rejected' | null
	order_by?: OrderBy | null
	count?: number
	offset?: number
}

export async function fetchIssueTypeSchema(): Promise<string[]> {
	try {
		const schema = await useBaseFetch('internal/delphi/issue_type/schema', { internal: true })
		// Expecting a JSON object map of type -> metadata; return its keys
		if (schema && typeof schema === 'object') {
			return Object.keys(schema as Record<string, unknown>)
		}
		return DUMMY_ISSUE_TYPES
	} catch {
		return DUMMY_ISSUE_TYPES
	}
}

export async function fetchDelphiIssues(params: FetchIssuesParams): Promise<DelphiIssueResult[]> {
	const query = new URLSearchParams()
	if (params.type) query.set('type', params.type)
	if (params.status) query.set('status', params.status)
	if (params.order_by) query.set('order_by', params.order_by)
	if (params.count != null) query.set('count', String(params.count))
	if (params.offset != null) query.set('offset', String(params.offset))

	try {
		const res = await useBaseFetch(`internal/delphi/issues?${query.toString()}`, { internal: true })
		return (res as any[]) || []
	} catch {
		return DUMMY_ISSUES
	}
}
