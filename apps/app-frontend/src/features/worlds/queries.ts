import type { GameVersion } from '@modrinth/ui'
import { queryOptions } from '@tanstack/vue-query'

import { traceStartupStep } from '@/helpers/startup-debug'
import { get_game_versions } from '@/helpers/tags'
import type { GameInstance } from '@/helpers/types'
import {
	getInstanceProtocolVersion,
	getInstanceWorlds,
	getRecentWorlds,
} from '@/platform/app-lib/worlds/commands'
import type { DisplayStatus } from '@/platform/app-lib/worlds/types'

import { sortWorlds } from './world-utils'

const HOME_DISPLAY_STATUSES: DisplayStatus[] = ['normal', 'favorite']

export const worldKeys = {
	all: ['worlds'] as const,
	instance: (instanceId: string) => [...worldKeys.all, instanceId] as const,
	recent: () => [...worldKeys.all, 'recent'] as const,
	protocol: (instanceId: string) => [...worldKeys.all, 'protocol', instanceId] as const,
}

export function instanceWorldsQueryOptions(instanceId: string) {
	return queryOptions({
		queryKey: worldKeys.instance(instanceId),
		networkMode: 'always',
		queryFn: async () => {
			const worlds = await getInstanceWorlds(instanceId)
			sortWorlds(worlds)
			return worlds
		},
		staleTime: 0,
	})
}

export function recentWorldsQueryOptions(limit: number) {
	return queryOptions({
		queryKey: [...worldKeys.recent(), limit, HOME_DISPLAY_STATUSES] as const,
		networkMode: 'always',
		queryFn: () =>
			traceStartupStep('Load recent worlds', () => getRecentWorlds(limit, HOME_DISPLAY_STATUSES)),
		staleTime: 30_000,
		refetchOnWindowFocus: false,
	})
}

export function instanceProtocolQueryOptions(instance: GameInstance) {
	const { id, game_version, loader, loader_version, install_stage, protocol_version } = instance
	return queryOptions({
		queryKey: [
			...worldKeys.protocol(id),
			{ game_version, loader, loader_version, install_stage, protocol_version },
		] as const,
		networkMode: 'always',
		queryFn: () =>
			traceStartupStep(`Load server protocol: ${id}`, () => getInstanceProtocolVersion(id)),
		staleTime: Infinity,
	})
}

export function gameVersionsQueryOptions() {
	return queryOptions({
		queryKey: ['tags', 'game-versions'] as const,
		networkMode: 'always',
		queryFn: (): Promise<GameVersion[]> =>
			traceStartupStep('Load game versions', () => get_game_versions()),
		staleTime: Infinity,
	})
}
