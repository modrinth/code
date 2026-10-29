import type { Archon } from '@modrinth/api-client'
import { injectAuth, injectModrinthClient } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { useStorage } from '@vueuse/core'
import { computed, type Ref, watch } from 'vue'

import type { GameInstance } from '@/helpers/types'
import { ensureManagedServerWorldExists } from '@/helpers/worlds'

import { linkedServerQueryOptions, linkedServerWorldsQueryOptions } from './queries'

type LinkedServerMetadata = {
	sharedInstanceId: string
	address: string
	region?: string
}

export function linkedServerMetadata(
	server: Archon.Servers.v1.ServerFull,
	sharedInstanceId: string,
	address: string,
): LinkedServerMetadata {
	const location = server.location
	return {
		sharedInstanceId,
		address,
		region:
			location.status === 'assigned' && location.location_metadata.region_should_be_user_displayed
				? location.location_metadata.region
				: undefined,
	}
}

/** Last known hosting metadata per instance, used while offline or before the linked server loads. */
export function useLinkedServerCache() {
	return useStorage<Record<string, LinkedServerMetadata>>('hosting-instance-metadata', {})
}

export function useLinkedServer(instance: Ref<GameInstance | undefined>, offline: Ref<boolean>) {
	const client = injectModrinthClient()
	const auth = injectAuth()
	const cache = useLinkedServerCache()
	const saved = computed(() => {
		const current = instance.value
		const metadata = current && cache.value[current.id]
		return metadata?.sharedInstanceId === current?.shared_instance?.id ? metadata : undefined
	})
	const canQuery = computed(
		() =>
			!!instance.value?.shared_instance &&
			!offline.value &&
			!!auth.user.value?.id &&
			auth.user.value.id === instance.value.shared_instance.linked_user_id,
	)
	const hostingQuery = useQuery(
		computed(() => ({
			...linkedServerQueryOptions(
				client,
				instance.value?.id ?? '',
				instance.value?.shared_instance?.id ?? '',
				auth.user.value?.id,
			),
			enabled: canQuery.value,
		})),
	)
	const liveServer = computed(() => {
		const result = hostingQuery.data.value
		if (!result) return undefined
		return result?.instanceId === instance.value?.id &&
			result?.sharedId === instance.value?.shared_instance?.id &&
			result?.userId === auth.user.value?.id
			? result.linkedServer
			: undefined
	})
	watch(liveServer, (server) => {
		const current = instance.value
		if (!current || server === undefined) return
		if (!server) {
			cache.value = Object.fromEntries(
				Object.entries(cache.value).filter(([id]) => id !== current.id),
			)
			return
		}
		const previousAddress = saved.value?.address
		cache.value[current.id] = {
			...saved.value,
			sharedInstanceId: current.shared_instance!.id,
			address: server.domain,
			region: server.region,
		}
		if (previousAddress !== server.domain && current.install_stage === 'installed') {
			void ensureManagedServerWorldExists(
				current.id,
				current.shared_instance?.server_manager_name ?? null,
				server.domain,
			)
		}
	})
	const isHostingInstance = computed(() =>
		liveServer.value !== undefined
			? !!liveServer.value
			: !!saved.value || !!instance.value?.shared_instance?.server_manager_name,
	)
	async function refreshOnlineStatus() {
		if (!canQuery.value || !isHostingInstance.value) return null
		const result = await hostingQuery.refetch({ throwOnError: false })
		return result.isError ? null : (result.data?.linkedServer?.online_status ?? null)
	}
	const worldsQuery = useQuery(
		computed(() => ({
			...linkedServerWorldsQueryOptions(instance.value?.id ?? ''),
			enabled:
				isHostingInstance.value && !saved.value && instance.value?.install_stage === 'installed',
		})),
	)
	const address = computed(() => {
		if (liveServer.value !== undefined) return liveServer.value?.domain
		if (saved.value?.address) return saved.value.address
		const world = worldsQuery.data.value?.find(
			(world) =>
				world.type === 'server' &&
				world.name === instance.value?.shared_instance?.server_manager_name,
		)
		return world?.type === 'server' ? world.address : undefined
	})
	return {
		isHostingInstance,
		onlineStatus: computed(() =>
			!canQuery.value || hostingQuery.isError.value
				? null
				: (liveServer.value?.online_status ?? null),
		),
		refreshOnlineStatus,
		region: computed(() =>
			liveServer.value !== undefined ? liveServer.value?.region : saved.value?.region,
		),
		address,
	}
}
