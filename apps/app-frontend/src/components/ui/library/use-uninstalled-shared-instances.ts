import { injectAuth, injectModrinthClient } from '@modrinth/ui'
import { useQueries, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type Ref, watch } from 'vue'

import {
	cache_shared_instance_library,
	type CachedSharedInstanceSummary,
	get_cached_shared_instance_library,
} from '@/helpers/instance'
import type { GameInstance, InstanceLoader } from '@/helpers/types'

export type UninstalledSharedInstance = {
	type: 'uninstalled_shared_instance'
	id: string
	shared_instance_id: string
	role: 'owner' | 'member'
	manager_id: string | null
	name: string
	icon_path?: string
	game_version: string
	loader: InstanceLoader
}

export const sharedInstanceLibraryKeys = {
	all: ['shared-instances', 'library'] as const,
	cached: (userId: string | null) => [...sharedInstanceLibraryKeys.all, 'cached', userId] as const,
	ids: (userId: string | null) => [...sharedInstanceLibraryKeys.all, 'ids', userId] as const,
	detail: (userId: string | null, sharedInstanceId: string) =>
		[...sharedInstanceLibraryKeys.all, 'detail', userId, sharedInstanceId] as const,
}

const fromCachedSummary = (summary: CachedSharedInstanceSummary): UninstalledSharedInstance => ({
	type: 'uninstalled_shared_instance',
	id: `shared-instance:${summary.shared_instance_id}`,
	shared_instance_id: summary.shared_instance_id,
	role: summary.role,
	manager_id: summary.manager_id,
	name: summary.name,
	icon_path: summary.icon_url ?? undefined,
	game_version: summary.game_version,
	loader: summary.loader,
})

const toCachedSummary = (instance: UninstalledSharedInstance): CachedSharedInstanceSummary => ({
	shared_instance_id: instance.shared_instance_id,
	role: instance.role,
	manager_id: instance.manager_id,
	name: instance.name,
	icon_url: instance.icon_path ?? null,
	game_version: instance.game_version,
	loader: instance.loader,
})

export function useUninstalledSharedInstances(instances: Ref<GameInstance[]>) {
	const auth = injectAuth()
	const client = injectModrinthClient()
	const queryClient = useQueryClient()
	const userId = computed(() => auth.user.value?.id ?? null)

	const cachedQuery = useQuery({
		queryKey: computed(() => sharedInstanceLibraryKeys.cached(userId.value)),
		queryFn: async () => (await get_cached_shared_instance_library()).map(fromCachedSummary),
		enabled: computed(() => !!userId.value),
		staleTime: Infinity,
	})

	const idsQuery = useQuery({
		queryKey: computed(() => sharedInstanceLibraryKeys.ids(userId.value)),
		queryFn: () =>
			client.sharedinstances.instances_v1.getForUser(userId.value!, { include_joined: true }),
		enabled: computed(() => !!userId.value),
		staleTime: 60_000,
	})

	const installedSharedInstanceIds = computed(
		() =>
			new Set(
				instances.value.flatMap((instance) =>
					instance.shared_instance?.id ? [instance.shared_instance.id] : [],
				),
			),
	)

	const uninstalledIds = computed(() =>
		[...new Set(idsQuery.data.value ?? [])].filter(
			(sharedInstanceId) => !installedSharedInstanceIds.value.has(sharedInstanceId),
		),
	)

	const loadSharedInstance = async (
		currentUserId: string,
		sharedInstanceId: string,
	): Promise<UninstalledSharedInstance | null> => {
		const [instance, version, users] = await Promise.all([
			client.sharedinstances.instances_v1.get(sharedInstanceId),
			client.sharedinstances.instances_v1.getLatestVersion(sharedInstanceId),
			client.sharedinstances.instances_v1.getUsers(sharedInstanceId),
		])
		if (instance.quarantine || !version.ready) return null

		const ownerId = users.users.find((user) => user.join_type === 'owner')?.id ?? null
		const isOwner = ownerId === currentUserId

		return {
			type: 'uninstalled_shared_instance',
			id: `shared-instance:${sharedInstanceId}`,
			shared_instance_id: sharedInstanceId,
			role: isOwner ? 'owner' : 'member',
			manager_id: isOwner ? null : ownerId,
			name: instance.name,
			icon_path: instance.icon ?? undefined,
			game_version: version.game_version,
			loader: version.loader as InstanceLoader,
		}
	}

	const detailQueries = useQueries({
		queries: computed(() =>
			uninstalledIds.value.map((sharedInstanceId) => ({
				queryKey: sharedInstanceLibraryKeys.detail(userId.value, sharedInstanceId),
				queryFn: () => loadSharedInstance(userId.value!, sharedInstanceId),
				select: (instance: UninstalledSharedInstance | null) => ({ sharedInstanceId, instance }),
				staleTime: 60_000,
			})),
		),
	})

	const sharedInstances = computed<UninstalledSharedInstance[]>(() => {
		if (!userId.value) return []

		const cached = cachedQuery.data.value ?? []
		if (!idsQuery.isSuccess.value) {
			return cached.filter(
				(instance) => !installedSharedInstanceIds.value.has(instance.shared_instance_id),
			)
		}

		const loaded = new Map(
			detailQueries.value.flatMap((query) =>
				query.data ? [[query.data.sharedInstanceId, query.data.instance] as const] : [],
			),
		)
		const cachedById = new Map(cached.map((instance) => [instance.shared_instance_id, instance]))
		return uninstalledIds.value.flatMap((sharedInstanceId) => {
			const instance = loaded.has(sharedInstanceId)
				? loaded.get(sharedInstanceId)
				: cachedById.get(sharedInstanceId)
			return instance ? [instance] : []
		})
	})

	const settledSummaries = computed(() => {
		if (!idsQuery.isSuccess.value || detailQueries.value.some((query) => query.isPending)) {
			return null
		}
		return JSON.stringify(sharedInstances.value.map(toCachedSummary))
	})

	watch(settledSummaries, (summaries) => {
		const currentUserId = userId.value
		if (!summaries || !currentUserId) return

		queryClient.setQueryData(sharedInstanceLibraryKeys.cached(currentUserId), sharedInstances.value)
		void cache_shared_instance_library(currentUserId, JSON.parse(summaries)).catch((error) =>
			console.warn('Failed to cache shared instance library:', error),
		)
	})

	return sharedInstances
}
