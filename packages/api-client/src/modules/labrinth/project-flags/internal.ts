import { AbstractModule } from '../../../core/abstract-module'
import type {
	CreateProjectFlagRuleRequest,
	EvalProjectFlagRuleRequest,
	ListProjectFlagModulesParams,
	ListProjectFlagRulesParams,
	ListProjectFlagRunResultsParams,
	ListProjectFlagRunsParams,
	PersistedProjectFlag,
	ProjectFlagDeployment,
	ProjectFlagEvaluationResult,
	ProjectFlagModule,
	ProjectFlagRule,
	ProjectFlagRuleSchemaResponse,
	ProjectFlagRun,
	ProjectFlagRunActivity,
	ProjectFlagRunResults,
	SaveProjectFlagModuleRequest,
} from './types'

export class LabrinthProjectFlagsInternalModule extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_project_flags_internal'
	}

	public async listModules(
		params: ListProjectFlagModulesParams = {},
	): Promise<ProjectFlagModule[]> {
		return this.client.request<ProjectFlagModule[]>('/moderation/project-flag/modules', {
			api: 'labrinth',
			version: 'internal',
			method: 'GET',
			params,
		})
	}

	public async getModule(name: string): Promise<ProjectFlagModule> {
		return this.client.request<ProjectFlagModule>('/moderation/project-flag/module', {
			api: 'labrinth',
			version: 'internal',
			method: 'GET',
			params: { name },
		})
	}

	public async createModule(body: SaveProjectFlagModuleRequest): Promise<ProjectFlagModule> {
		return this.client.request<ProjectFlagModule>('/moderation/project-flag/modules', {
			api: 'labrinth',
			version: 'internal',
			method: 'POST',
			body,
			retry: false,
		})
	}

	public async updateModule(
		name: string,
		body: SaveProjectFlagModuleRequest,
	): Promise<ProjectFlagModule> {
		return this.client.request<ProjectFlagModule>('/moderation/project-flag/module', {
			api: 'labrinth',
			version: 'internal',
			method: 'PUT',
			params: { name },
			body,
			retry: false,
		})
	}

	public async deleteModule(name: string): Promise<void> {
		return this.client.request<void>('/moderation/project-flag/module', {
			api: 'labrinth',
			version: 'internal',
			method: 'DELETE',
			params: { name },
			retry: false,
		})
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

	public async getProjectFlags(projectId: string): Promise<PersistedProjectFlag[]> {
		return this.client.request<PersistedProjectFlag[]>(
			`/moderation/project-flag/project/${encodeURIComponent(projectId)}`,
			{ api: 'labrinth', version: 'internal', method: 'GET' },
		)
	}

	public async getProjectContext(projectId: string): Promise<ProjectFlagRuleSchemaResponse> {
		return this.client.request<ProjectFlagRuleSchemaResponse>(
			`/moderation/project-flag/project/${encodeURIComponent(projectId)}/context`,
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

	/** Null means Temporal history was deleted or has expired; DB results may still exist. */
	public async getRun(id: string): Promise<ProjectFlagRun | null> {
		return this.client.request<ProjectFlagRun | null>(
			`/moderation/project-flag/rules/run/${encodeURIComponent(id)}`,
			{ api: 'labrinth', version: 'internal', method: 'GET' },
		)
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

	public async getDeployment(): Promise<ProjectFlagDeployment | null> {
		return this.client.request<ProjectFlagDeployment | null>(
			'/moderation/project-flag/deployment',
			{
				api: 'labrinth',
				version: 'internal',
				method: 'GET',
			},
		)
	}

	public async deployRun(id: string): Promise<ProjectFlagDeployment> {
		return this.client.request<ProjectFlagDeployment>(
			`/moderation/project-flag/rules/run/${encodeURIComponent(id)}/deploy`,
			{ api: 'labrinth', version: 'internal', method: 'POST', retry: false },
		)
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

	public async get(id: string): Promise<ProjectFlagRule> {
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
			retry: false,
		})
	}

	public async update(id: string, data: CreateProjectFlagRuleRequest): Promise<ProjectFlagRule> {
		return this.client.request<ProjectFlagRule>(`/moderation/project-flag/rules/${id}`, {
			api: 'labrinth',
			version: 'internal',
			method: 'PUT',
			body: data,
			retry: false,
		})
	}

	public async evalRule(data: EvalProjectFlagRuleRequest): Promise<ProjectFlagEvaluationResult[]> {
		return this.client.request<ProjectFlagEvaluationResult[]>(
			'/moderation/project-flag/rule/eval',
			{
				api: 'labrinth',
				version: 'internal',
				method: 'POST',
				body: data,
				timeout: 60_000,
				retry: false,
			},
		)
	}

	public async delete(id: string): Promise<void> {
		return this.client.request<void>(`/moderation/project-flag/rules/${id}`, {
			api: 'labrinth',
			version: 'internal',
			method: 'DELETE',
		})
	}
}
