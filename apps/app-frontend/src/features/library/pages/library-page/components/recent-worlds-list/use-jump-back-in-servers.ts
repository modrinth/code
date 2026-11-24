import { autoToHTML } from '@sfirew/minecraft-motd-parser'
import { useQueries, useQueryClient } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'

import { serverStatusQueryOptions } from '@/features/servers/queries'
import type { ServerData } from '@/features/servers/server-pings'
import { instanceProtocolQueryOptions } from '@/features/worlds/queries'
import type { GameInstance } from '@/helpers/types'
import type {
	ProtocolVersion,
	ServerStatus,
	WorldWithInstance,
} from '@/platform/app-lib/worlds/types'

type ServerWorldItem = {
	type: string
	instance: GameInstance
	world?: WorldWithInstance
}

export function useJumpBackInServers(items: MaybeRefOrGetter<ServerWorldItem[]>) {
	const queryClient = useQueryClient()

	const serverWorlds = computed(() =>
		toValue(items).flatMap((item) =>
			item.type === 'world' && item.world?.type === 'server'
				? [
						{
							instance: item.instance,
							address: item.world.address,
							managed: item.world.source === 'linked_server_project',
						},
					]
				: [],
		),
	)
	const serverInstances = computed(() => [
		...new Map(serverWorlds.value.map(({ instance }) => [instance.id, instance])).values(),
	])
	const protocolQueries = useQueries({
		queries: computed(() =>
			serverInstances.value.map((instance) => ({
				...instanceProtocolQueryOptions(instance),
				select: (protocol: ProtocolVersion | null) => ({ instanceId: instance.id, protocol }),
			})),
		),
	})
	const protocolVersions = computed(() => {
		const versions: Record<string, ProtocolVersion | null | undefined> = {}
		for (const query of protocolQueries.value) {
			if (query.data) versions[query.data.instanceId] = query.data.protocol
		}
		return versions
	})
	const serversToPing = computed(() => {
		const servers = new Map<
			string,
			{ address: string; protocol: ProtocolVersion | null; managed: boolean }
		>()
		for (const { instance, address, managed } of serverWorlds.value) {
			const protocol = protocolVersions.value[instance.id]
			const index = serverInstances.value.findIndex((server) => server.id === instance.id)
			if (!managed && protocol === undefined && !protocolQueries.value[index]?.isError) continue
			if (!servers.has(address)) {
				servers.set(address, { address, protocol: managed ? null : (protocol ?? null), managed })
			}
		}
		return [...servers.values()]
	})
	const serverQueries = useQueries({
		queries: computed(() =>
			serversToPing.value.map(({ address, protocol, managed }) => ({
				...serverStatusQueryOptions(address, protocol),
				refetchInterval: managed ? 30_000 : false,
				select: (status: ServerStatus) => ({
					address,
					status,
					renderedMotd: status.description ? autoToHTML(status.description) : undefined,
				}),
			})),
		),
	})
	const serverData = computed(() => {
		const data: Record<string, ServerData> = {}
		for (const [index, query] of serverQueries.value.entries()) {
			const address = serversToPing.value[index]?.address
			if (!address) continue
			data[address] = {
				refreshing: query.isPending && !query.isError,
				status: query.isError ? undefined : query.data?.status,
				renderedMotd: query.isError ? undefined : query.data?.renderedMotd,
			}
		}
		return data
	})

	function refreshServer(address: string) {
		const server = serversToPing.value.find((candidate) => candidate.address === address)
		if (!server) return
		void queryClient.refetchQueries(
			{ queryKey: serverStatusQueryOptions(address, server.protocol).queryKey, exact: true },
			{ cancelRefetch: false },
		)
	}

	return { protocolVersions, serverData, refreshServer }
}
