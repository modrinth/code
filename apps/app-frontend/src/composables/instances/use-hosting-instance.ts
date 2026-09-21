import type { Archon } from '@modrinth/api-client'
import { injectAuth, injectModrinthClient } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { useStorage } from '@vueuse/core'
import { computed, type Ref, watch } from 'vue'

import type { GameInstance } from '@/helpers/types'
import { get_instance_worlds } from '@/helpers/worlds'

type HostingInstanceMetadata = {
	sharedInstanceId: string
	serverId?: string
	worldId?: string
	address: string
	region?: string
}

export function hostingInstanceMetadata(
	server: Archon.Servers.v1.ServerFull,
	worldId: string,
	sharedInstanceId: string,
	address: string,
): HostingInstanceMetadata {
	const location = server.location
	return {
		sharedInstanceId,
		serverId: server.id,
		worldId,
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
	const serverQuery = useQuery({
		queryKey: computed(() => ['instances', instance.value?.id, 'hosting', auth.user.value?.id]),
		enabled: canQuery,
		queryFn: async () => {
			const current = instance.value!
			const sharedId = current.shared_instance!.id
			const known = saved.value
			const shared = await client.sharedinstances.instances_v1.get(sharedId, {
				query_linked_server: false,
			})
			if (!shared.linked_server) return null
			return {
				instanceId: current.id,
				metadata: {
					...known,
					sharedInstanceId: sharedId,
					address: shared.linked_server.domain,
					region: shared.linked_server.region,
				},
			}
		},
		staleTime: 60_000,
		retry: false,
	})
	watch(serverQuery.data, (result) => {
		if (result) cache.value[result.instanceId] = result.metadata
	})
	const isHostingInstance = computed(
		() => !!saved.value || !!instance.value?.shared_instance?.server_manager_name,
	)
	const statusQuery = useQuery({
		queryKey: computed(() => [
			'instances',
			instance.value?.id,
			'hosting-status',
			instance.value?.shared_instance?.id,
			auth.user.value?.id,
		]),
		enabled: computed(() => canQuery.value && isHostingInstance.value),
		queryFn: () =>
			client.sharedinstances.instances_v1.get(instance.value!.shared_instance!.id, {
				query_linked_server: true,
			}),
		staleTime: 30_000,
		refetchInterval: 30_000,
		retry: false,
	})
	async function refreshOnlineStatus() {
		if (!canQuery.value || !isHostingInstance.value) return null
		const result = await statusQuery.refetch({ throwOnError: false })
		return result.isError ? null : (result.data?.linked_server?.online_status ?? null)
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
			!canQuery.value || statusQuery.isError.value
				? null
				: (statusQuery.data.value?.linked_server?.online_status ?? null),
		),
		refreshOnlineStatus,
		region: computed(() => saved.value?.region),
		address,
	}
}
