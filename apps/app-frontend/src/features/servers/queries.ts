import { type QueryClient, queryOptions } from '@tanstack/vue-query'

import { getServerStatus } from '@/platform/app-lib/worlds/commands'
import type { ProtocolVersion, ServerStatus } from '@/platform/app-lib/worlds/types'

import { normalizeServerAddress } from './address'

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

export function serverStatusQueryOptions(
	address: string,
	protocolVersion: ProtocolVersion | null = null,
) {
	return queryOptions({
		queryKey: getServerStatusQueryKey(address, protocolVersion),
		queryFn: async () => {
			try {
				return await getServerStatus(address, protocolVersion)
			} catch (error) {
				if (protocolVersion?.legacy) throw error
				return await getServerStatus(address, { version: 74, legacy: true })
			}
		},
		staleTime: 30_000,
		gcTime: SERVER_STATUS_CACHE_MS,
		refetchOnWindowFocus: false,
		retry: false,
	})
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
		queryFn: () => getServerStatus(address, protocolVersion),
		staleTime: SERVER_STATUS_CACHE_MS,
		gcTime: SERVER_STATUS_CACHE_MS,
	})
}
