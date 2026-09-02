<script setup lang="ts">
import {
	defineMessages,
	injectAuth,
	injectUserPreferences,
	Slider,
	Toggle,
	useSavable,
	useVIntl,
} from '@modrinth/ui'
import { inject, onBeforeUnmount, onMounted, ref } from 'vue'

import {
	DEFAULT_FEATURE_FLAGS,
	type FeatureFlag,
	useAppSettings,
} from '@/composables/use-app-settings.ts'
import {
	QUICK_INSTANCE_LIMIT_MAX,
	useQuickInstanceLimit,
} from '@/composables/use-quick-instance-limit.ts'
import { type AppSettings, get, set } from '@/helpers/settings.ts'
import { appSettingsModalContextKey } from '@/providers/app-settings-modal'

const appSettings = useAppSettings()
const { formatMessage } = useVIntl()
const auth = injectAuth()
const { updatePreferences } = injectUserPreferences()
const settingsModal = inject(appSettingsModalContextKey, null)
const quickInstances = useQuickInstanceLimit()

const showJumpInFlag: FeatureFlag = 'worlds_in_home'

const messages = defineMessages({
	syncAcrossDevicesTitle: {
		id: 'app.features-settings.sync-across-devices.title',
		defaultMessage: 'Sync features across devices',
	},
	syncAcrossDevicesDescription: {
		id: 'app.features-settings.sync-across-devices.description',
		defaultMessage:
			"Use these feature settings everywhere you're signed in. Turn this off to keep separate settings on this device.",
	},
	syncAcrossDevicesSignedOutTooltip: {
		id: 'app.features-settings.sync-across-devices.signed-out-tooltip',
		defaultMessage: 'Sign into a Modrinth account to sync settings.',
	},
	instancePagesTitle: {
		id: 'app.features-settings.instance-pages.title',
		defaultMessage: 'Instance pages',
	},
	showWorldsTabTitle: {
		id: 'app.features-settings.show-worlds-tab.title',
		defaultMessage: 'Show Worlds tab in instances',
	},
	showWorldsTabDescription: {
		id: 'app.features-settings.show-worlds-tab.description',
		defaultMessage: 'Browse and launch worlds from each instance.',
	},
	showFilesTabTitle: {
		id: 'app.features-settings.show-files-tab.title',
		defaultMessage: 'Show Files tab in instances',
	},
	showFilesTabDescription: {
		id: 'app.features-settings.show-files-tab.description',
		defaultMessage: 'Browse the files in each instance from its navigation.',
	},
	showScreenshotsTabTitle: {
		id: 'app.features-settings.show-screenshots-tab.title',
		defaultMessage: 'Show Screenshots tab in instances',
	},
	showScreenshotsTabDescription: {
		id: 'app.features-settings.show-screenshots-tab.description',
		defaultMessage: 'Browse screenshots from the instance where they were taken.',
	},
	sidebarTitle: {
		id: 'app.features-settings.sidebar.title',
		defaultMessage: 'Sidebar',
	},
	showSkinSelectorTitle: {
		id: 'app.features-settings.show-skin-selector.title',
		defaultMessage: 'Show skin selector in sidebar',
	},
	showSkinSelectorDescription: {
		id: 'app.features-settings.show-skin-selector.description',
		defaultMessage: 'Add a shortcut to the skin selector to the left sidebar.',
	},
	quickInstancesTitle: {
		id: 'app.features-settings.quick-instances.title',
		defaultMessage: 'Quick instances in sidebar',
	},
	quickInstancesDescription: {
		id: 'app.features-settings.quick-instances.description',
		defaultMessage:
			'Choose the maximum number of recent instances shown in the left sidebar. You can also adjust this by dragging the divider.',
	},
	playPageTitle: {
		id: 'app.features-settings.play-page.title',
		defaultMessage: 'Play page',
	},
	showJumpInTitle: {
		id: 'app.features-settings.show-jump-in.title',
		defaultMessage: 'Show Jump in section',
	},
	showJumpInDescription: {
		id: 'app.features-settings.show-jump-in.description',
		defaultMessage: 'Show recently played worlds and instances at the top of the Play page.',
	},
})

type FeaturesSettingsState = {
	syncFeaturesAcrossDevices: boolean
	showFilesTab: boolean
	showWorldsTab: boolean
	showScreenshotsTab: boolean
	showSkinSelector: boolean
	quickInstanceCount: number
	showJumpIn: boolean
}

const persistedSettings = ref(await get())

function getFeaturesSettingsState(settings: AppSettings): FeaturesSettingsState {
	return {
		syncFeaturesAcrossDevices: settings.sync_features_across_devices,
		showFilesTab: settings.show_files_tab_in_instances,
		showWorldsTab: settings.show_worlds_tab_in_instances,
		showScreenshotsTab: settings.show_screenshots_tab_in_instances,
		showSkinSelector: settings.show_skin_selector_in_sidebar,
		quickInstanceCount: Math.min(
			quickInstances.limit.value ?? QUICK_INSTANCE_LIMIT_MAX,
			QUICK_INSTANCE_LIMIT_MAX,
		),
		showJumpIn: settings.feature_flags[showJumpInFlag] ?? DEFAULT_FEATURE_FLAGS[showJumpInFlag],
	}
}

const { saved, current, changes, saving, hasChanges, reset, save } = useSavable(
	() => getFeaturesSettingsState(persistedSettings.value),
	async (changedValues) => {
		const value = current.value

		if (value.syncFeaturesAcrossDevices && auth.user.value) {
			await updatePreferences({
				behavior: {
					show_jump_in: value.showJumpIn,
					show_files_tab_in_instances: value.showFilesTab,
					show_worlds_tab_in_instances: value.showWorldsTab,
					show_screenshots_tab_in_instances: value.showScreenshotsTab,
					show_skin_selector_in_sidebar: value.showSkinSelector,
					quick_instance_count: value.quickInstanceCount,
				},
			})
		}

		const nextSettings: AppSettings = {
			...persistedSettings.value,
			sync_features_across_devices: value.syncFeaturesAcrossDevices,
			show_files_tab_in_instances: value.showFilesTab,
			show_worlds_tab_in_instances: value.showWorldsTab,
			show_screenshots_tab_in_instances: value.showScreenshotsTab,
			show_skin_selector_in_sidebar: value.showSkinSelector,
			feature_flags: {
				...persistedSettings.value.feature_flags,
				[showJumpInFlag]: value.showJumpIn,
			},
		}

		await set(nextSettings)
		persistedSettings.value = nextSettings
		appSettings.setFeaturesSyncAcrossDevices(value.syncFeaturesAcrossDevices)
		appSettings.showFilesTabInInstances = value.showFilesTab
		appSettings.showWorldsTabInInstances = value.showWorldsTab
		appSettings.showScreenshotsTabInInstances = value.showScreenshotsTab
		appSettings.showSkinSelectorInSidebar = value.showSkinSelector
		appSettings.featureFlags[showJumpInFlag] = value.showJumpIn

		if (changedValues.quickInstanceCount !== undefined) {
			quickInstances.setLimit(
				value.quickInstanceCount >= QUICK_INSTANCE_LIMIT_MAX ? null : value.quickInstanceCount,
			)
		}
	},
)

async function saveFeaturesSettings(): Promise<void> {
	try {
		await save()
	} catch {
		return
	}
}

onMounted(() => {
	settingsModal?.registerUnsavedChangesController({
		hasChanges: () => hasChanges.value,
		getOriginal: () => saved.value,
		getModified: () => changes.value,
		isSaving: () => saving.value,
		reset,
		save: saveFeaturesSettings,
	})
})

onBeforeUnmount(() => {
	settingsModal?.registerUnsavedChangesController(null)
})
</script>

<template>
	<section class="border-0 border-b border-solid border-divider pb-6">
		<div class="flex items-center justify-between gap-4">
			<div>
				<h2 id="sync-features-across-devices-label" class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.syncAcrossDevicesTitle) }}
				</h2>
				<p class="m-0 mt-1 text-secondary">
					{{ formatMessage(messages.syncAcrossDevicesDescription) }}
				</p>
			</div>
			<span
				v-tooltip="
					!auth.user.value ? formatMessage(messages.syncAcrossDevicesSignedOutTooltip) : undefined
				"
				class="inline-flex shrink-0"
			>
				<Toggle
					id="sync-features-across-devices"
					:model-value="Boolean(auth.user.value) && current.syncFeaturesAcrossDevices"
					:disabled="!auth.user.value"
					aria-labelledby="sync-features-across-devices-label"
					@update:model-value="current.syncFeaturesAcrossDevices = $event"
				/>
			</span>
		</div>
	</section>

	<section class="mt-6">
		<h2 class="m-0 text-xl font-semibold text-contrast">
			{{ formatMessage(messages.instancePagesTitle) }}
		</h2>
		<div class="mt-4 flex flex-col gap-6">
			<div class="flex items-center justify-between gap-4">
				<div>
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.showWorldsTabTitle) }}
					</h3>
					<p class="m-0 mt-1">{{ formatMessage(messages.showWorldsTabDescription) }}</p>
				</div>
				<Toggle id="show-worlds-tab-in-instances" v-model="current.showWorldsTab" />
			</div>

			<div class="flex items-center justify-between gap-4">
				<div>
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.showFilesTabTitle) }}
					</h3>
					<p class="m-0 mt-1">{{ formatMessage(messages.showFilesTabDescription) }}</p>
				</div>
				<Toggle id="show-files-tab-in-instances" v-model="current.showFilesTab" />
			</div>

			<div class="flex items-center justify-between gap-4">
				<div>
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.showScreenshotsTabTitle) }}
					</h3>
					<p class="m-0 mt-1">
						{{ formatMessage(messages.showScreenshotsTabDescription) }}
					</p>
				</div>
				<Toggle id="show-screenshots-tab-in-instances" v-model="current.showScreenshotsTab" />
			</div>
		</div>
	</section>

	<section class="mt-8 border-0 border-t border-solid border-divider pt-6">
		<h2 class="m-0 text-xl font-semibold text-contrast">
			{{ formatMessage(messages.sidebarTitle) }}
		</h2>
		<div class="mt-4 flex flex-col gap-6">
			<div class="flex items-center justify-between gap-4">
				<div>
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.showSkinSelectorTitle) }}
					</h3>
					<p class="m-0 mt-1">{{ formatMessage(messages.showSkinSelectorDescription) }}</p>
				</div>
				<Toggle id="show-skin-selector-in-sidebar" v-model="current.showSkinSelector" />
			</div>

			<div class="flex flex-col gap-2.5">
				<h3 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.quickInstancesTitle) }}
				</h3>
				<Slider
					id="quick-instances-in-sidebar"
					v-model="current.quickInstanceCount"
					:min="0"
					:max="QUICK_INSTANCE_LIMIT_MAX"
					:step="1"
				/>
				<p class="m-0 leading-tight">
					{{ formatMessage(messages.quickInstancesDescription) }}
				</p>
			</div>
		</div>
	</section>

	<section class="mt-8 border-0 border-t border-solid border-divider pt-6">
		<h2 class="m-0 text-xl font-semibold text-contrast">
			{{ formatMessage(messages.playPageTitle) }}
		</h2>
		<div class="mt-4 flex items-center justify-between gap-4">
			<div>
				<h3 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.showJumpInTitle) }}
				</h3>
				<p class="m-0 mt-1">{{ formatMessage(messages.showJumpInDescription) }}</p>
			</div>
			<Toggle id="show-jump-in-section" v-model="current.showJumpIn" />
		</div>
	</section>
</template>
