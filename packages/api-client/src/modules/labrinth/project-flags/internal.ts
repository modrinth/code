import { AbstractModule } from '../../../core/abstract-module'
import type {
	CreateProjectFlagRuleRequest,
	EvalProjectFlagRuleRequest,
	ListProjectFlagRulesParams,
	ProjectFlagEvaluation,
	ProjectFlagRule,
	ProjectFlagRuleSchemaResponse,
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
