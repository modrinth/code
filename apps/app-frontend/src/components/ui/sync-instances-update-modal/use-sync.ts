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

type SyncUpdateOptionState = Record<SyncUpdateOption, boolean>
type SyncUpdateSourceState = Partial<Record<SyncUpdateOption, string>>

function createOptionState(value = false): SyncUpdateOptionState {
	return Object.fromEntries(
		syncUpdateOptions.map((option) => [option, value]),
	) as SyncUpdateOptionState
}

export function useSyncInstancesUpdate() {
	const queryClient = useQueryClient()
	const { handleError } = injectNotificationManager()
	const isOpen = ref(false)
	const draftInitialized = ref(false)
	const initialOptions = ref(createOptionState())
	const draftOptions = ref(createOptionState())
	const draftSourceInstanceIds = ref<SyncUpdateSourceState>({})
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
	const allSynced = computed(
		() => draftInitialized.value && syncUpdateOptions.every((option) => draftOptions.value[option]),
	)

	function initializeDraft() {
		const globalOptions = globalOptionsQuery.data.value
		if (!globalOptions) return

		const options = createOptionState()
		for (const option of syncUpdateOptions) {
			options[option] = globalOptions[option]
		}
		initialOptions.value = { ...options }
		draftOptions.value = options
		draftSourceInstanceIds.value = {}
		draftInitialized.value = true
	}

	watch(
		() => globalOptionsQuery.data.value,
		() => {
			if (isOpen.value && !draftInitialized.value) initializeDraft()
		},
	)

	watch([sources, sourceOptions], ([candidates]) => {
		if (!candidates.some((source) => source.id === sourceInstanceId.value && source.eligible)) {
			sourceInstanceId.value = candidates.find((source) => source.eligible)?.id ?? ''
		}
	})

	const syncMutation = useMutation({
		mutationKey: syncedOptionsKeys.set,
		mutationFn: async (
			changes: {
				option: SyncUpdateOption
				enabled: boolean
				baseInstanceId?: string
			}[],
		) => {
			for (const { option, enabled, baseInstanceId } of changes) {
				const updated = await set_global_synced_option(option, enabled, baseInstanceId)
				queryClient.setQueryData(syncedOptionsKeys.global, updated)
				initialOptions.value[option] = enabled
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

	function beginDraft() {
		isOpen.value = true
		draftInitialized.value = false
		sourceOptions.value = []
		sourceInstanceId.value = ''
		initializeDraft()
	}

	function finishDraft() {
		isOpen.value = false
		draftInitialized.value = false
		draftSourceInstanceIds.value = {}
		sourceOptions.value = []
		sourceInstanceId.value = ''
	}

	function stageOptions(
		options: readonly SyncUpdateOption[],
		enabled: boolean,
		baseInstanceId?: string,
	) {
		for (const option of options.filter(isSyncedOptionAvailable)) {
			draftOptions.value[option] = enabled
			if (enabled && !initialOptions.value[option] && baseInstanceId) {
				draftSourceInstanceIds.value[option] = baseInstanceId
			} else {
				draftSourceInstanceIds.value[option] = undefined
			}
		}
	}

	function isInitiallyEnabled(option: SyncUpdateOption) {
		return initialOptions.value[option]
	}

	async function applyDraft() {
		if (!draftInitialized.value) return

		const changes = syncUpdateOptions
			.filter((option) => draftOptions.value[option] !== initialOptions.value[option])
			.map((option) => ({
				option,
				enabled: draftOptions.value[option],
				baseInstanceId: draftOptions.value[option]
					? draftSourceInstanceIds.value[option]
					: undefined,
			}))
		if (changes.length === 0) return

		await syncMutation.mutateAsync(changes)
		draftSourceInstanceIds.value = {}
	}

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
		draftInitialized,
		draftOptions,
		syncMutation,
		sourceOptions,
		sourceInstanceId,
		sources,
		sourcesLoading,
		sourcesError,
		beginDraft,
		finishDraft,
		stageOptions,
		isInitiallyEnabled,
		applyDraft,
		chooseSource,
		retrySources,
	}
}
