import { queryOptions } from '@tanstack/vue-query'

import { getProjectV3Many } from '@/platform/app-lib/cache/commands'

export const libraryKeys = {
	serverProjectIds: (projectIds: string[]) =>
		['library', 'server-project-ids', ...projectIds] as const,
}

export function libraryServerProjectIdsQueryOptions(projectIds: string[]) {
	return queryOptions({
		queryKey: libraryKeys.serverProjectIds(projectIds),
		networkMode: 'always',
		queryFn: async () => {
			const projects = await getProjectV3Many(projectIds)
			return projects.flatMap((project) => (project?.minecraft_server != null ? [project.id] : []))
		},
		staleTime: 5 * 60_000,
	})
}
