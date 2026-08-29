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
		<UnsavedChangesPopup
			:original="saved"
			:modified="changes"
			:saving="saving"
			@reset="reset"
			@save="saveAppearanceSettings"
		/>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
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
	UnsavedChangesPopup,
	useSavable,
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

type AppearanceSettingsState = {
	theme: Theme | 'system'
	syncAcrossDevices: boolean
	advancedRendering: boolean
	projectLayouts: ProjectLayoutSetting[]
	externalLinksNewTab: boolean
	sidebarPreferences: SidebarPreferences
}

const layoutPreferenceKeys: Record<
	ProjectDisplayLocation,
	keyof Labrinth.Users.v3.LayoutPreferences
> = {
	mod: 'mods',
	plugin: 'plugins',
	datapack: 'datapacks',
	shader: 'shaders',
	resourcepack: 'resourcepacks',
	modpack: 'modpacks',
	server: 'servers',
	user: 'users',
}

function getAppearanceSettingsState(): AppearanceSettingsState {
	return {
		theme: theme.preferred,
		syncAcrossDevices: theme.syncAcrossDevices,
		advancedRendering: cosmetics.value.advancedRendering,
		projectLayouts: projectLayouts.value,
		externalLinksNewTab: cosmetics.value.externalLinksNewTab,
		sidebarPreferences: sidebarPreferences.value,
	}
}

const { saved, current, changes, saving, reset, save } = useSavable(
	getAppearanceSettingsState,
	async (appearanceChanges) => {
		const value = current.value
		const preferencesPatch: Labrinth.Users.v3.PartialUserPreferences = {}

		if (
			value.syncAcrossDevices &&
			auth.user.value &&
			(appearanceChanges.theme !== undefined || appearanceChanges.syncAcrossDevices !== undefined)
		) {
			preferencesPatch.appearance =
				value.theme === 'system' ? { auto: true } : { auto: false, theme: value.theme }
		}

		if (appearanceChanges.projectLayouts !== undefined) {
			const layouts: Partial<Labrinth.Users.v3.LayoutPreferences> = {}
			for (const setting of value.projectLayouts) {
				layouts[layoutPreferenceKeys[setting.type]] = setting.layout
			}
			preferencesPatch.layouts = layouts
		}

		if (appearanceChanges.sidebarPreferences !== undefined) {
			preferencesPatch.sidebars = value.sidebarPreferences
		}

		if (Object.keys(preferencesPatch).length > 0) {
			await updatePreferences(preferencesPatch)
		}

		if (value.theme !== 'system') {
			if (isDarkTheme(value.theme)) {
				theme.preferences.dark = value.theme
			} else {
				theme.preferences.light = value.theme
			}
		}

		theme.preferred = value.theme
		theme.syncAcrossDevices = value.syncAcrossDevices
		cosmetics.value.advancedRendering = value.advancedRendering
		cosmetics.value.externalLinksNewTab = value.externalLinksNewTab
		cosmetics.value.rightSearchLayout = value.sidebarPreferences.right_aligned_search
		cosmetics.value.leftContentLayout = value.sidebarPreferences.left_aligned_content
		for (const setting of value.projectLayouts) {
			cosmetics.value.searchDisplayMode[setting.type] = setting.layout === 'rows' ? 'list' : 'grid'
		}
	},
)

const themeOptions = computed(() => {
	const options: ('system' | Theme)[] = ['system', 'light', 'dark', 'oled']
	if (flags.value.developerMode || current.value.theme === 'retro') {
		options.push('retro')
	}
	return options
})

function setTheme(value: Theme | 'system'): void {
	current.value.theme = value
}

function setSyncAcrossDevices(value: boolean): void {
	current.value.syncAcrossDevices = value
}

function setAdvancedRendering(value: boolean): void {
	current.value.advancedRendering = value
}

function setProjectLayout(type: ProjectDisplayLocation, layout: ProjectLayout): void {
	current.value.projectLayouts = current.value.projectLayouts.map((setting) =>
		setting.type === type ? { ...setting, layout } : setting,
	)
}

function setExternalLinksNewTab(value: boolean): void {
	current.value.externalLinksNewTab = value
}

function setSidebarPreference(key: keyof SidebarPreferences, value: boolean): void {
	current.value.sidebarPreferences = {
		...current.value.sidebarPreferences,
		[key]: value,
	}
}

watch(
	[() => current.value.theme, () => saved.value.theme],
	([selectedTheme, savedTheme]) => {
		theme.preview = selectedTheme === savedTheme ? null : selectedTheme
	},
	{ immediate: true },
)

onBeforeUnmount(() => {
	theme.preview = null
})

async function saveAppearanceSettings(): Promise<void> {
	try {
		await save()
	} catch {
		return
	}
}

provideAppearanceSettings({
	deferPersistence: true,
	theme: {
		current: computed(() => current.value.theme),
		options: themeOptions,
		system: systemTheme,
		preferredDark: computed(() =>
			isDarkTheme(current.value.theme) ? current.value.theme : theme.preferences.dark,
		),
		set: setTheme,
		syncAcrossDevices: {
			value: computed(() => current.value.syncAcrossDevices),
			set: setSyncAcrossDevices,
		},
		syncDisabled: computed(() => !auth.user.value),
	},
	advancedRendering: {
		value: computed(() => current.value.advancedRendering),
		set: setAdvancedRendering,
	},
	projectLayouts: {
		value: computed(() => current.value.projectLayouts),
		set: setProjectLayout,
	},
	externalLinksNewTab: {
		value: computed(() => current.value.externalLinksNewTab),
		set: setExternalLinksNewTab,
	},
	sidebarPreferences: {
		value: computed(() => current.value.sidebarPreferences),
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
