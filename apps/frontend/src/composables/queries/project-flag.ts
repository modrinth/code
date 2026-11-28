import type { AbstractModrinthClient, ProjectFlagModule } from '@modrinth/api-client'

export const projectFlagDeploymentQueryOptions = (client: AbstractModrinthClient) => ({
	queryKey: ['moderation', 'project-flag', 'deployment'] as const,
	queryFn: () => client.labrinth.project_flags_internal.getDeployment(),
})

export const projectFlagRuleQueryOptions = (client: AbstractModrinthClient, id: string) => ({
	queryKey: ['moderation', 'project-flag', 'rule', id] as const,
	queryFn: () => client.labrinth.project_flags_internal.get(id),
})

export const projectFlagProjectContextQueryKey = [
	'moderation',
	'project-flag',
	'project-context',
] as const

export const projectFlagInputSchemaQueryOptions = (
	client: AbstractModrinthClient,
	projectId?: string,
) => ({
	queryKey: projectId
		? ([...projectFlagProjectContextQueryKey, projectId] as const)
		: (['moderation', 'project-flag', 'input-schema'] as const),
	queryFn: () =>
		projectId
			? client.labrinth.project_flags_internal.getProjectContext(projectId)
			: client.labrinth.project_flags_internal.getRuleSchema(),
	staleTime: 60_000,
})

export const projectFlagModulesQueryOptions = (client: AbstractModrinthClient) => ({
	queryKey: ['moderation', 'project-flag', 'modules'] as const,
	queryFn: async ({ signal }: { signal: AbortSignal }) => {
		const limit = 200
		const modules: ProjectFlagModule[] = []
		for (let offset = 0; ; offset += limit) {
			signal.throwIfAborted()
			const page = await client.labrinth.project_flags_internal.listModules({ limit, offset })
			modules.push(...page)
			if (page.length < limit) return modules
		}
	},
})
