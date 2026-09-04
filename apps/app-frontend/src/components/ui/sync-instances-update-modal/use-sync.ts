import { injectNotificationManager } from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

import { isSyncedOptionAvailable, set_global_synced_option } from '@/helpers/instance'
import {
	gameOptionsSyncSourcesQueryOptions,
	globalSyncedOptionsQueryOptions,
	syncedOptionsKeys,
} from '@/helpers/synced-options'
import { syncedPackKeys } from '@/helpers/synced-packs'
import { instanceKeys, instanceListQueryOptions } from '@/pages/instance/query-options'

export const syncUpdateOptions = (
	[
		'game_options',
		'multiplayer_servers',
		'command_history',
		'creative_hotbars',
		'resource_packs',
		'data_packs',
	] as const
).filter(isSyncedOptionAvailable)

export type SyncUpdateOption = (typeof syncUpdateOptions)[number]

export function useSyncInstancesUpdate() {
	const queryClient = useQueryClient()
	const { handleError } = injectNotificationManager()
	const isOpen = ref(false)
	const sourceOptions = ref<SyncUpdateOption[]>([])
	const sourceInstanceId = ref('')
	const needsGameOptionsSource = computed(() => sourceOptions.value.includes('game_options'))
	const globalOptionsQuery = useQuery({
		...globalSyncedOptionsQueryOptions(),
		enabled: isOpen,
	})
	const gameSourcesQuery = useQuery({
		...gameOptionsSyncSourcesQueryOptions(),
		enabled: needsGameOptionsSource,
	})
	const instancesQuery = useQuery({
		...instanceListQueryOptions(),
		staleTime: 0,
		enabled: computed(() => sourceOptions.value.length > 0 && !needsGameOptionsSource.value),
	})
	const sources = computed(() =>
		needsGameOptionsSource.value
			? (gameSourcesQuery.data.value ?? []).map((source) => ({
					id: source.source_id,
					name: source.name,
					icon_path: source.icon_path,
					eligible: source.eligible,
				}))
			: (instancesQuery.data.value ?? []).map((instance) => ({
					id: instance.id,
					name: instance.name,
					icon_path: instance.icon_path,
					eligible: instance.install_stage === 'installed' && !instance.quarantined,
				})),
	)
	const sourcesLoading = computed(() =>
		needsGameOptionsSource.value
			? gameSourcesQuery.isPending.value || gameSourcesQuery.isFetching.value
			: instancesQuery.isPending.value || instancesQuery.isFetching.value,
	)
	const sourcesError = computed(() =>
		needsGameOptionsSource.value ? gameSourcesQuery.isError.value : instancesQuery.isError.value,
	)
	const allSynced = computed(() =>
		syncUpdateOptions.every((option) => globalOptionsQuery.data.value?.[option]),
	)

	watch([sources, sourceOptions], ([candidates]) => {
		if (!candidates.some((source) => source.id === sourceInstanceId.value && source.eligible)) {
			sourceInstanceId.value = candidates.find((source) => source.eligible)?.id ?? ''
		}
	})

	const syncMutation = useMutation({
		mutationKey: syncedOptionsKeys.set,
		mutationFn: async ({
			options,
			enabled,
			baseInstanceId,
		}: {
			options: readonly SyncUpdateOption[]
			enabled: boolean
			baseInstanceId?: string
		}) => {
			for (const option of options.filter(isSyncedOptionAvailable)) {
				const updated = await set_global_synced_option(option, enabled, baseInstanceId)
				queryClient.setQueryData(syncedOptionsKeys.global, updated)
			}
		},
		onMutate: () => queryClient.cancelQueries({ queryKey: syncedOptionsKeys.global }),
		onError: handleError,
		onSettled: () =>
			Promise.all([
				queryClient.invalidateQueries({ queryKey: syncedOptionsKeys.global }),
				queryClient.invalidateQueries({ queryKey: syncedOptionsKeys.initialized }),
				queryClient.invalidateQueries({ queryKey: syncedOptionsKeys.gameSources }),
				queryClient.invalidateQueries({ queryKey: ['instance-synced-options'] }),
				queryClient.invalidateQueries({ queryKey: instanceKeys.all }),
				queryClient.invalidateQueries({ queryKey: ['worlds'] }),
				queryClient.invalidateQueries({ queryKey: syncedPackKeys.all }),
			]),
	})

	function chooseSource(options: readonly SyncUpdateOption[]) {
		sourceInstanceId.value = ''
		sourceOptions.value = options.filter(isSyncedOptionAvailable)
	}

	function retrySources() {
		return needsGameOptionsSource.value
			? gameSourcesQuery.refetch({ cancelRefetch: false })
			: instancesQuery.refetch({ cancelRefetch: false })
	}

	return {
		isOpen,
		globalOptionsQuery,
		allSynced,
		syncMutation,
		sourceOptions,
		sourceInstanceId,
		sources,
		sourcesLoading,
		sourcesError,
		chooseSource,
		retrySources,
	}
}
