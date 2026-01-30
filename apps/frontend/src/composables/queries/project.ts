import type { AbstractModrinthClient } from '@modrinth/api-client'

const STALE_TIME = 1000 * 60 * 5 // 5 minutes

export const projectQueryOptions = {
	v2: (projectId: string, client: AbstractModrinthClient) => ({
		queryKey: ['project', 'v2', projectId] as const,
		queryFn: () => client.labrinth.projects_v2.get(projectId),
		staleTime: STALE_TIME,
	}),

	v3: (projectId: string, client: AbstractModrinthClient) => ({
		queryKey: ['project', 'v3', projectId] as const,
		queryFn: () => client.labrinth.projects_v3.get(projectId),
		staleTime: STALE_TIME,
	}),

	members: (projectId: string, client: AbstractModrinthClient) => ({
		queryKey: ['project', projectId, 'members'] as const,
		queryFn: () => client.labrinth.projects_v3.getMembers(projectId),
		staleTime: STALE_TIME,
	}),

	dependencies: (projectId: string, client: AbstractModrinthClient) => ({
		queryKey: ['project', projectId, 'dependencies'] as const,
		queryFn: () => client.labrinth.projects_v2.getDependencies(projectId),
		staleTime: STALE_TIME,
	}),

	versionsV2: (projectId: string, client: AbstractModrinthClient) => ({
		queryKey: ['project', projectId, 'versions', 'v2'] as const,
		queryFn: () =>
			client.labrinth.versions_v3.getProjectVersions(projectId, { include_changelog: false }),
		staleTime: STALE_TIME,
	}),

	versionsV3: (projectId: string, client: AbstractModrinthClient) => ({
		queryKey: ['project', projectId, 'versions', 'v3'] as const,
		queryFn: () =>
			client.labrinth.versions_v3.getProjectVersions(projectId, {
				include_changelog: false,
				apiVersion: 3,
			}),
		staleTime: STALE_TIME,
	}),

	organization: (projectId: string, client: AbstractModrinthClient) => ({
		queryKey: ['project', projectId, 'organization'] as const,
		queryFn: () => client.labrinth.projects_v3.getOrganization(projectId),
		staleTime: STALE_TIME,
	}),
}
