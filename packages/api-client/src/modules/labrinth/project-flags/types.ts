import type { Labrinth } from '../types'

export interface ProjectFlagModule {
	name: string
	script: string
}

export type ListProjectFlagModulesParams = {
	limit?: number
	offset?: number
}

export type SaveProjectFlagModuleRequest = ProjectFlagModule

export interface ProjectFlagRuleParams {
	expression: string
	scan_files_for: Record<string, string>
	decision_questions: Record<string, string>
}

export interface ProjectFlagRule {
	id: string
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
	params: Omit<ProjectFlagRuleParams, 'scan_files_for' | 'decision_questions'> & {
		scan_files_for?: Record<string, string>
		decision_questions?: Record<string, string>
	}
}

export interface EvalProjectFlagRuleRequest {
	project_ids: string[]
	params: CreateProjectFlagRuleRequest['params']
}

export type ProjectFlagRuleSchemaResponse = Record<string, ProjectFlagValue>

export interface ProjectFlagEvaluation {
	flags: ProjectFlag[]
	log: string[]
}

export interface ProjectFlagEvaluationResult extends ProjectFlagEvaluation {
	project_id: string
	error: string | null
}

export type FlagOutcome = 'withheld' | 'manual_review'

export interface ProjectFlag {
	key: string
	flag_outcome: FlagOutcome
	reason: string | null
	retain_when_resolved: boolean
	[field: string]: ProjectFlagValue
}

export interface PersistedProjectFlag extends ProjectFlag {
	active: boolean
}

export interface ProjectFlagRunActivity {
	num_scanning: number
	num_workers: number
}

export interface ProjectFlagRunUser {
	id: string
	username: string
}

export interface ProjectFlagDeployment {
	ruleset_id: string
	deployed_by: string | null
	deployed_at: string
}

export interface ProjectFlagRun {
	id: string
	ruleset_id: string | null
	status: 'done' | { running: { num_queued: number } }
	/** Projects originally queued, including projects not yet evaluated. */
	total_projects: number
	/** Projects with persisted results, including failed evaluations. */
	num_scanned: number
	/** Successfully evaluated projects with at least one flag. */
	num_flagged: number
	/** Distinct flag violations, including retained flags and partial results from failed evaluations. */
	num_flags: number
	/** Successfully evaluated projects with no flags (would be approved). */
	num_no_flags: number
	/** Projects with any project or rule error, even if also flagged. */
	num_errors: number
	started_by: ProjectFlagRunUser | null
	started_at: string
	completed_at: string | null
	applied_at: string | null
	applied_by: ProjectFlagRunUser | null
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
	/** Snapshot evidence; inactive entries are resolved retained violations. */
	effective_flags: PersistedProjectFlag[]
	/** Distinct emitted and retained flag violations, including results with errors. */
	num_flags: number
	proposed_review_outcome: Labrinth.Projects.v3.ProjectReviewOutcome | null
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
