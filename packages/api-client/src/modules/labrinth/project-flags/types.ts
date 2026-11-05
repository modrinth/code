export interface ProjectFlagRuleParams {
	expression: string
	scan_files_for: Record<string, string>
}

export interface ProjectFlagRule {
	id: number
	name: string
	created_by: number | null
	created_at: string
	updated_by: number | null
	updated_at: string
	params: ProjectFlagRuleParams
}

export type ListProjectFlagRulesParams = {
	limit?: number
	offset?: number
}

export interface CreateProjectFlagRuleRequest {
	name: string
	params: Omit<ProjectFlagRuleParams, 'scan_files_for'> & {
		scan_files_for?: Record<string, string>
	}
}

export interface EvalProjectFlagRuleRequest {
	project_id: string
	params: CreateProjectFlagRuleRequest['params']
}

export type ProjectFlagRuleSchemaResponse = Record<string, ProjectFlagValue>

export interface ProjectFlagEvaluation {
	flags: ProjectFlag[]
	log: string[]
}

export interface ProjectFlag {
	key: string
	retain_if_removed: boolean
	[field: string]: ProjectFlagValue
}

export interface ProjectFlagRun {
	id: string
	status: 'running' | 'done'
	total_projects: number
	num_errors: number
	started_by: number | null
	started_at: string
	completed_at: string | null
	applied_at: string | null
	output: Record<string, never> | null
	error: string | null
}

export interface ProjectFlagRuleResult {
	flags: ProjectFlag[]
	log: string[]
	error: string | null
}

export interface ProjectFlagRunResult {
	project: {
		id: string
		title: string
		slug: string | null
		icon_url: string | null
	}
	rules: Record<string, ProjectFlagRuleResult & { name: string | null }>
	error: string | null
	evaluated_at: string
}

export interface ProjectFlagRunResults {
	results: ProjectFlagRunResult[]
	total: number
}

export type ListProjectFlagRunsParams = ListProjectFlagRulesParams
export type ListProjectFlagRunResultsParams = ListProjectFlagRulesParams

export type ProjectFlagValue =
	| null
	| boolean
	| number
	| string
	| ProjectFlagValue[]
	| { [key: string]: ProjectFlagValue }
