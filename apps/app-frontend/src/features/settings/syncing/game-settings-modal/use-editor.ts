import { defineMessages, injectNotificationManager, useVIntl } from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { useDebounceFn } from '@vueuse/core'
import {
	computed,
	type MaybeRefOrGetter,
	nextTick,
	onScopeDispose,
	ref,
	shallowRef,
	toValue,
} from 'vue'

import {
	type EditableGameSetting,
	type GameOptionCanonicalValue,
	type GameSettingsEditorState,
	gameSettingsKeys,
	gameSettingsQueryOptions,
	get_local_game_options_config,
	get_synced_game_options_config,
	save_local_game_option_changes,
	save_synced_game_option_changes,
	type UpdateGameSettingsRequest,
} from '@/helpers/game-options'

import {
	applyGameSettingEdits,
	canonicalValuesEqual,
	type GameSettingEdit,
	restoreGameSettingValues,
} from './editors'

type SaveVariables = {
	editId: number
	instanceId?: string
	edits: GameSettingEdit[]
}

type InFlightEdit = Omit<SaveVariables, 'edits'> & { edits: readonly GameSettingEdit[] }

const saveMutationKey = ['game-settings', 'save'] as const

export function useGameSettingsEditor(
	instanceId: MaybeRefOrGetter<string | undefined>,
	onSaved: () => void,
) {
	const { addNotification, handleError } = injectNotificationManager()
	const { formatMessage } = useVIntl()
	const queryClient = useQueryClient()
	const messages = defineMessages({
		conflictTitle: {
			id: 'app.settings.synced-options.game-settings.conflict-title',
			defaultMessage: 'These settings changed elsewhere',
		},
		conflictText: {
			id: 'app.settings.synced-options.game-settings.conflict-text',
			defaultMessage: 'We loaded the latest settings. Check your change and try again.',
		},
	})

	const editorInstanceId = ref<string>()
	const active = ref(false)
	const loaded = ref(false)
	const inFlightEdits = shallowRef<InFlightEdit[]>([])
	const pendingValues = shallowRef(new Map<string, GameOptionCanonicalValue | null>())
	let nextEditId = 0
	let loadGeneration = 0
	let savedSinceIdle = false

	const isLocalEditor = computed(() => !!editorInstanceId.value)

	const stateQuery = useQuery(
		computed(() => ({
			...gameSettingsQueryOptions(editorInstanceId.value),
			enabled: active.value,
		})),
	)

	const savedState = computed<GameSettingsEditorState | null>(() => {
		const data = stateQuery.data.value
		if (!loaded.value || !data) return null
		return applyGameSettingEdits(
			data,
			inFlightEdits.value
				.filter((entry) => entry.instanceId === editorInstanceId.value)
				.flatMap((entry) => entry.edits),
		)
	})
	const state = computed<GameSettingsEditorState | null>(() => {
		if (!savedState.value) return null
		return applyGameSettingEdits(
			savedState.value,
			[...pendingValues.value].map(([option_id, canonical_value]) => ({
				option_id,
				canonical_value,
			})),
		)
	})
	const loading = computed(
		() => !state.value && (stateQuery.isPending.value || stateQuery.isFetching.value),
	)
	const loadError = computed(() => !state.value && stateQuery.isError.value)

	const saveMutation = useMutation({
		mutationKey: saveMutationKey,
		networkMode: 'always',
		scope: { id: 'game-settings' },
		mutationFn: saveEdits,
		onMutate: async ({ instanceId: targetInstanceId, edits }: SaveVariables) => {
			const queryKey = gameSettingsQueryOptions(targetInstanceId).queryKey
			await queryClient.cancelQueries({ queryKey })
			const previous = queryClient.getQueryData(queryKey)
			const editedIds = new Set(edits.map((edit) => edit.option_id))
			const snapshot: EditableGameSetting[] =
				previous?.settings.filter((setting) => editedIds.has(setting.option_id)) ?? []
			if (previous) queryClient.setQueryData(queryKey, applyGameSettingEdits(previous, edits))
			return { snapshot }
		},
		onSuccess: ({ conflicts, refreshed }, { instanceId: targetInstanceId }) => {
			queryClient.setQueryData(gameSettingsQueryOptions(targetInstanceId).queryKey, refreshed)
			savedSinceIdle = true
			if (conflicts.length > 0) {
				addNotification({
					type: 'warning',
					title: formatMessage(messages.conflictTitle),
					text: formatMessage(messages.conflictText),
				})
			}
		},
		onError: (error, { instanceId: targetInstanceId }, context) => {
			if (context?.snapshot.length) {
				queryClient.setQueryData(gameSettingsQueryOptions(targetInstanceId).queryKey, (current) =>
					current ? restoreGameSettingValues(current, context.snapshot) : current,
				)
			}
			handleError(error)
		},
		onSettled: (_data, _error, { editId, instanceId: targetInstanceId }) => {
			inFlightEdits.value = inFlightEdits.value.filter((entry) => entry.editId !== editId)
			if (queryClient.isMutating({ mutationKey: saveMutationKey }) > 1) return
			void queryClient.invalidateQueries({
				queryKey: gameSettingsQueryOptions(targetInstanceId).queryKey,
			})
			if (targetInstanceId) {
				void queryClient.invalidateQueries({ queryKey: gameSettingsKeys.synced })
			}
			if (!savedSinceIdle) return
			savedSinceIdle = false
			onSaved()
		},
	})

	async function saveEdits({ instanceId: targetInstanceId, edits }: SaveVariables) {
		const options = gameSettingsQueryOptions(targetInstanceId)
		await queryClient.cancelQueries({ queryKey: options.queryKey })
		const current = await queryClient.ensureQueryData(options)
		const revisions = new Map(
			current.settings.map((setting) => [setting.option_id, setting.option_revision]),
		)
		const request: UpdateGameSettingsRequest = {
			expected_summary_revision: current.summary_revision,
			expected_canonical_revision: current.canonical_revision,
			expected_catalog_revision: current.catalog_revision,
			changes: edits.flatMap((edit) => {
				const revision = revisions.get(edit.option_id)
				return revision === undefined ? [] : [{ ...edit, base_option_revision: revision }]
			}),
		}
		const result = targetInstanceId
			? await save_local_game_option_changes(targetInstanceId, request)
			: await save_synced_game_option_changes(request)
		const refreshed =
			result.state ??
			(targetInstanceId
				? await get_local_game_options_config(targetInstanceId)
				: await get_synced_game_options_config())
		return { conflicts: result.conflicts ?? [], refreshed }
	}

	const debouncedFlush = useDebounceFn(flushPendingValues, 500)

	function persist(edits: GameSettingEdit[]) {
		if (edits.length === 0) return
		const variables: SaveVariables = {
			editId: ++nextEditId,
			instanceId: editorInstanceId.value,
			edits,
		}
		inFlightEdits.value = [...inFlightEdits.value, variables]
		saveMutation.mutate(variables)
	}

	function valueEdits(
		values: Iterable<[string, GameOptionCanonicalValue | null]>,
	): GameSettingEdit[] {
		const settings = new Map(
			savedState.value?.settings.map((setting) => [setting.option_id, setting]) ?? [],
		)
		const edits: GameSettingEdit[] = []
		for (const [optionId, value] of values) {
			const setting = settings.get(optionId)
			if (!setting || value === null) continue
			const unchanged =
				setting.value_state === 'canonical' &&
				canonicalValuesEqual(setting.canonical_value, value)
			if (unchanged) continue
			edits.push({ option_id: optionId, canonical_value: value })
		}
		return edits
	}

	function flushPendingValues() {
		if (pendingValues.value.size === 0) return
		const entries = [...pendingValues.value]
		pendingValues.value = new Map(entries.filter(([, value]) => value === null))
		persist(valueEdits(entries))
	}

	async function load() {
		const generation = ++loadGeneration
		flushPendingValues()
		editorInstanceId.value = toValue(instanceId)
		active.value = true
		const options = gameSettingsQueryOptions(editorInstanceId.value)
		if (!editorInstanceId.value && queryClient.getQueryData(options.queryKey)) {
			loaded.value = true
			return true
		}
		loaded.value = false
		await nextTick()
		const result = await stateQuery.refetch({ cancelRefetch: false })
		if (generation !== loadGeneration) return false
		if (result.isError) {
			handleError(result.error)
			return false
		}
		loaded.value = !!result.data
		return loaded.value
	}

	function reset() {
		flushPendingValues()
		loadGeneration++
		active.value = false
		loaded.value = false
		pendingValues.value = new Map()
	}

	function setSyncEnabled(optionIds: readonly string[], enabled: boolean) {
		flushPendingValues()
		if (!savedState.value) return
		const ids = new Set(optionIds)
		persist(
			savedState.value.settings
				.filter((setting) => ids.has(setting.option_id) && setting.sync_enabled !== enabled)
				.map((setting) => ({ option_id: setting.option_id, sync_enabled: enabled })),
		)
	}

	function setCanonicalValue(optionId: string, value: GameOptionCanonicalValue | null) {
		if (pendingValues.value.has(optionId)) {
			pendingValues.value = new Map(
				[...pendingValues.value].filter(([pendingOptionId]) => pendingOptionId !== optionId),
			)
		}
		flushPendingValues()
		persist(valueEdits([[optionId, value]]))
	}

	function queueCanonicalValue(optionId: string, value: GameOptionCanonicalValue | null) {
		const next = new Map(pendingValues.value)
		next.set(optionId, value)
		pendingValues.value = next
		void debouncedFlush()
	}

	onScopeDispose(reset)

	return {
		state,
		isLocalEditor,
		loading,
		loadError,
		load,
		reset,
		flushPendingValues,
		setSyncEnabled,
		setCanonicalValue,
		queueCanonicalValue,
	}
}
