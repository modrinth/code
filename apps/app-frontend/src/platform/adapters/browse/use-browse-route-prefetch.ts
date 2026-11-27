import type { AbstractModrinthClient } from '@modrinth/api-client'
import { serverDetailQueryOptions, serverDetailV1QueryOptions } from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { onScopeDispose } from 'vue'
import { type RouteLocationNormalized, useRouter } from 'vue-router'

import {
	instanceDetailQueryOptions,
	instanceLinkedProjectQueryOptions,
} from '@/pages/instance/query-options'

import {
	browseServerContentQueryOptions,
	browseTagsQueryOptions,
	installedProjectIdsQueryOptions,
} from './query-options'
import { readQueryString } from './server-install-content'

export function useBrowseRoutePrefetch(client: AbstractModrinthClient) {
	const router = useRouter()
	const queryClient = useQueryClient()

	const ensure = queryClient.ensureQueryData.bind(queryClient)

	function quietly<T>(promise: Promise<T>) {
		return promise.catch(() => undefined)
	}

	async function prefetchInstance(instanceId: string, fromWorlds: boolean) {
		const [instance] = await Promise.all([
			instanceId ? quietly(ensure(instanceDetailQueryOptions(instanceId))) : undefined,
			quietly(ensure(installedProjectIdsQueryOptions(instanceId, fromWorlds))),
		])
		const linkedProjectId = instance?.link?.project_id
		if (linkedProjectId) {
			await quietly(ensure(instanceLinkedProjectQueryOptions(linkedProjectId)))
		}
	}

	async function prefetchServer(serverId: string, worldIdFromRoute: string | null) {
		const [, server] = await Promise.all([
			quietly(ensure(serverDetailQueryOptions(client, serverId))),
			worldIdFromRoute ? undefined : quietly(ensure(serverDetailV1QueryOptions(client, serverId))),
		])
		const worlds = server?.worlds ?? []
		const worldId =
			worldIdFromRoute ?? (worlds.find((world) => world.is_active) ?? worlds[0])?.id ?? null
		if (worldId) await quietly(ensure(browseServerContentQueryOptions(client, serverId, worldId)))
	}

	async function prefetch(to: RouteLocationNormalized) {
		if (!to.path.startsWith('/browse/')) return
		const serverId = readQueryString(to.query.sid)
		await Promise.all([
			quietly(ensure(browseTagsQueryOptions())),
			prefetchInstance(
				readQueryString(to.query.i) ?? '',
				readQueryString(to.query.from) === 'worlds',
			),
			serverId ? prefetchServer(serverId, readQueryString(to.query.wid)) : undefined,
		])
	}

	let pendingPrefetch: Promise<void> | undefined
	const cleanups = [
		router.beforeEach((to) => {
			pendingPrefetch = prefetch(to)
		}),
		router.beforeResolve(() => pendingPrefetch),
	]

	onScopeDispose(() => cleanups.forEach((cleanup) => cleanup()))
}
