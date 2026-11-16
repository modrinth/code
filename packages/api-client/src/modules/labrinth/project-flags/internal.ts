import { AbstractModule } from '../../../core/abstract-module'
import type {
	CreateProjectFlagRuleRequest,
	EvalProjectFlagRuleRequest,
	ListProjectFlagRulesParams,
	ListProjectFlagRunResultsParams,
	ListProjectFlagRunsParams,
	ProjectFlagEvaluation,
	ProjectFlagRule,
	ProjectFlagRuleSchemaResponse,
	ProjectFlagRun,
	ProjectFlagRunActivity,
	ProjectFlagRunResults,
} from './types'

export class LabrinthProjectFlagsInternalModule extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_project_flags_internal'
	}

	public async getRuleSchema(): Promise<ProjectFlagRuleSchemaResponse> {
		return this.client.request<ProjectFlagRuleSchemaResponse>(
			'/moderation/project-flag/rules/schema',
			{
				api: 'labrinth',
				version: 'internal',
				method: 'GET',
			},
		)
	}

	public async list(params: ListProjectFlagRulesParams = {}): Promise<ProjectFlagRule[]> {
		return this.client.request<ProjectFlagRule[]>('/moderation/project-flag/rules', {
			api: 'labrinth',
			version: 'internal',
			method: 'GET',
			params,
		})
	}

	public async listRuns(params: ListProjectFlagRunsParams = {}): Promise<ProjectFlagRun[]> {
		return this.client.request<ProjectFlagRun[]>('/moderation/project-flag/rules/run', {
			api: 'labrinth',
			version: 'internal',
			method: 'GET',
			params,
		})
	}

	public async getRunActivity(): Promise<ProjectFlagRunActivity> {
		return this.client.request<ProjectFlagRunActivity>(
			'/moderation/project-flag/rules/run/activity',
			{
				api: 'labrinth',
				version: 'internal',
				method: 'GET',
			},
		)
	}

	public async startRun(): Promise<ProjectFlagRun> {
		return this.client.request<ProjectFlagRun>('/moderation/project-flag/rules/run', {
			api: 'labrinth',
			version: 'internal',
			method: 'POST',
			retry: false,
		})
	}

	public async deleteRun(id: string): Promise<void> {
		return this.client.request<void>(
			`/moderation/project-flag/rules/run/${encodeURIComponent(id)}`,
			{ api: 'labrinth', version: 'internal', method: 'DELETE', retry: false },
		)
	}

	public async getRunResults(
		id: string,
		params: ListProjectFlagRunResultsParams = {},
	): Promise<ProjectFlagRunResults> {
		return this.client.request<ProjectFlagRunResults>(
			`/moderation/project-flag/rules/run/${encodeURIComponent(id)}/results`,
			{
				api: 'labrinth',
				version: 'internal',
				method: 'GET',
				params,
			},
		)
	}

	public async get(id: number): Promise<ProjectFlagRule> {
		return this.client.request<ProjectFlagRule>(`/moderation/project-flag/rules/${id}`, {
			api: 'labrinth',
			version: 'internal',
			method: 'GET',
		})
	}

	public async create(data: CreateProjectFlagRuleRequest): Promise<ProjectFlagRule> {
		return this.client.request<ProjectFlagRule>('/moderation/project-flag/rules', {
			api: 'labrinth',
			version: 'internal',
			method: 'POST',
			body: data,
		})
	}

	public async update(id: number, data: CreateProjectFlagRuleRequest): Promise<ProjectFlagRule> {
		return this.client.request<ProjectFlagRule>(`/moderation/project-flag/rules/${id}`, {
			api: 'labrinth',
			version: 'internal',
			method: 'PUT',
			body: data,
		})
	}

	public async evalRule(data: EvalProjectFlagRuleRequest): Promise<ProjectFlagEvaluation> {
		return this.client.request<ProjectFlagEvaluation>('/moderation/project-flag/rule/eval', {
			api: 'labrinth',
			version: 'internal',
			method: 'POST',
			body: data,
		})
	}

	public async delete(id: number): Promise<void> {
		return this.client.request<void>(`/moderation/project-flag/rules/${id}`, {
			api: 'labrinth',
			version: 'internal',
			method: 'DELETE',
		})
	}
}
