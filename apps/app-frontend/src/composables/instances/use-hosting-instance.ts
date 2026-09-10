import type { Archon } from '@modrinth/api-client'
import { getHostingServerAddress, injectAuth, injectModrinthClient } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { useStorage } from '@vueuse/core'
import { computed, type Ref, watch } from 'vue'

import type { GameInstance } from '@/helpers/types'
import { get_instance_worlds } from '@/helpers/worlds'

type HostingInstanceMetadata = {
	sharedInstanceId: string
	serverId: string
	worldId: string
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
		region: location.status === 'assigned' && location.location_metadata.region_should_be_user_displayed
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
	const isHostingInstance = computed(() => !!instance.value?.shared_instance?.server_manager_name)
	const saved = computed(() => {
		const current = instance.value
		const metadata = current && cache.value[current.id]
		return metadata?.sharedInstanceId === current?.shared_instance?.id ? metadata : undefined
	})
	const serverQuery = useQuery({
		queryKey: computed(() => ['instances', instance.value?.id, 'hosting', auth.user.value?.id]),
		enabled: computed(() => isHostingInstance.value && !offline.value && !!auth.user.value?.id && auth.user.value.id === instance.value?.shared_instance?.linked_user_id),
		queryFn: async () => {
			const current = instance.value!
			const sharedId = current.shared_instance!.id
			const known = saved.value
			const servers = known
				? [await client.archon.servers_v1.get(known.serverId)]
				: await client.archon.servers_v1.list()
			const server = servers.find((server) => server.worlds.some((world) => world.content?.shared_instance_id === sharedId))
			const world = server?.worlds.find((world) => world.content?.shared_instance_id === sharedId)
			if (!server || !world) return null
			const legacy = await client.archon.servers_v0.get(server.id)
			return { instanceId: current.id, metadata: hostingInstanceMetadata(server, world.id, sharedId, getHostingServerAddress(legacy.net, server.subdomain)) }
		},
		staleTime: 60_000,
		retry: false,
	})
	watch(serverQuery.data, (result) => {
		if (result) cache.value[result.instanceId] = result.metadata
	})
	const worldsQuery = useQuery({
		queryKey: computed(() => ['instances', instance.value?.id, 'hosting-worlds']),
		enabled: computed(() => isHostingInstance.value && !saved.value && instance.value?.install_stage === 'installed'),
		queryFn: () => get_instance_worlds(instance.value!.id),
	})
	const address = computed(() => {
		if (saved.value?.address) return saved.value.address
		const world = worldsQuery.data.value?.find((world) =>
			world.type === 'server' && world.name === instance.value?.shared_instance?.server_manager_name,
		)
		return world?.type === 'server' ? world.address : undefined
	})
	return {
		isHostingInstance,
		region: computed(() => saved.value?.region),
		address,
	}
}
