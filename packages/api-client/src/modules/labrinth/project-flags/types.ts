export interface ProjectFlagRule {
	id: number
	name: string
	created_by: number | null
	created_at: string
	updated_by: number | null
	updated_at: string
	expression: string
}

export type ListProjectFlagRulesParams = {
	limit?: number
	offset?: number
}

export interface CreateProjectFlagRuleRequest {
	name: string
	expression: string
}

export interface EvalProjectFlagRuleRequest {
	project_id: string
	expression: string
}

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
