import type { AbstractModrinthClient } from '@modrinth/api-client'

export const projectFlagDeploymentQueryOptions = (client: AbstractModrinthClient) => ({
	queryKey: ['moderation', 'project-flag', 'deployment'] as const,
	queryFn: () => client.labrinth.project_flags_internal.getDeployment(),
})
