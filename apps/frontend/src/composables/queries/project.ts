import type { AbstractModrinthClient } from '@modrinth/api-client'
import type { QueryClient } from '@tanstack/query-core'

export const STALE_TIME = 1000 * 60 * 5 // 5 minutes
export const STALE_TIME_LONG = 1000 * 60 * 10 // 10 minutes

/** Anything with a canonical project ID and optional slug (search hits, full projects, etc.). */
export type ProjectCheckIdentity = {
	id?: string | null
	project_id?: string | null
	slug?: string | null
}

/**
 * Warm `['project', 'check', …]` so navigating to a project by slug skips the check network round-trip.
 * Safe to call with search hits, full project objects, or mixed lists.
 */
export function warmProjectCheckCaches(
	queryClient: QueryClient,
	projects: ProjectCheckIdentity | readonly ProjectCheckIdentity[] | null | undefined,
) {
	const list = projects == null ? [] : Array.isArray(projects) ? projects : [projects]
	for (const project of list) {
		const id = project.id ?? project.project_id
		if (!id) continue
		queryClient.setQueryData(['project', 'check', id], { id })
		if (project.slug) {
			queryClient.setQueryData(['project', 'check', project.slug], { id })
		}
	}
}

/**
 * Project query options.
 *
 * `v2` / `v3` / members / etc. must be keyed by the **canonical project ID**.
 * When you only have a route slug (or unknown id-or-slug), resolve with `check` first.
 */
export const projectQueryOptions = {
	/** Resolve a slug or ID to the canonical project ID. */
	check: (idOrSlug: string, client: AbstractModrinthClient) => ({
		queryKey: ['project', 'check', idOrSlug] as const,
		queryFn: () => client.labrinth.projects_v2.check(idOrSlug),
		staleTime: STALE_TIME,
	}),

	/** @param projectId Canonical project ID (not slug) */
	v2: (projectId: string, client: AbstractModrinthClient) => ({
		queryKey: ['project', 'v2', projectId] as const,
		queryFn: () => client.labrinth.projects_v2.get(projectId),
		staleTime: STALE_TIME,
	}),

	/** @param projectId Canonical project ID (not slug) */
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
