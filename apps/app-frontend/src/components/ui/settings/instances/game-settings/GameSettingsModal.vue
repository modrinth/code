<script setup lang="ts">
import {
	EyeIcon,
	LanguagesIcon,
	LinkIcon,
	MessageIcon,
	MonitorIcon,
	RefreshCwIcon,
	SaveIcon,
	SearchIcon,
	SettingsIcon,
	ShirtIcon,
	SpinnerIcon,
	TagCategoryAudioIcon,
	TagCategoryGamepad2Icon,
	UnlinkIcon,
	WrenchIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Button,
	commonMessages,
	ConfirmLeaveModal,
	defineMessages,
	FloatingActionBar,
	injectNotificationManager,
	Input,
	TabbedModal,
	type TabbedModalTab,
	useVIntl,
} from '@modrinth/ui'
import type { Component } from 'vue'
import { computed, onUnmounted, ref } from 'vue'

import {
	type EditableGameSetting,
	type GameOptionCanonicalValue,
	type GameSettingChange,
	type GameSettingCategory,
	type GameSettingsEditorState,
	get_local_game_options_config,
	get_synced_game_options_config,
	preview_local_game_option_changes,
	preview_synced_game_option_changes,
	save_local_game_option_changes,
	save_synced_game_option_changes,
	type UpdateGameSettingsRequest,
} from '@/helpers/game-options'

import { minecraftKeybindConflictKey } from './game-keybinds'
import {
	canonicalValuesEqual,
	cloneGameSettingsState,
	gameSettingChanges,
	settingCanBeEnabled,
	settingSearchText,
} from './game-setting-editors'
import {
	formatGameSettingDescription,
	formatGameSettingLabel,
	gameSettingCategoryMessage,
} from './game-setting-messages'
import GameSettingRow from './GameSettingRow.vue'

const props = defineProps<{
	instanceId?: string
}>()

const emit = defineEmits<{
	saved: []
}>()

const { formatMessage } = useVIntl()
const { addNotification, handleError } = injectNotificationManager()

const messages = defineMessages({
	title: {
		id: 'app.settings.synced-options.game-settings.title',
		defaultMessage: 'Game settings',
	},
	syncTitle: {
		id: 'app.settings.synced-options.game-settings.sync-title',
		defaultMessage: 'Sync game settings',
	},
	syncedOptionsDescription: {
		id: 'app.settings.synced-options.game-settings.description',
		defaultMessage:
			'Synced options are applied to every instance and override the values included with a modpack.',
	},
	search: {
		id: 'app.settings.synced-options.game-settings.search',
		defaultMessage: 'Search settings...',
	},
	enableAll: {
		id: 'app.settings.synced-options.game-settings.enable-all',
		defaultMessage: 'Sync tab',
	},
	disableAll: {
		id: 'app.settings.synced-options.game-settings.disable-all',
		defaultMessage: 'Unsync all',
	},
	customEmpty: {
		id: 'app.settings.synced-options.game-settings.custom-empty',
		defaultMessage:
			'Settings added by mods will appear here after Modrinth finds them in one of your instances.',
	},
	empty: {
		id: 'app.settings.synced-options.game-settings.empty',
		defaultMessage: 'No settings match your search or filter.',
	},
	loading: {
		id: 'app.settings.synced-options.game-settings.loading',
		defaultMessage: 'Loading settings...',
	},
	loadFailed: {
		id: 'app.settings.synced-options.game-settings.load-failed',
		defaultMessage: 'We couldn’t load your game settings.',
	},
	retry: {
		id: 'app.settings.synced-options.game-settings.retry',
		defaultMessage: 'Retry',
	},
	saveSettings: {
		id: 'app.settings.synced-options.game-settings.save',
		defaultMessage: 'Save settings',
	},
	unsavedChanges: {
		id: 'app.settings.synced-options.game-settings.unsaved-changes',
		defaultMessage: 'You have unsaved changes.',
	},
	conflictTitle: {
		id: 'app.settings.synced-options.game-settings.conflict-title',
		defaultMessage: 'These settings changed elsewhere',
	},
	conflictText: {
		id: 'app.settings.synced-options.game-settings.conflict-text',
		defaultMessage: 'We loaded the latest settings. Check your changes, then save again.',
	},
})

const modal = ref<InstanceType<typeof TabbedModal> | null>(null)
const confirmLeaveModal = ref<InstanceType<typeof ConfirmLeaveModal> | null>(null)
const baseState = ref<GameSettingsEditorState | null>(null)
const draftState = ref<GameSettingsEditorState | null>(null)
const activeCategoryId = ref('')
const search = ref('')
const loading = ref(false)
const saving = ref(false)
const loadError = ref(false)
const touchedValueOptionIds = ref<Set<string>>(new Set())
let allowClose = false
let previewTimer: ReturnType<typeof setTimeout> | null = null
let previewGeneration = 0
let loadGeneration = 0
let saveGeneration = 0

const isLocalEditor = computed(() => !!props.instanceId)

const categoryIcons: Record<string, Component> = {
	skin_customization: ShirtIcon,
	video: MonitorIcon,
	video_settings: MonitorIcon,
	language: LanguagesIcon,
	music_and_sound: TagCategoryAudioIcon,
	controls: TagCategoryGamepad2Icon,
	chat: MessageIcon,
	chat_settings: MessageIcon,
	accessibility: EyeIcon,
	custom: WrenchIcon,
	custom_settings: WrenchIcon,
}

const categories = computed<GameSettingCategory[]>(() => {
	if (!draftState.value) return []

	const settings = draftState.value.settings
	const visible = draftState.value.categories.filter(
		(category) =>
			category.is_custom || settings.some((setting) => setting.category_id === category.id),
	)
	if (!visible.some((category) => category.is_custom || category.id === 'custom_settings')) {
		visible.push({
			id: 'custom_settings',
			is_custom: true,
		})
	}
	return visible
})

const categoryTabs = computed<TabbedModalTab[]>(() =>
	categories.value.map((category) => ({
		name: gameSettingCategoryMessage(category),
		icon: categoryIcon(category),
	})),
)

const activeCategory = computed(() =>
	categories.value.find((category) => category.id === activeCategoryId.value),
)

const categorySettings = computed(() => {
	if (!draftState.value) return []

	const query = search.value.trim().toLocaleLowerCase()
	return draftState.value.settings.filter((setting) => {
		if (!query && setting.category_id !== activeCategoryId.value) return false
		if (
			query &&
			!settingSearchText(
				setting,
				formatGameSettingLabel(formatMessage, setting),
				formatGameSettingDescription(formatMessage, setting),
			).includes(query)
		)
			return false
		return true
	})
})

const keybindConflicts = computed(() => {
	if (!draftState.value) return new Map<string, string[]>()

	const bindings = new Map<string, EditableGameSetting[]>()
	for (const setting of draftState.value.settings) {
		if (setting.editor.type !== 'key_binding' || setting.canonical_value?.type !== 'key_binding')
			continue
		const key = minecraftKeybindConflictKey(setting.canonical_value.value)
		if (!key) continue
		bindings.set(key, [...(bindings.get(key) ?? []), setting])
	}

	const conflicts = new Map<string, string[]>()
	for (const settings of bindings.values()) {
		if (settings.length < 2) continue
		for (const setting of settings) {
			conflicts.set(
				setting.option_id,
				settings
					.filter((candidate) => candidate.option_id !== setting.option_id)
					.map((candidate) => formatGameSettingLabel(formatMessage, candidate)),
			)
		}
	}
	return conflicts
})

const dirtyChanges = computed(() =>
	gameSettingChanges(baseState.value, draftState.value, touchedValueOptionIds.value),
)
const isDirty = computed(() => dirtyChanges.value.length > 0)
const dirtyOptionIds = computed(() => new Set(dirtyChanges.value.map((change) => change.option_id)))
const hasBlockingDraft = computed(
	() =>
		draftState.value?.settings.some(
			(setting) =>
				dirtyOptionIds.value.has(setting.option_id) &&
				(isLocalEditor.value || setting.sync_enabled) &&
				(!!setting.validation_error || ['mixed', 'unset', 'invalid'].includes(setting.value_state)),
		) ?? false,
)
const enableCandidates = computed(() =>
	categorySettings.value.filter((setting) => !setting.sync_enabled && settingCanBeEnabled(setting)),
)
const disableCandidates = computed(() =>
	categorySettings.value.filter((setting) => setting.sync_enabled && !setting.controlled),
)
const toggleableCategorySettings = computed(() =>
	categorySettings.value.filter((setting) => !setting.controlled),
)
const allCategorySettingsSynced = computed(
	() =>
		toggleableCategorySettings.value.length > 0 &&
		toggleableCategorySettings.value.every((setting) => setting.sync_enabled),
)
const modalTitle = computed(() =>
	formatMessage(isLocalEditor.value ? messages.title : messages.syncTitle),
)

function categoryIcon(category: GameSettingCategory): Component {
	return categoryIcons[category.id] ?? SettingsIcon
}

function changeCategory(_fromIndex: number, toIndex: number): boolean {
	const category = categories.value[toIndex]
	if (!category) return false
	activeCategoryId.value = category.id
	search.value = ''
	return true
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

async function load() {
	const generation = ++loadGeneration
	previewGeneration++
	loading.value = true
	loadError.value = false
	try {
		const state = props.instanceId
			? await get_local_game_options_config(props.instanceId)
			: await get_synced_game_options_config()
		if (generation !== loadGeneration) return
		baseState.value = cloneGameSettingsState(state)
		draftState.value = cloneGameSettingsState(state)
		touchedValueOptionIds.value = new Set()
		const currentCategoryExists = categories.value.some(
			(category) => category.id === activeCategoryId.value,
		)
		if (!currentCategoryExists)
			activeCategoryId.value = categories.value[0]?.id ?? 'custom_settings'
		const activeCategoryIndex = categories.value.findIndex(
			(category) => category.id === activeCategoryId.value,
		)
		if (activeCategoryIndex >= 0) modal.value?.setTab(activeCategoryIndex)
	} catch (error) {
		if (generation !== loadGeneration) return
		loadError.value = true
		handleError(error)
	} finally {
		if (generation === loadGeneration) loading.value = false
	}
}

function show() {
	allowClose = false
	search.value = ''
	modal.value?.show()
	void load()
}

function hide() {
	modal.value?.hide()
}

function reset() {
	if (previewTimer) clearTimeout(previewTimer)
	previewTimer = null
	previewGeneration++
	loadGeneration++
	saveGeneration++
	baseState.value = null
	draftState.value = null
	loading.value = false
	saving.value = false
	loadError.value = false
	touchedValueOptionIds.value = new Set()
	allowClose = false
}

function beforeHide(): boolean {
	if (allowClose || !isDirty.value) return true
	void confirmDiscard()
	return false
}

async function confirmDiscard() {
	const discard = await confirmLeaveModal.value?.prompt()
	if (!discard) return
	allowClose = true
	modal.value?.hide()
}

function cancelChanges() {
	if (!baseState.value) return
	if (previewTimer) clearTimeout(previewTimer)
	previewTimer = null
	previewGeneration++
	draftState.value = cloneGameSettingsState(baseState.value)
	touchedValueOptionIds.value = new Set()
}

function settingById(optionId: string): EditableGameSetting | undefined {
	return draftState.value?.settings.find((setting) => setting.option_id === optionId)
}

function setSyncEnabled(optionId: string, enabled: boolean) {
	if (!draftState.value) return
	draftState.value.settings = draftState.value.settings.map((setting) =>
		setting.option_id === optionId
			? { ...setting, sync_enabled: enabled, validation_error: null }
			: setting,
	)
	schedulePreview()
}

function setCanonicalValue(optionId: string, value: GameOptionCanonicalValue | null) {
	if (!draftState.value) return
	const currentSetting = settingById(optionId)
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

function enableVisible() {
	if (!draftState.value) return
	const optionIds = new Set(enableCandidates.value.map((setting) => setting.option_id))
	draftState.value.settings = draftState.value.settings.map((setting) =>
		optionIds.has(setting.option_id)
			? { ...setting, sync_enabled: true, validation_error: null }
			: setting,
	)
	schedulePreview()
}

function disableVisible() {
	if (!draftState.value) return
	const optionIds = new Set(disableCandidates.value.map((setting) => setting.option_id))
	draftState.value.settings = draftState.value.settings.map((setting) =>
		optionIds.has(setting.option_id)
			? { ...setting, sync_enabled: false, validation_error: null }
			: setting,
	)
	schedulePreview()
}

function toggleVisibleSync() {
	if (allCategorySettingsSynced.value) {
		disableVisible()
	} else {
		enableVisible()
	}
}

function schedulePreview() {
	previewGeneration++
	if (previewTimer) clearTimeout(previewTimer)
	previewTimer = setTimeout(() => void preview(), 350)
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
		const previewState = props.instanceId
			? await preview_local_game_option_changes(props.instanceId, request)
			: await preview_synced_game_option_changes(request)
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

async function save() {
	if (saving.value) return
	const request = editorRequest()
	if (!request || request.changes.length === 0 || !baseState.value || !draftState.value) return

	if (previewTimer) clearTimeout(previewTimer)
	previewTimer = null
	previewGeneration++
	const generation = ++saveGeneration
	const previousBase = cloneGameSettingsState(baseState.value)
	const optimisticState = cloneGameSettingsState(draftState.value)
	const previouslyTouchedOptionIds = new Set(touchedValueOptionIds.value)
	baseState.value = cloneGameSettingsState(optimisticState)
	touchedValueOptionIds.value = new Set()
	saving.value = true
	try {
		const result = props.instanceId
			? await save_local_game_option_changes(props.instanceId, request)
			: await save_synced_game_option_changes(request)
		if (result.conflicts?.length) {
			const refreshed =
				result.state ??
				(props.instanceId
					? await get_local_game_options_config(props.instanceId)
					: await get_synced_game_options_config())
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
			if (
				gameSettingChanges(baseState.value, draftState.value, touchedValueOptionIds.value).length >
				0
			) {
				schedulePreview()
			}
			return
		}

		const refreshed =
			result.state ??
			(props.instanceId
				? await get_local_game_options_config(props.instanceId)
				: await get_synced_game_options_config())
		if (generation !== saveGeneration) {
			emit('saved')
			return
		}
		if (!baseState.value || !draftState.value) return
		previewGeneration++
		const stagedDraft = cloneGameSettingsState(draftState.value)
		const touchedSinceSave = new Set(touchedValueOptionIds.value)
		const changesSinceSave = gameSettingChanges(
			optimisticState,
			stagedDraft,
			touchedSinceSave,
		)
		applyChangesToRefreshedState(
			refreshed,
			stagedDraft,
			changesSinceSave,
			touchedSinceSave,
		)
		if (changesSinceSave.length > 0) schedulePreview()
		emit('saved')
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
		handleError(error)
	} finally {
		if (generation === saveGeneration) saving.value = false
	}
}

onUnmounted(() => {
	if (previewTimer) clearTimeout(previewTimer)
})

defineExpose({ show, hide })
</script>

<template>
	<TabbedModal
		ref="modal"
		:tabs="categoryTabs"
		:header="modalTitle"
		:before-hide="beforeHide"
		:before-tab-change="changeCategory"
		:on-after-hide="reset"
		:floating-action-bar-shown="isDirty"
		:max-width="'min(1080px, calc(95vw - 2rem))'"
		:width="'min(1080px, calc(95vw - 2rem))'"
	>
		<template #sidebar-header>
			<div class="pb-4">
				<Input
					v-model="search"
					:icon="SearchIcon"
					type="search"
					autocomplete="off"
					:placeholder="formatMessage(messages.search)"
					wrapper-class="w-full shrink-0"
				/>
			</div>
		</template>

		<template #content>
			<div class="flex min-h-full min-w-0 flex-col">
				<div
					v-if="!isLocalEditor"
					class="flex shrink-0 flex-wrap items-start justify-between gap-4 pb-4"
				>
					<p class="m-0 min-w-60 flex-1 text-primary">
						{{ formatMessage(messages.syncedOptionsDescription) }}
					</p>
					<div class="ml-auto flex w-48 justify-end">
						<Button
							size="lg"
							:disabled="
								allCategorySettingsSynced
									? disableCandidates.length === 0
									: enableCandidates.length === 0
							"
							@click="toggleVisibleSync"
						>
							<LinkIcon v-if="allCategorySettingsSynced" />
							<UnlinkIcon v-else />
							{{
								formatMessage(allCategorySettingsSynced ? messages.disableAll : messages.enableAll)
							}}
						</Button>
					</div>
				</div>

				<div
					v-if="loading"
					class="flex min-h-40 flex-1 items-center justify-center gap-2 text-secondary"
				>
					<SpinnerIcon class="size-5 animate-spin" />
					{{ formatMessage(messages.loading) }}
				</div>
				<div
					v-else-if="loadError"
					class="flex min-h-40 flex-1 flex-col items-center justify-center gap-3 text-secondary"
				>
					<p class="m-0">{{ formatMessage(messages.loadFailed) }}</p>
					<Button @click="load">
						<RefreshCwIcon />
						{{ formatMessage(messages.retry) }}
					</Button>
				</div>
				<div v-else class="min-h-0 flex-1">
					<div
						v-if="categorySettings.length === 0"
						class="flex h-full min-h-40 items-center justify-center px-8 text-center text-secondary"
					>
						{{
							activeCategory?.is_custom && !search
								? formatMessage(messages.customEmpty)
								: formatMessage(messages.empty)
						}}
					</div>
					<div v-else class="flex flex-col gap-4">
						<GameSettingRow
							v-for="setting in categorySettings"
							:key="setting.option_id"
							:setting="setting"
							:keybind-conflicts="keybindConflicts.get(setting.option_id)"
							:show-sync-toggle="!isLocalEditor"
							@update:sync-enabled="setSyncEnabled(setting.option_id, $event)"
							@update:canonical-value="setCanonicalValue(setting.option_id, $event)"
						/>
					</div>
				</div>
			</div>
		</template>

		<template #floating-action-bar>
			<FloatingActionBar v-if="isDirty" :shown="true" :inline="true" :aria-label="modalTitle">
				<p class="m-0 text-sm font-semibold md:text-base">
					{{ formatMessage(messages.unsavedChanges) }}
				</p>
				<div class="ml-auto flex gap-2">
					<Button type="outlined" @click="cancelChanges">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</Button>
					<Button
						type="colored"
						color="brand"
						:disabled="!isDirty || loading || saving || hasBlockingDraft"
						:loading="saving"
						@click="save"
					>
						<SaveIcon v-if="!saving" />
						{{ formatMessage(messages.saveSettings) }}
					</Button>
				</div>
			</FloatingActionBar>
		</template>
	</TabbedModal>

	<ConfirmLeaveModal ref="confirmLeaveModal" />
</template>
