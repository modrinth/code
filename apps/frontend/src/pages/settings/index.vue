<template>
	<div>
		<Admonition v-if="flags.developerMode" type="critical" class="mb-4" show-actions-underneath>
			<template #icon="{ iconClass }">
				<CodeIcon :class="iconClass" aria-hidden="true" />
			</template>
			<IntlFormatted :message-id="developerModeBanner.description">
				<template #strong="{ children }">
					<strong>
						<component :is="() => normalizeChildren(children)" />
					</strong>
				</template>
			</IntlFormatted>
			<template #actions>
				<Button type="colored" color="red" @click="disableDeveloperMode()">
					{{ formatMessage(developerModeBanner.deactivate) }}
				</Button>
			</template>
		</Admonition>
		<section class="universal-card">
			<AppearanceSettingsLayout />
		</section>
	</div>
</template>

<script setup lang="ts">
import { CodeIcon } from '@modrinth/assets'
import {
	Admonition,
	AppearanceSettingsLayout,
	Button,
	defineMessages,
	injectAuth,
	injectNotificationManager,
	injectUserPreferences,
	IntlFormatted,
	isProjectDisplayLocation,
	normalizeChildren,
	type ProjectDisplayLocation,
	type ProjectLayout,
	type ProjectLayoutSetting,
	provideAppearanceSettings,
	type SidebarPreferences,
	useVIntl,
} from '@modrinth/ui'

import { isDarkTheme, type Theme } from '~/plugins/theme/index.ts'

const { addNotification } = injectNotificationManager()
const auth = injectAuth()
const { updatePreferences } = injectUserPreferences()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	headTitle: {
		id: 'settings.head-title',
		defaultMessage: 'Display settings',
	},
})

const developerModeBanner = defineMessages({
	description: {
		id: 'settings.display.banner.developer-mode.description',
		defaultMessage:
			"<strong>Developer mode</strong> is active. This will allow you to view the internal IDs of various things throughout Modrinth that may be helpful if you're a developer using the Modrinth API. Click on the Modrinth logo at the bottom of the page 5 times to toggle developer mode.",
	},
	deactivate: {
		id: 'settings.display.banner.developer-mode.button',
		defaultMessage: 'Deactivate developer mode',
	},
})

useHead({
	title: () => `${formatMessage(messages.headTitle)} - Modrinth`,
})

const notifications = defineMessages({
	developerModeDeactivatedTitle: {
		id: 'settings.display.notification.developer-mode-deactivated.title',
		defaultMessage: 'Developer mode deactivated',
	},
	developerModeDeactivatedText: {
		id: 'settings.display.notification.developer-mode-deactivated.text',
		defaultMessage: 'Developer mode has been disabled',
	},
})

const cosmetics = useCosmetics()
const flags = useFeatureFlags()
const tags = useGeneratedState()
const theme = useTheme()

// On the server the value of native theme can be 'unknown'. To hydrate
// correctly, we need to make sure we aren't using 'unknown' and values between
// server and client renders are in sync.

const serverSystemTheme = useState(() => {
	const theme_ = theme.native
	if (theme_ === 'unknown') return 'light'
	return theme_
})

const systemTheme = useMountedValue((mounted): Theme => {
	const systemTheme_ = mounted ? theme.native : serverSystemTheme.value
	return systemTheme_ === 'light' ? theme.preferences.light : theme.preferences.dark
})

const themeOptions = computed(() => {
	const options: ('system' | Theme)[] = ['system', 'light', 'dark', 'oled']
	if (flags.value.developerMode || theme.preferred === 'retro') {
		options.push('retro')
	}
	return options
})

const projectLayouts = computed<ProjectLayoutSetting[]>(() => {
	const layouts = tags.value.projectTypes
		.map(({ id }) => id)
		.filter(isProjectDisplayLocation)
		.map(
			(type): ProjectLayoutSetting => ({
				type,
				layout: cosmetics.value.searchDisplayMode[type] === 'list' ? 'rows' : 'grid',
			}),
		)

	layouts.push({
		type: 'user',
		layout: cosmetics.value.searchDisplayMode.user === 'list' ? 'rows' : 'grid',
	})

	return layouts
})

const sidebarPreferences = computed<SidebarPreferences>(() => ({
	right_aligned_search: cosmetics.value.rightSearchLayout,
	left_aligned_content: cosmetics.value.leftContentLayout,
}))

function setTheme(value: Theme | 'system'): void {
	if (value !== 'system') {
		if (isDarkTheme(value)) {
			theme.preferences.dark = value
		} else {
			theme.preferences.light = value
		}
	}

	theme.preferred = value
}

function setSyncAcrossDevices(value: boolean): void {
	theme.syncAcrossDevices = value
}

function setAdvancedRendering(value: boolean): void {
	cosmetics.value.advancedRendering = value
}

function setProjectLayout(type: ProjectDisplayLocation, layout: ProjectLayout): void {
	cosmetics.value.searchDisplayMode[type] = layout === 'rows' ? 'list' : 'grid'
}

function setExternalLinksNewTab(value: boolean): void {
	cosmetics.value.externalLinksNewTab = value
}

function setSidebarPreference(key: keyof SidebarPreferences, value: boolean): void {
	if (key === 'right_aligned_search') {
		cosmetics.value.rightSearchLayout = value
	} else {
		cosmetics.value.leftContentLayout = value
	}
}

provideAppearanceSettings({
	theme: {
		current: computed(() => theme.preferred),
		options: themeOptions,
		system: systemTheme,
		set: setTheme,
		syncAcrossDevices: {
			value: computed(() => theme.syncAcrossDevices),
			set: setSyncAcrossDevices,
		},
		syncDisabled: computed(() => !auth.user.value),
	},
	advancedRendering: {
		value: computed(() => cosmetics.value.advancedRendering),
		set: setAdvancedRendering,
	},
	projectLayouts: {
		value: projectLayouts,
		set: setProjectLayout,
	},
	externalLinksNewTab: {
		value: computed(() => cosmetics.value.externalLinksNewTab),
		set: setExternalLinksNewTab,
	},
	sidebarPreferences: {
		value: sidebarPreferences,
		set: setSidebarPreference,
	},
	updatePreferences,
})

function disableDeveloperMode(): void {
	flags.value.developerMode = !flags.value.developerMode
	saveFeatureFlags()
	addNotification({
		title: formatMessage(notifications.developerModeDeactivatedTitle),
		text: formatMessage(notifications.developerModeDeactivatedText),
		type: 'success',
	})
}
</script>
