import { autoToHTML } from '@sfirew/minecraft-motd-parser'
import { useQueries, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'

import { normalizeServerAddress } from '@/features/servers/address'
import { serverStatusQueryOptions } from '@/features/servers/queries'
import type { ServerData } from '@/features/servers/server-pings'
import { instanceProtocolQueryOptions } from '@/features/worlds/queries'
import { injectInstancePage } from '@/pages/instance/instance-context'
import type { ServerStatus, ServerWorld } from '@/platform/app-lib/worlds/types'

import type { ServerAvailability } from '../types'

export function useServerStatuses(servers: MaybeRefOrGetter<ServerWorld[]>) {
	const instancePage = injectInstancePage()
	const queryClient = useQueryClient()

	const protocolQuery = useQuery(
		computed(() => instanceProtocolQueryOptions(instancePage.instance.value)),
	)
	const protocolVersion = computed(() => protocolQuery.data.value ?? null)
	const protocolSettled = computed(
		() => protocolQuery.isSuccess.value || protocolQuery.isError.value,
	)

	const headerServerAddress = computed(() => {
		const address = instancePage.serverAddress.value
		return instancePage.isServerInstance.value && address ? normalizeServerAddress(address) : null
	})

	function isHeaderServer(address: string) {
		return (
			!!headerServerAddress.value && normalizeServerAddress(address) === headerServerAddress.value
		)
	}

	function statusOptions(address: string) {
		return serverStatusQueryOptions(address, isHeaderServer(address) ? null : protocolVersion.value)
	}

	const addresses = computed(() => [...new Set(toValue(servers).map((server) => server.address))])

	const statusQueries = useQueries({
		queries: computed(() =>
			addresses.value.map((address) => ({
				...statusOptions(address),
				enabled: protocolSettled.value || isHeaderServer(address),
				select: (status: ServerStatus) => ({
					status,
					renderedMotd: status.description ? autoToHTML(status.description) : undefined,
				}),
			})),
		),
	})

	const serverData = computed(() => {
		const data: Record<string, ServerData & { availability: ServerAvailability }> = {}
		for (const [index, query] of statusQueries.value.entries()) {
			const address = addresses.value[index]
			if (!address) continue
			const status = query.isError ? undefined : query.data?.status
			data[address] = {
				refreshing: query.isPending || query.isFetching,
				status,
				renderedMotd: query.isError ? undefined : query.data?.renderedMotd,
				availability: status ? 'online' : query.isError ? 'offline' : 'pending',
			}
		}
		return data
	})

	function getServerData(world: ServerWorld) {
		return serverData.value[world.address]
	}

	function getServerAvailability(world: ServerWorld): ServerAvailability {
		return getServerData(world)?.availability ?? 'pending'
	}

	function refreshServer(address: string) {
		return queryClient.refetchQueries(
			{ queryKey: statusOptions(address).queryKey, exact: true },
			{ cancelRefetch: false },
		)
	}

	function refreshAllServers() {
		return Promise.all(addresses.value.map(refreshServer))
	}

	return {
		protocolVersion,
		getServerData,
		getServerAvailability,
		refreshServer,
		refreshAllServers,
	}
}
