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
	Input,
	TabbedModal,
	type TabbedModalTab,
	useVIntl,
} from '@modrinth/ui'
import type { Component } from 'vue'
import { computed, ref } from 'vue'

import type { EditableGameSetting, GameSettingCategory } from '@/helpers/game-options'

import { settingCanBeEnabled, settingSearchText } from './editors'
import { minecraftKeybindConflictKey } from './keybinds'
import {
	formatGameSettingDescription,
	formatGameSettingLabel,
	gameSettingCategoryMessage,
} from './messages'
import GameSettingRow from './row.vue'
import { useGameSettingsEditor } from './use-editor'

const props = defineProps<{
	instanceId?: string
}>()

const emit = defineEmits<{
	saved: []
}>()

const { formatMessage } = useVIntl()

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
		id: 'app.settings.synced-options.game-settings.sync-description',
		defaultMessage:
			'Synced options are applied to every instance and override the values included with a modpack.',
	},
	search: {
		id: 'app.settings.synced-options.game-settings.search',
		defaultMessage: 'Search settings...',
	},
	enableAll: {
		id: 'app.settings.synced-options.game-settings.enable-all',
		defaultMessage: 'Sync all',
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
})

const modal = ref<InstanceType<typeof TabbedModal> | null>(null)
const confirmLeaveModal = ref<InstanceType<typeof ConfirmLeaveModal> | null>(null)
const activeCategoryId = ref('')
const search = ref('')
let allowClose = false

const {
	draftState,
	isLocalEditor,
	isDirty,
	hasBlockingDraft,
	loading,
	loadError,
	saving,
	load: loadSettings,
	reset: resetEditor,
	cancelChanges,
	setSyncEnabled,
	setCanonicalValue,
	save,
} = useGameSettingsEditor(
	() => props.instanceId,
	() => emit('saved'),
)

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
		const key = minecraftKeybindConflictKey(setting.option_id, setting.canonical_value.value)
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

async function load() {
	if (!(await loadSettings())) return
	if (!categories.value.some((category) => category.id === activeCategoryId.value)) {
		activeCategoryId.value = categories.value[0]?.id ?? 'custom_settings'
	}
	const index = categories.value.findIndex((category) => category.id === activeCategoryId.value)
	if (index >= 0) modal.value?.setTab(index)
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
	resetEditor()
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

function toggleVisibleSync() {
	const enabled = !allCategorySettingsSynced.value
	const candidates = enabled ? enableCandidates.value : disableCandidates.value
	setSyncEnabled(
		candidates.map((setting) => setting.option_id),
		enabled,
	)
}

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
		max-width="min(1080px, calc(95vw - 2rem))"
		width="min(1080px, calc(95vw - 2rem))"
	>
		<template #sidebar-header>
			<div class="pb-4">
				<Input
					v-model="search"
					:icon="SearchIcon"
					type="search"
					autocomplete="off"
					:placeholder="formatMessage(messages.search)"
					:aria-label="formatMessage(messages.search)"
					wrapper-class="w-full shrink-0"
				/>
			</div>
		</template>

		<template #content>
			<div class="flex min-h-full min-w-0 flex-col">
				<div
					v-if="!isLocalEditor && (loading || loadError || categorySettings.length > 0)"
					class="mb-4 flex shrink-0 flex-wrap items-start justify-between gap-4 border-0 border-b border-solid border-surface-4 pb-4"
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
							<LinkIcon v-if="allCategorySettingsSynced" aria-hidden="true" />
							<UnlinkIcon v-else aria-hidden="true" />
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
					<SpinnerIcon class="size-5 animate-spin" aria-hidden="true" />
					{{ formatMessage(messages.loading) }}
				</div>
				<div
					v-else-if="loadError"
					class="flex min-h-40 flex-1 flex-col items-center justify-center gap-3 text-secondary"
				>
					<p class="m-0">{{ formatMessage(messages.loadFailed) }}</p>
					<Button @click="load">
						<RefreshCwIcon aria-hidden="true" />
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
							@update:sync-enabled="setSyncEnabled([setting.option_id], $event)"
							@update:canonical-value="setCanonicalValue(setting.option_id, $event)"
						/>
					</div>
				</div>
			</div>
		</template>

		<template #floating-action-bar>
			<FloatingActionBar v-if="isDirty" shown inline :aria-label="modalTitle">
				<p class="m-0 text-sm font-semibold md:text-base">
					{{ formatMessage(messages.unsavedChanges) }}
				</p>
				<div class="ml-auto flex gap-2">
					<Button type="outlined" @click="cancelChanges">
						<XIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.cancelButton) }}
					</Button>
					<Button
						type="colored"
						color="brand"
						:disabled="!isDirty || loading || saving || hasBlockingDraft"
						:loading="saving"
						@click="save"
					>
						<SaveIcon aria-hidden="true" />
						{{ formatMessage(messages.saveSettings) }}
					</Button>
				</div>
			</FloatingActionBar>
		</template>
	</TabbedModal>

	<ConfirmLeaveModal ref="confirmLeaveModal" />
</template>
