import type { QueryClient } from '@tanstack/vue-query'

import {
	get_server_status,
	normalizeServerAddress,
	type ProtocolVersion,
	type ServerStatus,
} from '@/helpers/worlds'

export const SERVER_STATUS_CACHE_MS = 10 * 60 * 1000

function getProtocolVersionKey(protocolVersion: ProtocolVersion | null) {
	if (!protocolVersion) return 'default'
	return `${protocolVersion.version}:${protocolVersion.legacy ? 'legacy' : 'modern'}`
}

export function getServerStatusQueryKey(
	address: string,
	protocolVersion: ProtocolVersion | null = null,
) {
	return [
		'minecraft-server-status',
		normalizeServerAddress(address) || address.trim().toLowerCase(),
		getProtocolVersionKey(protocolVersion),
	] as const
}

export function getFreshCachedServerStatus(
	queryClient: QueryClient,
	address: string,
	protocolVersion: ProtocolVersion | null = null,
) {
	const queryKey = getServerStatusQueryKey(address, protocolVersion)
	const updatedAt = queryClient.getQueryState(queryKey)?.dataUpdatedAt ?? 0
	if (!updatedAt || Date.now() - updatedAt >= SERVER_STATUS_CACHE_MS) return undefined
	return queryClient.getQueryData<ServerStatus>(queryKey)
}

export async function fetchCachedServerStatus(
	queryClient: QueryClient,
	address: string,
	protocolVersion: ProtocolVersion | null = null,
) {
	return await queryClient.fetchQuery({
		queryKey: getServerStatusQueryKey(address, protocolVersion),
		queryFn: () => get_server_status(address, protocolVersion),
		staleTime: SERVER_STATUS_CACHE_MS,
		gcTime: SERVER_STATUS_CACHE_MS,
	})
}
