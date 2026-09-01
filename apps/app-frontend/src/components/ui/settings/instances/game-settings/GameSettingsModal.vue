<script setup lang="ts">
import {
	EyeIcon,
	LanguagesIcon,
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
	WrenchIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Button,
	ButtonGroup,
	commonMessages,
	ConfirmLeaveModal,
	defineMessages,
	injectNotificationManager,
	Input,
	NewModal,
	Tabs,
	type TabsTab,
	useVIntl,
} from '@modrinth/ui'
import type { Component } from 'vue'
import { computed, onUnmounted, ref } from 'vue'

import {
	get_synced_game_options_config,
	type EditableGameSetting,
	type GameOptionCanonicalValue,
	type GameSettingCategory,
	type GameSettingsEditorState,
	preview_synced_game_option_changes,
	save_synced_game_option_changes,
	type UpdateGameSettingsRequest,
} from '@/helpers/game-options'

import GameSettingRow from './GameSettingRow.vue'
import {
	cloneGameSettingsState,
	gameSettingChanges,
	settingCanBeEnabled,
	settingSearchText,
} from './game-setting-editors'
import {
	formatGameSettingCategory,
	formatGameSettingDescription,
	formatGameSettingLabel,
} from './game-setting-messages'

const emit = defineEmits<{
	saved: []
}>()

const { formatMessage } = useVIntl()
const { addNotification, handleError } = injectNotificationManager()

const messages = defineMessages({
	title: {
		id: 'app.settings.synced-options.game-settings.title',
		defaultMessage: 'Sync game settings',
	},
	search: {
		id: 'app.settings.synced-options.game-settings.search',
		defaultMessage: 'Search settings...',
	},
	all: {
		id: 'app.settings.synced-options.game-settings.filter.all',
		defaultMessage: 'All',
	},
	on: {
		id: 'app.settings.synced-options.game-settings.filter.on',
		defaultMessage: 'On',
	},
	off: {
		id: 'app.settings.synced-options.game-settings.filter.off',
		defaultMessage: 'Off',
	},
	enableAll: {
		id: 'app.settings.synced-options.game-settings.enable-all',
		defaultMessage: 'Enable all',
	},
	enableAllCount: {
		id: 'app.settings.synced-options.game-settings.enable-all-count',
		defaultMessage: 'Enable all ({count})',
	},
	disableAll: {
		id: 'app.settings.synced-options.game-settings.disable-all',
		defaultMessage: 'Disable all',
	},
	disableAllCount: {
		id: 'app.settings.synced-options.game-settings.disable-all-count',
		defaultMessage: 'Disable all ({count})',
	},
	customEmpty: {
		id: 'app.settings.synced-options.game-settings.custom-empty',
		defaultMessage:
			'Mod-added options will appear here after they are discovered in a participating instance.',
	},
	empty: {
		id: 'app.settings.synced-options.game-settings.empty',
		defaultMessage: 'No settings match these filters.',
	},
	loading: {
		id: 'app.settings.synced-options.game-settings.loading',
		defaultMessage: 'Loading game settings...',
	},
	loadFailed: {
		id: 'app.settings.synced-options.game-settings.load-failed',
		defaultMessage: 'Game settings could not be loaded.',
	},
	retry: {
		id: 'app.settings.synced-options.game-settings.retry',
		defaultMessage: 'Retry',
	},
	saveSettings: {
		id: 'app.settings.synced-options.game-settings.save',
		defaultMessage: 'Save settings',
	},
	saved: {
		id: 'app.settings.synced-options.game-settings.saved',
		defaultMessage: 'Game settings saved',
	},
	savedSummary: {
		id: 'app.settings.synced-options.game-settings.saved-summary',
		defaultMessage:
			'{applied, plural, one {# setting applied} other {# settings applied}}, {migrated, plural, one {# migrated} other {# migrated}}, {deferred, plural, one {# deferred} other {# deferred}}, {unsupported, plural, one {# unsupported} other {# unsupported}}, {failed, plural, one {# failed} other {# failed}}.',
	},
	previewFailed: {
		id: 'app.settings.synced-options.game-settings.preview-failed',
		defaultMessage: 'Compatibility could not be refreshed. Review the values or try again.',
	},
	conflictTitle: {
		id: 'app.settings.synced-options.game-settings.conflict-title',
		defaultMessage: 'Game settings changed elsewhere',
	},
	conflictText: {
		id: 'app.settings.synced-options.game-settings.conflict-text',
		defaultMessage:
			'The latest values have been loaded. Review the conflicting settings before saving again.',
	},
})

type SelectionFilter = 'all' | 'on' | 'off'

const modal = ref<InstanceType<typeof NewModal> | null>(null)
const confirmLeaveModal = ref<InstanceType<typeof ConfirmLeaveModal> | null>(null)
const baseState = ref<GameSettingsEditorState | null>(null)
const draftState = ref<GameSettingsEditorState | null>(null)
const activeCategoryId = ref('')
const search = ref('')
const selectionFilter = ref<SelectionFilter>('all')
const loading = ref(false)
const saving = ref(false)
const previewing = ref(false)
const loadError = ref(false)
const previewError = ref(false)
const touchedValueOptionIds = ref<Set<string>>(new Set())
let allowClose = false
let previewTimer: ReturnType<typeof setTimeout> | null = null
let previewGeneration = 0
let loadGeneration = 0

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

const filterTabs = computed<TabsTab[]>(() => [
	{ value: 'all', label: formatMessage(messages.all) },
	{ value: 'on', label: formatMessage(messages.on) },
	{ value: 'off', label: formatMessage(messages.off) },
])

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
		if (selectionFilter.value === 'on' && !setting.sync_enabled) return false
		if (selectionFilter.value === 'off' && setting.sync_enabled) return false
		return true
	})
})

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
				setting.sync_enabled &&
				(!!setting.validation_error || ['mixed', 'unset', 'invalid'].includes(setting.value_state)),
		) ?? false,
)
const enableCandidates = computed(() =>
	categorySettings.value.filter((setting) => !setting.sync_enabled && settingCanBeEnabled(setting)),
)
const disableCandidates = computed(() =>
	categorySettings.value.filter((setting) => setting.sync_enabled && !setting.controlled),
)

function categoryIcon(category: GameSettingCategory): Component {
	return categoryIcons[category.id] ?? SettingsIcon
}

function setSelectionFilter(value: string | number) {
	selectionFilter.value = value as SelectionFilter
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
	previewError.value = false
	try {
		const state = await get_synced_game_options_config()
		if (generation !== loadGeneration) return
		baseState.value = cloneGameSettingsState(state)
		draftState.value = cloneGameSettingsState(state)
		touchedValueOptionIds.value = new Set()
		const currentCategoryExists = categories.value.some(
			(category) => category.id === activeCategoryId.value,
		)
		if (!currentCategoryExists) activeCategoryId.value = categories.value[0]?.id ?? 'custom_settings'
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
	selectionFilter.value = 'all'
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
	baseState.value = null
	draftState.value = null
	loading.value = false
	saving.value = false
	previewing.value = false
	loadError.value = false
	previewError.value = false
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

function settingById(optionId: string): EditableGameSetting | undefined {
	return draftState.value?.settings.find((setting) => setting.option_id === optionId)
}

function setSyncEnabled(optionId: string, enabled: boolean) {
	const setting = settingById(optionId)
	if (!setting) return
	setting.sync_enabled = enabled
	setting.validation_error = null
	schedulePreview()
}

function setCanonicalValue(optionId: string, value: GameOptionCanonicalValue | null) {
	const setting = settingById(optionId)
	if (!setting) return
	touchedValueOptionIds.value = new Set(touchedValueOptionIds.value).add(optionId)
	setting.canonical_value = value
	setting.value_state = value ? 'canonical' : 'unset'
	setting.validation_error = null
	schedulePreview()
}

function enableVisible() {
	if (saving.value) return
	for (const setting of enableCandidates.value) setting.sync_enabled = true
	schedulePreview()
}

function disableVisible() {
	if (saving.value) return
	for (const setting of disableCandidates.value) setting.sync_enabled = false
	schedulePreview()
}

function schedulePreview() {
	previewError.value = false
	previewGeneration++
	previewing.value = true
	if (previewTimer) clearTimeout(previewTimer)
	previewTimer = setTimeout(() => void preview(), 350)
}

function mergePreview(preview: GameSettingsEditorState) {
	if (!baseState.value || !draftState.value) return
	const previousBase = baseState.value
	const dirtyIds = new Set(
		gameSettingChanges(
			previousBase,
			draftState.value,
			touchedValueOptionIds.value,
		).map((change) => change.option_id),
	)
	const stagedSettings = new Map(
		draftState.value.settings
			.filter((setting) => dirtyIds.has(setting.option_id))
			.map((setting) => [setting.option_id, setting]),
	)
	const previousBaseSettings = new Map(
		previousBase.settings.map((setting) => [setting.option_id, setting]),
	)
	baseState.value = {
		...cloneGameSettingsState(preview),
		settings: preview.settings.map((setting) =>
			dirtyIds.has(setting.option_id)
				? (previousBaseSettings.get(setting.option_id) ?? setting)
				: setting,
		),
	}
	draftState.value = {
		...preview,
		settings: preview.settings.map((setting) => {
			const staged = stagedSettings.get(setting.option_id)
			if (!staged) return setting
			return {
				...setting,
				sync_enabled: staged.sync_enabled,
				canonical_value: staged.canonical_value,
				option_revision: staged.option_revision,
			}
		}),
	}
}

async function preview() {
	const request = editorRequest()
	if (!request || request.changes.length === 0) {
		previewGeneration++
		previewing.value = false
		if (baseState.value) draftState.value = cloneGameSettingsState(baseState.value)
		return
	}

	const generation = ++previewGeneration
	previewing.value = true
	try {
		const previewState = await preview_synced_game_option_changes(request)
		if (generation !== previewGeneration) return
		mergePreview(previewState)
		previewError.value = false
	} catch {
		if (generation === previewGeneration) previewError.value = true
	} finally {
		if (generation === previewGeneration) previewing.value = false
	}
}

async function save() {
	const request = editorRequest()
	if (!request || request.changes.length === 0) return

	if (previewTimer) clearTimeout(previewTimer)
	previewTimer = null
	previewGeneration++
	saving.value = true
	try {
		const result = await save_synced_game_option_changes(request)
		if (result.conflicts?.length) {
			const refreshed = result.state ?? (await get_synced_game_options_config())
			const previousDraft = draftState.value
			const stagedChanges = new Map(request.changes.map((change) => [change.option_id, change]))
			const conflicts = new Set(result.conflicts)
			baseState.value = cloneGameSettingsState(refreshed)
			draftState.value = cloneGameSettingsState(refreshed)

			for (const setting of draftState.value.settings) {
				const previous = previousDraft?.settings.find(
					(candidate) => candidate.option_id === setting.option_id,
				)
				if (conflicts.has(setting.option_id)) {
					setting.validation_error = 'changed_since_opened'
					continue
				}
				const change = stagedChanges.get(setting.option_id)
				if (!change || !previous) continue
				if (change.sync_enabled !== undefined) setting.sync_enabled = previous.sync_enabled
				if (change.canonical_value !== undefined) {
					setting.canonical_value = previous.canonical_value
					setting.value_state = previous.value_state
				}
			}

			addNotification({
				type: 'warning',
				title: formatMessage(messages.conflictTitle),
				text: formatMessage(messages.conflictText),
			})
			if (
				gameSettingChanges(
					baseState.value,
					draftState.value,
					touchedValueOptionIds.value,
				).length > 0
			) {
				schedulePreview()
			}
			return
		}

		addNotification({
			type: result.failed > 0 ? 'warning' : 'success',
			title: formatMessage(messages.saved),
			text: formatMessage(messages.savedSummary, {
				applied: result.applied,
				migrated: result.migrated,
				deferred: result.deferred,
				unsupported: result.unsupported,
				failed: result.failed,
			}),
		})
		emit('saved')
		allowClose = true
		modal.value?.hide()
	} catch (error) {
		handleError(error)
	} finally {
		saving.value = false
	}
}

onUnmounted(() => {
	if (previewTimer) clearTimeout(previewTimer)
})

defineExpose({ show, hide })
</script>

<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.title)"
		:no-padding="true"
		:actions-divider="true"
		:before-hide="beforeHide"
		:on-after-hide="reset"
		:disable-close="saving"
		max-width="930px"
		width="930px"
	>
		<div class="grid h-[550px] min-h-0 grid-cols-[272px_minmax(0,1fr)] bg-surface-2">
			<aside class="flex min-h-0 flex-col gap-2 border-0 border-r border-solid border-surface-5 bg-surface-3 p-4">
				<Input
					v-model="search"
					:icon="SearchIcon"
					type="search"
					autocomplete="off"
					:placeholder="formatMessage(messages.search)"
					wrapper-class="w-full shrink-0"
				/>

				<nav class="flex min-h-0 flex-col overflow-y-auto" :aria-label="formatMessage(messages.title)">
					<button
						v-for="category in categories"
						:key="category.id"
						type="button"
						class="flex cursor-pointer items-center gap-2 rounded-[14px] border-0 px-4 py-2.5 text-left font-semibold outline-none transition-colors focus-visible:ring-4 focus-visible:ring-brand-shadow"
						:class="
							category.id === activeCategoryId
								? 'bg-highlight-green text-green'
								: 'bg-transparent text-primary hover:bg-surface-4'
						"
						@click="activeCategoryId = category.id; search = ''"
					>
						<component :is="categoryIcon(category)" class="size-5 shrink-0" />
						<span class="truncate">{{ formatGameSettingCategory(formatMessage, category) }}</span>
					</button>
				</nav>
			</aside>

			<main class="flex min-h-0 min-w-0 flex-col bg-surface-2">
				<div class="flex shrink-0 flex-wrap items-center justify-between gap-3 p-6">
					<div class="flex items-center gap-2">
						<Tabs
							:value="selectionFilter"
							:tabs="filterTabs"
							@update:value="setSelectionFilter"
						/>
						<SpinnerIcon v-if="previewing" class="size-5 animate-spin text-secondary" />
					</div>
					<ButtonGroup>
						<Button
							size="sm"
							:disabled="saving || enableCandidates.length === 0"
							@click="enableVisible"
						>
							{{
								enableCandidates.length
									? formatMessage(messages.enableAllCount, { count: enableCandidates.length })
									: formatMessage(messages.enableAll)
							}}
						</Button>
						<Button
							size="sm"
							:disabled="saving || disableCandidates.length === 0"
							@click="disableVisible"
						>
							{{
								disableCandidates.length
									? formatMessage(messages.disableAllCount, { count: disableCandidates.length })
									: formatMessage(messages.disableAll)
							}}
						</Button>
					</ButtonGroup>
				</div>

				<div v-if="loading" class="flex flex-1 items-center justify-center gap-2 text-secondary">
					<SpinnerIcon class="size-5 animate-spin" />
					{{ formatMessage(messages.loading) }}
				</div>
				<div v-else-if="loadError" class="flex flex-1 flex-col items-center justify-center gap-3 text-secondary">
					<p class="m-0">{{ formatMessage(messages.loadFailed) }}</p>
					<Button @click="load">
						<RefreshCwIcon />
						{{ formatMessage(messages.retry) }}
					</Button>
				</div>
				<div v-else class="min-h-0 flex-1 overflow-y-auto px-6 py-2">
					<p v-if="previewError" class="m-0 border-0 border-b border-solid border-surface-5 py-3 text-sm text-orange">
						{{ formatMessage(messages.previewFailed) }}
					</p>
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
					<GameSettingRow
						v-for="setting in categorySettings"
						:key="setting.option_id"
						:setting="setting"
						:disabled="saving"
						@update:sync-enabled="setSyncEnabled(setting.option_id, $event)"
						@update:canonical-value="setCanonicalValue(setting.option_id, $event)"
					/>
				</div>
			</main>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2 p-2">
				<Button type="outlined" :disabled="saving" @click="modal?.hide()">
					<XIcon />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button
					type="colored"
					color="brand"
					:disabled="
						!isDirty || loading || previewing || saving || previewError || hasBlockingDraft
					"
					:loading="saving"
					@click="save"
				>
					<SaveIcon v-if="!saving" />
					{{ formatMessage(messages.saveSettings) }}
				</Button>
			</div>
		</template>
	</NewModal>

	<ConfirmLeaveModal ref="confirmLeaveModal" />
</template>
