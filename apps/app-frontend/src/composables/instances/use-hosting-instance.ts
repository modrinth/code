import type { Archon } from '@modrinth/api-client'
import { injectAuth, injectModrinthClient } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { useStorage } from '@vueuse/core'
import { computed, type Ref, watch } from 'vue'

import type { GameInstance } from '@/helpers/types'
import { get_instance_worlds } from '@/helpers/worlds'

type HostingInstanceMetadata = {
	sharedInstanceId: string
	address: string
	region?: string
}

export function hostingInstanceMetadata(
	server: Archon.Servers.v1.ServerFull,
	sharedInstanceId: string,
	address: string,
): HostingInstanceMetadata {
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

export function useHostingInstanceCache() {
	return useStorage<Record<string, HostingInstanceMetadata>>('hosting-instance-metadata', {})
}

export function useHostingInstance(instance: Ref<GameInstance | undefined>, offline: Ref<boolean>) {
	const client = injectModrinthClient()
	const auth = injectAuth()
	const cache = useHostingInstanceCache()
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
	const hostingQuery = useQuery({
		queryKey: computed(() => [
			'instances',
			instance.value?.id,
			'hosting',
			instance.value?.shared_instance?.id,
			auth.user.value?.id,
		]),
		enabled: canQuery,
		queryFn: async () => {
			const current = instance.value!
			const sharedId = current.shared_instance!.id
			const userId = auth.user.value?.id
			const shared = await client.sharedinstances.instances_v1.get(sharedId, {
				query_linked_server: true,
			})
			return {
				instanceId: current.id,
				sharedId,
				userId,
				linkedServer: shared.linked_server,
			}
		},
		staleTime: 30_000,
		refetchInterval: 30_000,
		retry: false,
	})
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
			delete cache.value[current.id]
			return
		}
		cache.value[current.id] = {
			...saved.value,
			sharedInstanceId: current.shared_instance!.id,
			address: server.domain,
			region: server.region,
		}
	})
	const isHostingInstance = computed(
		() =>
			liveServer.value !== undefined
				? !!liveServer.value
				: !!saved.value || !!instance.value?.shared_instance?.server_manager_name,
	)
	async function refreshOnlineStatus() {
		if (!canQuery.value || !isHostingInstance.value) return null
		const result = await hostingQuery.refetch({ throwOnError: false })
		return result.isError ? null : (result.data?.linkedServer?.online_status ?? null)
	}
	const worldsQuery = useQuery({
		queryKey: computed(() => ['instances', instance.value?.id, 'hosting-worlds']),
		enabled: computed(
			() =>
				isHostingInstance.value && !saved.value && instance.value?.install_stage === 'installed',
		),
		queryFn: () => get_instance_worlds(instance.value!.id),
	})
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
