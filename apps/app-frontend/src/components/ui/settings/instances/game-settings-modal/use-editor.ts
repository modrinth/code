import { defineMessages, injectNotificationManager, useVIntl } from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, nextTick, onScopeDispose, ref, toValue } from 'vue'

import {
	type EditableGameSetting,
	type GameOptionCanonicalValue,
	type GameSettingChange,
	type GameSettingsEditorState,
	get_local_game_options_config,
	get_synced_game_options_config,
	preview_local_game_option_changes,
	preview_synced_game_option_changes,
	save_local_game_option_changes,
	save_synced_game_option_changes,
	type UpdateGameSettingsRequest,
} from '@/helpers/game-options'

import { canonicalValuesEqual, cloneGameSettingsState, gameSettingChanges } from './editors'

function editorQueryKey(instanceId?: string) {
	return instanceId
		? (['game-settings', 'local', instanceId] as const)
		: (['game-settings', 'synced'] as const)
}

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
			defaultMessage: 'We loaded the latest settings. Check your changes, then save again.',
		},
	})

	const baseState = ref<GameSettingsEditorState | null>(null)
	const draftState = ref<GameSettingsEditorState | null>(null)
	const touchedValueOptionIds = ref<Set<string>>(new Set())
	let previewTimer: ReturnType<typeof setTimeout> | null = null
	let previewGeneration = 0
	let loadGeneration = 0
	let saveGeneration = 0

	const editorInstanceId = ref<string>()
	const isLocalEditor = computed(() => !!editorInstanceId.value)

	const dirtyChanges = computed(() =>
		gameSettingChanges(baseState.value, draftState.value, touchedValueOptionIds.value),
	)
	const isDirty = computed(() => dirtyChanges.value.length > 0)
	const dirtyOptionIds = computed(
		() => new Set(dirtyChanges.value.map((change) => change.option_id)),
	)
	const hasBlockingDraft = computed(
		() =>
			draftState.value?.settings.some(
				(setting) =>
					dirtyOptionIds.value.has(setting.option_id) &&
					(isLocalEditor.value || setting.sync_enabled) &&
					(!!setting.validation_error ||
						['mixed', 'unset', 'invalid'].includes(setting.value_state)),
			) ?? false,
	)
	const stateQuery = useQuery(
		computed(() => {
			const instanceId = editorInstanceId.value
			return {
				queryKey: editorQueryKey(instanceId),
				queryFn: () =>
					instanceId ? get_local_game_options_config(instanceId) : get_synced_game_options_config(),
				enabled: false,
				retry: false,
			}
		}),
	)
	const loading = computed(() => stateQuery.isPending.value || stateQuery.isFetching.value)
	const previewMutation = useMutation({
		mutationFn: ({
			instanceId,
			request,
		}: {
			instanceId?: string
			request: UpdateGameSettingsRequest
		}) =>
			instanceId
				? preview_local_game_option_changes(instanceId, request)
				: preview_synced_game_option_changes(request),
	})
	const saveMutation = useMutation({
		mutationFn: saveDraft,
		onError: handleError,
	})

	async function load() {
		const generation = ++loadGeneration
		cancelPreview()
		editorInstanceId.value = toValue(instanceId)
		await nextTick()
		const result = await stateQuery.refetch()
		if (generation !== loadGeneration) return false
		if (result.isError) {
			handleError(result.error)
			return false
		}
		if (!result.data) return false
		baseState.value = cloneGameSettingsState(result.data)
		draftState.value = cloneGameSettingsState(result.data)
		touchedValueOptionIds.value = new Set()
		return true
	}

	function cancelPreview() {
		if (previewTimer) clearTimeout(previewTimer)
		previewTimer = null
		previewGeneration++
	}

	function reset() {
		cancelPreview()
		loadGeneration++
		saveGeneration++
		baseState.value = null
		draftState.value = null
		touchedValueOptionIds.value = new Set()
		saveMutation.reset()
	}

	function cancelChanges() {
		if (!baseState.value) return
		cancelPreview()
		draftState.value = cloneGameSettingsState(baseState.value)
		touchedValueOptionIds.value = new Set()
	}

	function setSyncEnabled(optionIds: readonly string[], enabled: boolean) {
		if (!draftState.value) return
		const ids = new Set(optionIds)
		draftState.value.settings = draftState.value.settings.map((setting) =>
			ids.has(setting.option_id)
				? { ...setting, sync_enabled: enabled, validation_error: null }
				: setting,
		)
		schedulePreview()
	}

	function schedulePreview() {
		cancelPreview()
		previewTimer = setTimeout(() => void preview(), 350)
	}

	function save() {
		if (saveMutation.isPending.value || loading.value || hasBlockingDraft.value || !isDirty.value)
			return
		saveMutation.mutate()
	}

	function editorRequest(): UpdateGameSettingsRequest | null {
		if (!baseState.value || !draftState.value) return null
		return {
			expected_summary_revision: draftState.value.summary_revision,
			expected_canonical_revision: draftState.value.canonical_revision,
			expected_catalog_revision: draftState.value.catalog_revision,
			changes: dirtyChanges.value,
		}
	}

	function setCanonicalValue(optionId: string, value: GameOptionCanonicalValue | null) {
		if (!draftState.value) return
		const currentSetting = draftState.value.settings.find(
			(setting) => setting.option_id === optionId,
		)
		const baseSetting = baseState.value?.settings.find((setting) => setting.option_id === optionId)
		if (!currentSetting) return

		const revertedToBase =
			!!baseSetting &&
			!canonicalValuesEqual(currentSetting.canonical_value, baseSetting.canonical_value) &&
			canonicalValuesEqual(value, baseSetting.canonical_value)
		const touchedOptionIds = new Set(touchedValueOptionIds.value)
		if (revertedToBase) {
			touchedOptionIds.delete(optionId)
		} else {
			touchedOptionIds.add(optionId)
		}
		touchedValueOptionIds.value = touchedOptionIds
		draftState.value.settings = draftState.value.settings.map((setting) =>
			setting.option_id === optionId
				? {
						...setting,
						canonical_value: value,
						value_state: revertedToBase ? baseSetting.value_state : value ? 'canonical' : 'unset',
						validation_error: revertedToBase ? baseSetting.validation_error : null,
					}
				: setting,
		)
		schedulePreview()
	}

	function mergePreview(preview: GameSettingsEditorState) {
		if (!baseState.value || !draftState.value) return
		const previousBase = cloneGameSettingsState(baseState.value)
		const previousDraft = cloneGameSettingsState(draftState.value)
		const dirtyIds = new Set(
			gameSettingChanges(previousBase, previousDraft, touchedValueOptionIds.value).map(
				(change) => change.option_id,
			),
		)
		const stagedSettings = new Map(
			previousDraft.settings
				.filter((setting) => dirtyIds.has(setting.option_id))
				.map((setting) => [setting.option_id, setting]),
		)
		const previousBaseSettings = new Map(
			previousBase.settings.map((setting) => [setting.option_id, setting]),
		)
		const nextBase = cloneGameSettingsState(preview)
		const nextDraft = cloneGameSettingsState(preview)
		nextBase.settings = nextBase.settings.map((setting) =>
			dirtyIds.has(setting.option_id)
				? { ...(previousBaseSettings.get(setting.option_id) ?? setting) }
				: setting,
		)
		nextDraft.settings = nextDraft.settings.map((setting): EditableGameSetting => {
			const staged = stagedSettings.get(setting.option_id)
			if (!staged) return setting
			return {
				...setting,
				sync_enabled: staged.sync_enabled,
				canonical_value: staged.canonical_value,
				option_revision: staged.option_revision,
			}
		})
		baseState.value = nextBase
		draftState.value = nextDraft
	}

	async function preview() {
		const request = editorRequest()
		if (!request || request.changes.length === 0) {
			previewGeneration++
			if (baseState.value) draftState.value = cloneGameSettingsState(baseState.value)
			return
		}

		const generation = ++previewGeneration
		try {
			const previewState = await previewMutation.mutateAsync({
				instanceId: editorInstanceId.value,
				request,
			})
			if (generation !== previewGeneration) return
			mergePreview(previewState)
		} catch {
			return
		}
	}

	function applyChangesToRefreshedState(
		refreshed: GameSettingsEditorState,
		stagedDraft: GameSettingsEditorState,
		changes: GameSettingChange[],
		touchedOptionIds: ReadonlySet<string>,
		conflictOptionIds: ReadonlySet<string> = new Set(),
	) {
		const changesById = new Map(changes.map((change) => [change.option_id, change]))
		const stagedSettings = new Map(
			stagedDraft.settings.map((setting) => [setting.option_id, setting]),
		)
		const nextBase = cloneGameSettingsState(refreshed)
		const nextDraft = cloneGameSettingsState(refreshed)

		nextDraft.settings = nextDraft.settings.map((setting) => {
			const change = changesById.get(setting.option_id)
			const staged = stagedSettings.get(setting.option_id)
			if (!change || !staged) return setting

			return {
				...setting,
				...(change.sync_enabled !== undefined ? { sync_enabled: staged.sync_enabled } : {}),
				...(change.canonical_value !== undefined
					? {
							canonical_value: staged.canonical_value,
							value_state: staged.value_state,
						}
					: {}),
				validation_error: conflictOptionIds.has(setting.option_id)
					? 'changed_since_opened'
					: staged.validation_error,
			}
		})

		baseState.value = nextBase
		draftState.value = nextDraft
		touchedValueOptionIds.value = new Set(
			[...touchedOptionIds].filter(
				(optionId) => changesById.get(optionId)?.canonical_value !== undefined,
			),
		)
	}

	async function saveDraft() {
		const request = editorRequest()
		if (!request || request.changes.length === 0 || !baseState.value || !draftState.value) return

		cancelPreview()
		const targetInstanceId = editorInstanceId.value
		const generation = ++saveGeneration
		const previousBase = cloneGameSettingsState(baseState.value)
		const optimisticState = cloneGameSettingsState(draftState.value)
		const previouslyTouchedOptionIds = new Set(touchedValueOptionIds.value)
		baseState.value = cloneGameSettingsState(optimisticState)
		touchedValueOptionIds.value = new Set()
		try {
			const result = targetInstanceId
				? await save_local_game_option_changes(targetInstanceId, request)
				: await save_synced_game_option_changes(request)
			const refreshed =
				result.state ??
				(targetInstanceId
					? await get_local_game_options_config(targetInstanceId)
					: await get_synced_game_options_config())
			queryClient.setQueryData(editorQueryKey(targetInstanceId), refreshed)
			if (result.conflicts?.length) {
				if (generation !== saveGeneration || !draftState.value) return
				previewGeneration++
				const stagedDraft = cloneGameSettingsState(draftState.value)
				const retainedTouchedOptionIds = new Set([
					...previouslyTouchedOptionIds,
					...touchedValueOptionIds.value,
				])
				const stagedChanges = gameSettingChanges(
					previousBase,
					stagedDraft,
					retainedTouchedOptionIds,
				)
				const conflicts = new Set(result.conflicts)
				applyChangesToRefreshedState(
					refreshed,
					stagedDraft,
					stagedChanges,
					retainedTouchedOptionIds,
					conflicts,
				)

				addNotification({
					type: 'warning',
					title: formatMessage(messages.conflictTitle),
					text: formatMessage(messages.conflictText),
				})
				if (isDirty.value) {
					schedulePreview()
				}
				return
			}

			if (generation !== saveGeneration) {
				onSaved()
				return
			}
			if (!baseState.value || !draftState.value) return
			previewGeneration++
			const stagedDraft = cloneGameSettingsState(draftState.value)
			const touchedSinceSave = new Set(touchedValueOptionIds.value)
			const changesSinceSave = gameSettingChanges(optimisticState, stagedDraft, touchedSinceSave)
			applyChangesToRefreshedState(refreshed, stagedDraft, changesSinceSave, touchedSinceSave)
			if (changesSinceSave.length > 0) schedulePreview()
			onSaved()
		} catch (error) {
			if (generation === saveGeneration && draftState.value) {
				previewGeneration++
				baseState.value = previousBase
				touchedValueOptionIds.value = new Set([
					...previouslyTouchedOptionIds,
					...touchedValueOptionIds.value,
				])
				if (dirtyChanges.value.length > 0) schedulePreview()
			}
			throw error
		}
	}

	onScopeDispose(reset)

	return {
		draftState,
		isLocalEditor,
		isDirty,
		hasBlockingDraft,
		loading,
		loadError: stateQuery.isError,
		saving: saveMutation.isPending,
		load,
		reset,
		cancelChanges,
		setSyncEnabled,
		setCanonicalValue,
		save,
	}
}
