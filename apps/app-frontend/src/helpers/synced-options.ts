import { queryOptions } from '@tanstack/vue-query'

import { list_game_options_sync_sources } from './game-options'
import {
	get_command_history,
	get_global_synced_options,
	get_initialized_synced_options,
	list_synced_servers,
} from './instance'

export const syncedOptionsKeys = {
	global: ['global-synced-options'] as const,
	initialized: ['initialized-synced-options'] as const,
	gameSources: ['game-options-sync-sources'] as const,
	set: ['global-synced-options', 'set'] as const,
	servers: ['synced-servers', 'list'] as const,
	commandHistory: ['synced-options', 'command-history'] as const,
}

export function globalSyncedOptionsQueryOptions() {
	return queryOptions({
		queryKey: syncedOptionsKeys.global,
		queryFn: get_global_synced_options,
	})
}

export function initializedSyncedOptionsQueryOptions() {
	return queryOptions({
		queryKey: syncedOptionsKeys.initialized,
		queryFn: get_initialized_synced_options,
	})
}

export function gameOptionsSyncSourcesQueryOptions() {
	return queryOptions({
		queryKey: syncedOptionsKeys.gameSources,
		queryFn: list_game_options_sync_sources,
		staleTime: 0,
	})
}

export function syncedServersQueryOptions() {
	return queryOptions({
		queryKey: syncedOptionsKeys.servers,
		queryFn: list_synced_servers,
	})
}

export function commandHistoryQueryOptions() {
	return queryOptions({
		queryKey: syncedOptionsKeys.commandHistory,
		queryFn: get_command_history,
	})
}
