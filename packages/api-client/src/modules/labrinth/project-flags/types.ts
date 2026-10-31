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
	flags: ProjectFlagValue[]
	log: string[]
}

export type ProjectFlagValue =
	| null
	| boolean
	| number
	| string
	| ProjectFlagValue[]
	| { [key: string]: ProjectFlagValue }
