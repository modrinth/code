import type { AbstractModrinthClient, Labrinth } from '@modrinth/api-client'
import { serverKeys } from '@modrinth/ui'
import { queryOptions, type UseQueryReturnType } from '@tanstack/vue-query'

import {
	get_installed_project_ids as getInstalledProjectIds,
	list as listInstances,
} from '@/helpers/instance'
import { get_categories, get_game_versions, get_loaders } from '@/helpers/tags'
import { get_instance_worlds } from '@/helpers/worlds'
import { instanceKeys } from '@/pages/instance/query-options'

export function browseTagsQueryOptions() {
	return queryOptions({
		queryKey: ['browse', 'tags'] as const,
		networkMode: 'always',
		queryFn: async () => {
			const [categories, loaders, gameVersions] = await Promise.all([
				get_categories() as Promise<Labrinth.Tags.v2.Category[]>,
				get_loaders() as Promise<Labrinth.Tags.v2.Loader[]>,
				get_game_versions() as Promise<Labrinth.Tags.v2.GameVersion[]>,
			])
			return { categories, loaders, gameVersions }
		},
		staleTime: Infinity,
	})
}

export function installedProjectIdsQueryOptions(instanceId: string, fromWorlds: boolean) {
	return queryOptions({
		queryKey: instanceId
			? instanceKeys.installedProjectIds(instanceId, fromWorlds ? 'worlds' : 'content')
			: ([...instanceKeys.all, 'installed-project-ids'] as const),
		networkMode: 'always',
		queryFn: async (): Promise<string[]> => {
			if (!instanceId) {
				return (await listInstances())
					.map((gameInstance) => gameInstance.link?.project_id)
					.filter((projectId): projectId is string => !!projectId)
			}
			if (fromWorlds) {
				return (await get_instance_worlds(instanceId))
					.filter((world) => world.type === 'server' && 'project_id' in world && world.project_id)
					.map((world) => (world as { project_id: string }).project_id)
			}
			return await getInstalledProjectIds(instanceId)
		},
		staleTime: 0,
	})
}

export function browseServerContentQueryOptions(
	client: AbstractModrinthClient,
	serverId: string,
	worldId: string,
) {
	return queryOptions({
		queryKey: [...serverKeys.content(serverId), 'installed', worldId] as const,
		queryFn: () => client.archon.content_v1.getAddons(serverId, worldId),
	})
}

export async function settleQuery<TData, TError>(query: UseQueryReturnType<TData, TError>) {
	if (query.data.value !== undefined) return
	await query.suspense()
}
