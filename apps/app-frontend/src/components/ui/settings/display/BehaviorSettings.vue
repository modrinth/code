<script setup lang="ts">
import { defineMessages, injectAuth, injectUserPreferences, Toggle, useVIntl } from '@modrinth/ui'
import { ref, watch } from 'vue'

import { type FeatureFlag, useAppSettings } from '@/composables/use-app-settings.ts'
import { get, set } from '@/helpers/settings.ts'

const appSettings = useAppSettings()
const { formatMessage } = useVIntl()
const auth = injectAuth()
const { updatePreferences } = injectUserPreferences()

const worldsInHomeFlag: FeatureFlag = 'worlds_in_home'
const skipNonEssentialWarningsFlag: FeatureFlag = 'skip_non_essential_warnings'
const skipUnknownPackWarningFlag: FeatureFlag = 'skip_unknown_pack_warning'
const showPlayTimeFlag: FeatureFlag = 'show_instance_play_time'

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
	jumpBackIntoWorldsTitle: {
		id: 'app.appearance-settings.jump-back-into-worlds.title',
		defaultMessage: 'Jump into worlds or instances',
	},
	jumpBackIntoWorldsDescription: {
		id: 'app.appearance-settings.jump-back-into-worlds.description',
		defaultMessage:
			'Show recently played worlds or instances in the "Jump in" section on the Home page.',
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

const settings = ref(await get())

function syncBehaviorPreferences() {
	return updatePreferences({
		behavior: {
			minimize_app: settings.value.hide_on_process_start,
			hide_right_sidebar: settings.value.toggle_sidebar,
			show_jump_in: appSettings.getFeatureFlag(worldsInHomeFlag),
			show_play_time: appSettings.getFeatureFlag(showPlayTimeFlag),
			hide_nametag: settings.value.hide_nametag_skins_page,
			warn_on_unknown_modpacks: !appSettings.getFeatureFlag(skipUnknownPackWarningFlag),
			skip_non_essential_warnings: appSettings.getFeatureFlag(skipNonEssentialWarningsFlag),
		},
	}).catch(() => {
		setSyncBehaviorAcrossDevices(false)
		return undefined
	})
}

function setSyncBehaviorAcrossDevices(value: boolean) {
	appSettings.setBehaviorSyncAcrossDevices(value)
	settings.value.sync_behavior_across_devices = value
}

watch(
	settings,
	async () => {
		await set(settings.value)
		if (!appSettings.syncBehaviorAcrossDevices) return

		await syncBehaviorPreferences()
	},
	{ deep: true },
)
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
					:model-value="appSettings.syncBehaviorAcrossDevices"
					:disabled="!auth.user.value"
					aria-labelledby="sync-behavior-across-devices-label"
					@update:model-value="setSyncBehaviorAcrossDevices"
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
				<Toggle id="minimize-launcher" v-model="settings.hide_on_process_start" />
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
					@update:model-value="
						(e) => {
							settings.toggle_sidebar = !!e
							appSettings.toggleSidebar = settings.toggle_sidebar
						}
					"
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
						{{ formatMessage(messages.jumpBackIntoWorldsTitle) }}
					</h3>
					<p class="m-0 mt-1">
						{{ formatMessage(messages.jumpBackIntoWorldsDescription) }}
					</p>
				</div>
				<Toggle
					id="jump-back-into-worlds"
					:model-value="appSettings.getFeatureFlag(worldsInHomeFlag)"
					@update:model-value="
						() => {
							const newValue = !appSettings.getFeatureFlag(worldsInHomeFlag)
							appSettings.featureFlags[worldsInHomeFlag] = newValue
							settings.feature_flags[worldsInHomeFlag] = newValue
						}
					"
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
					:model-value="appSettings.getFeatureFlag(showPlayTimeFlag)"
					@update:model-value="
						() => {
							const newValue = !appSettings.getFeatureFlag(showPlayTimeFlag)
							appSettings.featureFlags[showPlayTimeFlag] = newValue
							settings.feature_flags[showPlayTimeFlag] = newValue
						}
					"
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
					:model-value="appSettings.hideNametagSkinsPage"
					@update:model-value="
						(e) => {
							appSettings.hideNametagSkinsPage = !!e
							settings.hide_nametag_skins_page = appSettings.hideNametagSkinsPage
						}
					"
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
					:model-value="!appSettings.getFeatureFlag(skipUnknownPackWarningFlag)"
					@update:model-value="
						(e) => {
							const warnBeforeUnknownPackInstall = !!e
							const skipUnknownPackWarning = !warnBeforeUnknownPackInstall
							appSettings.featureFlags[skipUnknownPackWarningFlag] = skipUnknownPackWarning
							settings.feature_flags[skipUnknownPackWarningFlag] = skipUnknownPackWarning
						}
					"
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
					:model-value="appSettings.getFeatureFlag(skipNonEssentialWarningsFlag)"
					@update:model-value="
						() => {
							const newValue = !appSettings.getFeatureFlag(skipNonEssentialWarningsFlag)
							appSettings.featureFlags[skipNonEssentialWarningsFlag] = newValue
							settings.feature_flags[skipNonEssentialWarningsFlag] = newValue
						}
					"
				/>
			</div>
		</div>
	</section>
</template>
