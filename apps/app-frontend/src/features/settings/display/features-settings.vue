<script setup lang="ts">
import {
	defineMessages,
	injectAuth,
	injectNotificationManager,
	injectUserPreferences,
	Slider,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { useDebounceFn } from '@vueuse/core'
import { computed, onBeforeUnmount, ref } from 'vue'

import { useAppSettings } from '@/composables/use-app-settings.ts'
import {
	QUICK_INSTANCE_LIMIT_MAX,
	useQuickInstanceLimit,
} from '@/composables/use-quick-instance-limit.ts'
import { type GlobalSyncedOptions, set_global_synced_option } from '@/helpers/instance.ts'
import {
	type AppSettings,
	appSettingsKeys,
	appSettingsQueryOptions,
	get,
	set,
} from '@/helpers/settings.ts'
import { globalSyncedOptionsQueryOptions, syncedOptionsKeys } from '@/helpers/synced-options'
import { screenshotKeys } from '@/pages/instance/query-options.ts'

const appSettings = useAppSettings()
const { formatMessage } = useVIntl()
const auth = injectAuth()
const { handleError } = injectNotificationManager()
const { updatePreferences } = injectUserPreferences()
const quickInstances = useQuickInstanceLimit()
const queryClient = useQueryClient()

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
		defaultMessage: 'Instances',
	},
	showWorldsTabTitle: {
		id: 'app.features-settings.show-worlds-tab.title',
		defaultMessage: 'Show Worlds tab in instances',
	},
	showFilesTabTitle: {
		id: 'app.features-settings.show-files-tab.title',
		defaultMessage: 'Show Files tab in instances',
	},
	showScreenshotsTabTitle: {
		id: 'app.features-settings.show-screenshots-tab.title',
		defaultMessage: 'Show Screenshots tab in instances',
	},
	sidebarTitle: {
		id: 'app.features-settings.sidebar.title',
		defaultMessage: 'Sidebar',
	},
	showAllScreenshotsTitle: {
		id: 'app.features-settings.show-all-screenshots.title',
		defaultMessage: 'Show all screenshots in sidebar',
	},
	showAllScreenshotsDescription: {
		id: 'app.features-settings.show-all-screenshots.description',
		defaultMessage:
			'Show a button in the left sidebar to view screenshots from all your instances.',
	},
	showSkinSelectorTitle: {
		id: 'app.features-settings.show-skin-selector.title',
		defaultMessage: 'Show skin selector in sidebar',
	},
	showSkinSelectorDescription: {
		id: 'app.features-settings.show-skin-selector.description',
		defaultMessage: 'Show a button in the left sidebar to open the skin selector.',
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

type FeatureSettingKey =
	| 'sync_features_across_devices'
	| 'show_files_tab_in_instances'
	| 'show_worlds_tab_in_instances'
	| 'show_screenshots_tab_in_instances'
	| 'show_skin_selector_in_sidebar'
	| 'show_jump_in'

type FeaturesSettingsPatch = {
	settings?: Partial<Pick<AppSettings, FeatureSettingKey>>
	showAllScreenshots?: boolean
	quickInstanceCount?: number
}

const settingsQuery = useQuery(appSettingsQueryOptions())
const globalOptionsQuery = useQuery(globalSyncedOptionsQueryOptions())
await Promise.all([settingsQuery.suspense(), globalOptionsQuery.suspense()])

const settings = computed(() => settingsQuery.data.value!)
const showAllScreenshots = computed(() => globalOptionsQuery.data.value!.screenshots)

const pendingQuickInstanceCount = ref<number | null>(null)
const quickInstanceCount = computed(
	() =>
		pendingQuickInstanceCount.value ?? quickInstances.limit.value ?? QUICK_INSTANCE_LIMIT_MAX,
)

function applyFeatureSettings(value: AppSettings): void {
	appSettings.setFeaturesSyncAcrossDevices(value.sync_features_across_devices)
	appSettings.showFilesTabInInstances = value.show_files_tab_in_instances
	appSettings.showWorldsTabInInstances = value.show_worlds_tab_in_instances
	appSettings.showScreenshotsTabInInstances = value.show_screenshots_tab_in_instances
	appSettings.showSkinSelectorInSidebar = value.show_skin_selector_in_sidebar
	appSettings.showJumpIn = value.show_jump_in
}

const settingsMutation = useMutation({
	mutationKey: appSettingsKeys.update,
	scope: { id: 'app-settings' },
	mutationFn: async (patch: FeaturesSettingsPatch) => {
		const nextSettings: AppSettings = { ...(await get()), ...patch.settings }

		if (nextSettings.sync_features_across_devices && auth.user.value) {
			await updatePreferences({
				behavior: {
					show_jump_in: nextSettings.show_jump_in,
					show_files_tab_in_instances: nextSettings.show_files_tab_in_instances,
					show_worlds_tab_in_instances: nextSettings.show_worlds_tab_in_instances,
					show_screenshots_tab_in_instances: nextSettings.show_screenshots_tab_in_instances,
					show_all_screenshots: patch.showAllScreenshots ?? showAllScreenshots.value,
					show_skin_selector_in_sidebar: nextSettings.show_skin_selector_in_sidebar,
					quick_instance_count:
						patch.quickInstanceCount ?? quickInstances.limit.value ?? QUICK_INSTANCE_LIMIT_MAX,
				},
			})
		}

		await Promise.all([
			patch.settings ? set(nextSettings) : undefined,
			patch.showAllScreenshots !== undefined
				? set_global_synced_option('screenshots', patch.showAllScreenshots)
				: undefined,
		])
		if (patch.showAllScreenshots !== undefined) {
			await queryClient.invalidateQueries({ queryKey: screenshotKeys.all })
		}
	},
	onMutate: async (patch) => {
		const previousQuickInstanceLimit = quickInstances.limit.value
		if (patch.quickInstanceCount !== undefined) {
			quickInstances.setLimit(patch.quickInstanceCount)
		}

		await Promise.all([
			queryClient.cancelQueries({ queryKey: appSettingsKeys.all }),
			queryClient.cancelQueries({ queryKey: syncedOptionsKeys.global }),
		])
		const previousSettings = queryClient.getQueryData<AppSettings>(appSettingsKeys.all)
		const previousGlobalOptions = queryClient.getQueryData<GlobalSyncedOptions>(
			syncedOptionsKeys.global,
		)

		if (previousSettings && patch.settings) {
			const nextSettings = { ...previousSettings, ...patch.settings }
			queryClient.setQueryData(appSettingsKeys.all, nextSettings)
			applyFeatureSettings(nextSettings)
		}
		if (previousGlobalOptions && patch.showAllScreenshots !== undefined) {
			queryClient.setQueryData(syncedOptionsKeys.global, {
				...previousGlobalOptions,
				screenshots: patch.showAllScreenshots,
			})
		}

		return { previousSettings, previousGlobalOptions, previousQuickInstanceLimit }
	},
	onError: (error, patch, context) => {
		if (context?.previousSettings && patch.settings) {
			queryClient.setQueryData(appSettingsKeys.all, context.previousSettings)
			applyFeatureSettings(context.previousSettings)
		}
		if (context?.previousGlobalOptions && patch.showAllScreenshots !== undefined) {
			queryClient.setQueryData(syncedOptionsKeys.global, context.previousGlobalOptions)
		}
		if (context && patch.quickInstanceCount !== undefined) {
			quickInstances.setLimit(context.previousQuickInstanceLimit)
		}
		handleError(error)
	},
	onSettled: async () => {
		if (queryClient.isMutating({ mutationKey: appSettingsKeys.update }) === 1) {
			await Promise.all([
				queryClient.invalidateQueries({ queryKey: appSettingsKeys.all }),
				queryClient.invalidateQueries({ queryKey: syncedOptionsKeys.global }),
			])
		}
	},
})

function updateSetting<K extends FeatureSettingKey>(key: K, value: AppSettings[K]): void {
	settingsMutation.mutate({ settings: { [key]: value } as Pick<AppSettings, K> })
}

function updateShowAllScreenshots(enabled: boolean): void {
	settingsMutation.mutate({ showAllScreenshots: enabled })
}

function flushQuickInstanceCount(): void {
	const count = pendingQuickInstanceCount.value
	if (count === null) return
	pendingQuickInstanceCount.value = null
	settingsMutation.mutate({ quickInstanceCount: count })
}

const saveQuickInstanceCount = useDebounceFn(flushQuickInstanceCount, 500)

function updateQuickInstanceCount(count: number): void {
	pendingQuickInstanceCount.value = count
	saveQuickInstanceCount()
}

onBeforeUnmount(flushQuickInstanceCount)
</script>

<template>
	<section class="border-0 border-b border-solid border-surface-4 pb-6">
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
					:model-value="Boolean(auth.user.value) && settings.sync_features_across_devices"
					:disabled="!auth.user.value"
					aria-labelledby="sync-features-across-devices-label"
					@update:model-value="updateSetting('sync_features_across_devices', $event)"
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
				</div>
				<Toggle
					id="show-worlds-tab-in-instances"
					:model-value="settings.show_worlds_tab_in_instances"
					:aria-label="formatMessage(messages.showWorldsTabTitle)"
					@update:model-value="updateSetting('show_worlds_tab_in_instances', $event)"
				/>
			</div>

			<div class="flex items-center justify-between gap-4">
				<div>
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.showFilesTabTitle) }}
					</h3>
				</div>
				<Toggle
					id="show-files-tab-in-instances"
					:model-value="settings.show_files_tab_in_instances"
					:aria-label="formatMessage(messages.showFilesTabTitle)"
					@update:model-value="updateSetting('show_files_tab_in_instances', $event)"
				/>
			</div>

			<div class="flex items-center justify-between gap-4">
				<div>
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.showScreenshotsTabTitle) }}
					</h3>
				</div>
				<Toggle
					id="show-screenshots-tab-in-instances"
					:model-value="settings.show_screenshots_tab_in_instances"
					:aria-label="formatMessage(messages.showScreenshotsTabTitle)"
					@update:model-value="updateSetting('show_screenshots_tab_in_instances', $event)"
				/>
			</div>
		</div>
	</section>

	<section class="mt-8 border-0 border-t border-solid border-surface-4 pt-6">
		<h2 class="m-0 text-xl font-semibold text-contrast">
			{{ formatMessage(messages.sidebarTitle) }}
		</h2>
		<div class="mt-4 flex flex-col gap-6">
			<div class="flex items-center justify-between gap-4">
				<div>
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.showAllScreenshotsTitle) }}
					</h3>
					<p class="m-0 mt-1">
						{{ formatMessage(messages.showAllScreenshotsDescription) }}
					</p>
				</div>
				<Toggle
					id="show-all-screenshots"
					:model-value="showAllScreenshots"
					:aria-label="formatMessage(messages.showAllScreenshotsTitle)"
					@update:model-value="updateShowAllScreenshots"
				/>
			</div>

			<div class="flex items-center justify-between gap-4">
				<div>
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.showSkinSelectorTitle) }}
					</h3>
					<p class="m-0 mt-1">{{ formatMessage(messages.showSkinSelectorDescription) }}</p>
				</div>
				<Toggle
					id="show-skin-selector-in-sidebar"
					:model-value="settings.show_skin_selector_in_sidebar"
					:aria-label="formatMessage(messages.showSkinSelectorTitle)"
					@update:model-value="updateSetting('show_skin_selector_in_sidebar', $event)"
				/>
			</div>

			<div class="flex flex-col gap-2.5">
				<h3 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.quickInstancesTitle) }}
				</h3>
				<Slider
					id="quick-instances-in-sidebar"
					:model-value="quickInstanceCount"
					:aria-label="formatMessage(messages.quickInstancesTitle)"
					:min="0"
					:max="QUICK_INSTANCE_LIMIT_MAX"
					:step="1"
					@update:model-value="updateQuickInstanceCount"
				/>
				<p class="m-0 leading-tight">
					{{ formatMessage(messages.quickInstancesDescription) }}
				</p>
			</div>
		</div>
	</section>

	<section class="mt-8 border-0 border-t border-solid border-surface-4 pt-6">
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
			<Toggle
				id="show-jump-in-section"
				:model-value="settings.show_jump_in"
				:aria-label="formatMessage(messages.showJumpInTitle)"
				@update:model-value="updateSetting('show_jump_in', $event)"
			/>
		</div>
	</section>
</template>
