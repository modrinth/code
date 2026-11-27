<script setup lang="ts">
import {
	defineMessages,
	injectAuth,
	injectNotificationManager,
	injectUserPreferences,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed } from 'vue'

import { useAppSettings } from '@/composables/use-app-settings.ts'
import HiddenInstancesSettings from '@/features/settings/display/hidden-instances-settings/index.vue'
import {
	type AppSettings,
	appSettingsKeys,
	appSettingsQueryOptions,
	get,
	set,
} from '@/helpers/settings.ts'

const appSettings = useAppSettings()
const { formatMessage } = useVIntl()
const auth = injectAuth()
const { handleError } = injectNotificationManager()
const { updatePreferences } = injectUserPreferences()
const queryClient = useQueryClient()

const messages = defineMessages({
	syncAcrossDevicesTitle: {
		id: 'app.behavior-settings.sync-across-devices.title',
		defaultMessage: 'Sync behavior across devices',
	},
	syncAcrossDevicesDescription: {
		id: 'app.behavior-settings.sync-across-devices.description',
		defaultMessage:
			"Use these behavior settings everywhere you're signed in. Turn this off to keep separate settings on this device.",
	},
	syncAcrossDevicesSignedOutTooltip: {
		id: 'app.behavior-settings.sync-across-devices.signed-out-tooltip',
		defaultMessage: 'Sign into a Modrinth account to sync settings.',
	},
	startupAndNavigationTitle: {
		id: 'app.behavior-settings.startup-and-navigation.title',
		defaultMessage: 'Startup and navigation',
	},
	contentTitle: {
		id: 'app.behavior-settings.content.title',
		defaultMessage: 'Home and content',
	},
	confirmationsTitle: {
		id: 'app.behavior-settings.confirmations.title',
		defaultMessage: 'Confirmations',
	},
	minimizeLauncherTitle: {
		id: 'app.appearance-settings.minimize-launcher.title',
		defaultMessage: 'Minimize app',
	},
	minimizeLauncherDescription: {
		id: 'app.appearance-settings.minimize-launcher.description',
		defaultMessage: 'Minimize Modrinth App when Minecraft starts.',
	},
	refocusOnGameCloseTitle: {
		id: 'app.behavior-settings.refocus-on-game-close.title',
		defaultMessage: 'Refocus app when Minecraft closes',
	},
	refocusOnGameCloseDescription: {
		id: 'app.behavior-settings.refocus-on-game-close.description',
		defaultMessage: 'Bring Modrinth App to the foreground when Minecraft exits.',
	},
	defaultLandingPageHome: {
		id: 'app.appearance-settings.default-landing-page.home',
		defaultMessage: 'Home',
	},
	defaultLandingPageLibrary: {
		id: 'app.appearance-settings.default-landing-page.library',
		defaultMessage: 'Library',
	},
	toggleSidebarTitle: {
		id: 'app.appearance-settings.toggle-sidebar.title',
		defaultMessage: 'Hide right sidebar',
	},
	toggleSidebarDescription: {
		id: 'app.appearance-settings.toggle-sidebar.description',
		defaultMessage: 'Hide the right sidebar by default and add a button to show or hide it.',
	},
	compactModeTitle: {
		id: 'app.appearance-settings.compact-mode.title',
		defaultMessage: 'Compact mode',
	},
	compactModeDescription: {
		id: 'app.appearance-settings.compact-mode.description',
		defaultMessage: 'Display library instances in a compact row layout.',
	},
	showPlayTimeTitle: {
		id: 'app.appearance-settings.show-play-time.title',
		defaultMessage: 'Show play time',
	},
	showPlayTimeDescription: {
		id: 'app.appearance-settings.show-play-time.description',
		defaultMessage: `Show how long you've played each instance.`,
	},
	hideNametagTitle: {
		id: 'app.appearance-settings.hide-nametag.title',
		defaultMessage: 'Hide nametag',
	},
	hideNametagDescription: {
		id: 'app.appearance-settings.hide-nametag.description',
		defaultMessage: 'Hide your username above the player preview on the Skin selector page.',
	},
	unknownPackWarningTitle: {
		id: 'app.appearance-settings.unknown-pack-warning.title',
		defaultMessage: 'Warn me before installing unknown modpacks',
	},
	unknownPackWarningDescription: {
		id: 'app.appearance-settings.unknown-pack-warning.description',
		defaultMessage:
			"Show a safety warning before installing a Modrinth Pack (.mrpack) that isn't hosted on Modrinth.",
	},
	skipNonEssentialWarningsTitle: {
		id: 'app.appearance-settings.skip-non-essential-warnings.title',
		defaultMessage: 'Skip non-essential warnings',
	},
	skipNonEssentialWarningsDescription: {
		id: 'app.appearance-settings.skip-non-essential-warnings.description',
		defaultMessage:
			'Skip confirmations for low-risk actions such as duplicate installs, normal content deletion, bulk updates, unlinking, and repairs. Warnings for dangerous actions are always shown.',
	},
})

type BehaviorSettingKey =
	| 'sync_behavior_across_devices'
	| 'hide_on_process_start'
	| 'refocus_on_game_close'
	| 'toggle_sidebar'
	| 'compact_instance_cards'
	| 'show_play_time'
	| 'hide_nametag_skins_page'
	| 'warn_on_unknown_modpacks'
	| 'skip_non_essential_warnings'

type BehaviorSettingsPatch = Partial<Pick<AppSettings, BehaviorSettingKey>>

const settingsQuery = useQuery(appSettingsQueryOptions())
await settingsQuery.suspense()

const settings = computed(() => settingsQuery.data.value!)

function applyBehaviorSettings(value: AppSettings): void {
	appSettings.setBehaviorSyncAcrossDevices(value.sync_behavior_across_devices)
	appSettings.refocusOnGameClose = value.refocus_on_game_close
	appSettings.compactInstanceCards = value.compact_instance_cards
	appSettings.showPlayTime = value.show_play_time
	appSettings.warnOnUnknownModpacks = value.warn_on_unknown_modpacks
	appSettings.skipNonEssentialWarnings = value.skip_non_essential_warnings
	appSettings.toggleSidebar = value.toggle_sidebar
	appSettings.hideNametagSkinsPage = value.hide_nametag_skins_page
}

const settingsMutation = useMutation({
	mutationKey: appSettingsKeys.update,
	scope: { id: 'app-settings' },
	mutationFn: async (patch: BehaviorSettingsPatch) => {
		const nextSettings: AppSettings = { ...(await get()), ...patch }

		if (nextSettings.sync_behavior_across_devices && auth.user.value) {
			await updatePreferences({
				behavior: {
					minimize_app: nextSettings.hide_on_process_start,
					refocus_on_game_close: nextSettings.refocus_on_game_close,
					hide_right_sidebar: nextSettings.toggle_sidebar,
					compact_instance_cards: nextSettings.compact_instance_cards,
					show_play_time: nextSettings.show_play_time,
					hide_nametag: nextSettings.hide_nametag_skins_page,
					warn_on_unknown_modpacks: nextSettings.warn_on_unknown_modpacks,
					skip_non_essential_warnings: nextSettings.skip_non_essential_warnings,
				},
			})
		}

		await set(nextSettings)
	},
	onMutate: async (patch) => {
		await queryClient.cancelQueries({ queryKey: appSettingsKeys.all })
		const previousSettings = queryClient.getQueryData<AppSettings>(appSettingsKeys.all)
		if (previousSettings) {
			const nextSettings = { ...previousSettings, ...patch }
			queryClient.setQueryData(appSettingsKeys.all, nextSettings)
			applyBehaviorSettings(nextSettings)
		}
		return { previousSettings }
	},
	onError: (error, _patch, context) => {
		if (context?.previousSettings) {
			queryClient.setQueryData(appSettingsKeys.all, context.previousSettings)
			applyBehaviorSettings(context.previousSettings)
		}
		handleError(error)
	},
	onSettled: async () => {
		if (queryClient.isMutating({ mutationKey: appSettingsKeys.update }) === 1) {
			await queryClient.invalidateQueries({ queryKey: appSettingsKeys.all })
		}
	},
})

function updateSetting<K extends BehaviorSettingKey>(key: K, value: AppSettings[K]): void {
	settingsMutation.mutate({ [key]: value } as BehaviorSettingsPatch)
}
</script>
<template>
	<section class="border-0 border-b border-solid border-divider pb-6">
		<div class="flex items-center justify-between gap-4">
			<div>
				<h2 id="sync-behavior-across-devices-label" class="m-0 text-lg font-semibold text-contrast">
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
					id="sync-behavior-across-devices"
					:model-value="Boolean(auth.user.value) && settings.sync_behavior_across_devices"
					:disabled="!auth.user.value"
					aria-labelledby="sync-behavior-across-devices-label"
					@update:model-value="updateSetting('sync_behavior_across_devices', $event)"
				/>
			</span>
		</div>
	</section>

	<section class="mt-6">
		<h2 class="m-0 text-xl font-semibold text-contrast">
			{{ formatMessage(messages.startupAndNavigationTitle) }}
		</h2>
		<div class="mt-4 flex flex-col gap-6">
			<div class="flex items-center justify-between gap-4">
				<div>
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.minimizeLauncherTitle) }}
					</h3>
					<p class="m-0 mt-1">
						{{ formatMessage(messages.minimizeLauncherDescription) }}
					</p>
				</div>
				<Toggle
					id="minimize-launcher"
					:model-value="settings.hide_on_process_start"
					@update:model-value="updateSetting('hide_on_process_start', $event)"
				/>
			</div>

			<div class="flex items-center justify-between gap-4">
				<div>
					<h3 id="refocus-on-game-close-label" class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.refocusOnGameCloseTitle) }}
					</h3>
					<p id="refocus-on-game-close-description" class="m-0 mt-1">
						{{ formatMessage(messages.refocusOnGameCloseDescription) }}
					</p>
				</div>
				<Toggle
					id="refocus-on-game-close"
					:model-value="settings.refocus_on_game_close"
					aria-labelledby="refocus-on-game-close-label"
					aria-describedby="refocus-on-game-close-description"
					@update:model-value="updateSetting('refocus_on_game_close', $event)"
				/>
			</div>

			<div class="flex items-center justify-between gap-4">
				<div>
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.toggleSidebarTitle) }}
					</h3>
					<p class="m-0 mt-1">{{ formatMessage(messages.toggleSidebarDescription) }}</p>
				</div>
				<Toggle
					id="toggle-sidebar"
					:model-value="settings.toggle_sidebar"
					@update:model-value="updateSetting('toggle_sidebar', $event)"
				/>
			</div>
		</div>
	</section>

	<section class="mt-8 border-0 border-t border-solid border-divider pt-6">
		<h2 class="m-0 text-xl font-semibold text-contrast">
			{{ formatMessage(messages.contentTitle) }}
		</h2>
		<div class="mt-4 flex flex-col gap-6">
			<div class="flex items-center justify-between gap-4">
				<div>
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.compactModeTitle) }}
					</h3>
					<p class="m-0 mt-1">{{ formatMessage(messages.compactModeDescription) }}</p>
				</div>
				<Toggle
					id="compact-mode"
					:model-value="settings.compact_instance_cards"
					@update:model-value="updateSetting('compact_instance_cards', $event)"
				/>
			</div>

			<div class="flex items-center justify-between gap-4">
				<div>
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.showPlayTimeTitle) }}
					</h3>
					<p class="m-0 mt-1">{{ formatMessage(messages.showPlayTimeDescription) }}</p>
				</div>
				<Toggle
					id="show-play-time"
					:model-value="settings.show_play_time"
					@update:model-value="updateSetting('show_play_time', $event)"
				/>
			</div>

			<div class="flex items-center justify-between gap-4">
				<div>
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.hideNametagTitle) }}
					</h3>
					<p class="m-0 mt-1">{{ formatMessage(messages.hideNametagDescription) }}</p>
				</div>
				<Toggle
					id="hide-nametag-skins-page"
					:model-value="settings.hide_nametag_skins_page"
					@update:model-value="updateSetting('hide_nametag_skins_page', $event)"
				/>
			</div>
		</div>
	</section>

	<section class="mt-8 border-0 border-t border-solid border-divider pt-6">
		<h2 class="m-0 text-xl font-semibold text-contrast">
			{{ formatMessage(messages.confirmationsTitle) }}
		</h2>
		<div class="mt-4 flex flex-col gap-6">
			<div class="flex items-center justify-between gap-4">
				<div>
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.unknownPackWarningTitle) }}
					</h3>
					<p class="m-0 mt-1">
						{{ formatMessage(messages.unknownPackWarningDescription) }}
					</p>
				</div>
				<Toggle
					id="warn-before-installing-unknown-modpacks"
					:model-value="settings.warn_on_unknown_modpacks"
					@update:model-value="updateSetting('warn_on_unknown_modpacks', $event)"
				/>
			</div>

			<div class="flex items-center justify-between gap-4">
				<div>
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.skipNonEssentialWarningsTitle) }}
					</h3>
					<p class="m-0 mt-1">
						{{ formatMessage(messages.skipNonEssentialWarningsDescription) }}
					</p>
				</div>
				<Toggle
					id="skip-non-essential-warnings"
					:model-value="settings.skip_non_essential_warnings"
					@update:model-value="updateSetting('skip_non_essential_warnings', $event)"
				/>
			</div>
		</div>
	</section>

	<HiddenInstancesSettings />
</template>
