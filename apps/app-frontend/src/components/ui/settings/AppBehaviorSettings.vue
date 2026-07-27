<script setup lang="ts">
import { Chips, defineMessages, Toggle, useVIntl } from '@modrinth/ui'
import { ref, watch } from 'vue'

import { get, set } from '@/helpers/settings.ts'
import { useTheming } from '@/store/state'
import type { FeatureFlag } from '@/store/theme.ts'

const themeStore = useTheming()
const { formatMessage } = useVIntl()

const worldsInHomeFlag: FeatureFlag = 'worlds_in_home'
const skipNonEssentialWarningsFlag: FeatureFlag = 'skip_non_essential_warnings'
const skipUnknownPackWarningFlag: FeatureFlag = 'skip_unknown_pack_warning'
const showPlayTimeFlag: FeatureFlag = 'show_instance_play_time'

type LandingPage = 'Home' | 'Library'
const landingPageOptions: LandingPage[] = ['Home', 'Library']

const messages = defineMessages({
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
	defaultLandingPageTitle: {
		id: 'app.appearance-settings.default-landing-page.title',
		defaultMessage: 'Default landing page',
	},
	defaultLandingPageDescription: {
		id: 'app.appearance-settings.default-landing-page.description',
		defaultMessage: 'Choose the page shown when Modrinth App opens.',
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
		defaultMessage: 'Jump back into worlds',
	},
	jumpBackIntoWorldsDescription: {
		id: 'app.appearance-settings.jump-back-into-worlds.description',
		defaultMessage: 'Show recently played worlds in the "Jump back in" section on the Home page.',
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
		defaultMessage: 'Hide your username above the player preview on the Skins page.',
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

function formatLandingPageLabel(page: LandingPage) {
	switch (page) {
		case 'Home':
			return formatMessage(messages.defaultLandingPageHome)
		case 'Library':
			return formatMessage(messages.defaultLandingPageLibrary)
	}
}

const settings = ref(await get())

watch(
	settings,
	async () => {
		await set(settings.value)
	},
	{ deep: true },
)
</script>
<template>
	<section>
		<h2 class="m-0 text-xl font-semibold text-contrast">
			{{ formatMessage(messages.startupAndNavigationTitle) }}
		</h2>
		<div class="mt-4 flex flex-col gap-6">
			<div class="flex flex-col gap-2.5">
				<h3 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.defaultLandingPageTitle) }}
				</h3>
				<Chips
					v-model="settings.default_page"
					:items="landingPageOptions"
					:format-label="formatLandingPageLabel"
					:capitalize="false"
					:aria-label="formatMessage(messages.defaultLandingPageTitle)"
				/>
				<p class="m-0">
					{{ formatMessage(messages.defaultLandingPageDescription) }}
				</p>
			</div>

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
							themeStore.toggleSidebar = settings.toggle_sidebar
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
					:model-value="themeStore.getFeatureFlag(worldsInHomeFlag)"
					@update:model-value="
						() => {
							const newValue = !themeStore.getFeatureFlag(worldsInHomeFlag)
							themeStore.featureFlags[worldsInHomeFlag] = newValue
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
					:model-value="themeStore.getFeatureFlag(showPlayTimeFlag)"
					@update:model-value="
						() => {
							const newValue = !themeStore.getFeatureFlag(showPlayTimeFlag)
							themeStore.featureFlags[showPlayTimeFlag] = newValue
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
					:model-value="themeStore.hideNametagSkinsPage"
					@update:model-value="
						(e) => {
							themeStore.hideNametagSkinsPage = !!e
							settings.hide_nametag_skins_page = themeStore.hideNametagSkinsPage
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
					:model-value="!themeStore.getFeatureFlag(skipUnknownPackWarningFlag)"
					@update:model-value="
						(e) => {
							const warnBeforeUnknownPackInstall = !!e
							const skipUnknownPackWarning = !warnBeforeUnknownPackInstall
							themeStore.featureFlags[skipUnknownPackWarningFlag] = skipUnknownPackWarning
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
					:model-value="themeStore.getFeatureFlag(skipNonEssentialWarningsFlag)"
					@update:model-value="
						() => {
							const newValue = !themeStore.getFeatureFlag(skipNonEssentialWarningsFlag)
							themeStore.featureFlags[skipNonEssentialWarningsFlag] = newValue
							settings.feature_flags[skipNonEssentialWarningsFlag] = newValue
						}
					"
				/>
			</div>
		</div>
	</section>
</template>
