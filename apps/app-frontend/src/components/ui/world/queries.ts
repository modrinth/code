import { queryOptions } from '@tanstack/vue-query'

import { traceStartupStep } from '@/helpers/startup-debug'
import type { GameInstance } from '@/helpers/types'
import {
	get_instance_protocol_version,
	get_recent_worlds,
	type ProtocolVersion,
	refreshServerData,
	type ServerData,
} from '@/helpers/worlds'

export const recentWorldsKey = ['worlds', 'recent'] as const

export function recentWorldsQueryOptions(limit: number) {
	return queryOptions({
		queryKey: [...recentWorldsKey, limit, ['normal', 'favorite']] as const,
		queryFn: () =>
			traceStartupStep('Load recent worlds', () =>
				get_recent_worlds(limit, ['normal', 'favorite']),
			),
		staleTime: 30_000,
		refetchOnWindowFocus: false,
	})
}

export function instanceProtocolQueryOptions(instance: GameInstance) {
	const { id, game_version, loader, loader_version, install_stage, protocol_version } = instance
	return queryOptions({
		queryKey: [
			'worlds',
			'protocol',
			id,
			{ game_version, loader, loader_version, install_stage, protocol_version },
		] as const,
		queryFn: () =>
			traceStartupStep(`Load server protocol: ${id}`, () => get_instance_protocol_version(id)),
		staleTime: Infinity,
	})
}

export function serverStatusQueryOptions(address: string, protocol: ProtocolVersion | null) {
	return queryOptions({
		queryKey: ['worlds', 'server-status', address, protocol] as const,
		queryFn: async () => {
			const data: ServerData = { refreshing: true }
			await refreshServerData(data, protocol, address)
			return data
		},
		staleTime: 30_000,
		refetchOnWindowFocus: false,
	})
}
