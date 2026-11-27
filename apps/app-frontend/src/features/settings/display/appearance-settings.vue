<script setup lang="ts">
import {
	AppearanceSettingsLayout,
	injectAuth,
	injectNotificationManager,
	injectUserPreferences,
	provideAppearanceSettings,
} from '@modrinth/ui'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { platform } from '@tauri-apps/plugin-os'
import { computed } from 'vue'

import { useAppSettings } from '@/composables/use-app-settings.ts'
import { type ColorTheme, type DarkTheme, isDarkTheme, useTheme } from '@/composables/use-theme.ts'
import { type AppSettings, appSettingsKeys, get, set } from '@/helpers/settings.ts'

const theme = useTheme()
const appSettings = useAppSettings()
const auth = injectAuth()
const { handleError } = injectNotificationManager()
const { updatePreferences } = injectUserPreferences()
const queryClient = useQueryClient()
const os = platform()

type AppearanceSettingKey =
	| 'theme'
	| 'sync_theme_across_devices'
	| 'advanced_rendering'
	| 'native_decorations'

type AppearanceSettings = Pick<AppSettings, AppearanceSettingKey>

function readAppearanceSettings(): AppearanceSettings {
	return {
		theme: theme.preferred,
		sync_theme_across_devices: theme.syncAcrossDevices,
		advanced_rendering: theme.advancedRendering,
		native_decorations: appSettings.nativeDecorations,
	}
}

function applyAppearanceSettings(value: AppearanceSettings, preferredDark?: DarkTheme): void {
	if (preferredDark) {
		theme.preferredDark = preferredDark
	} else if (isDarkTheme(value.theme)) {
		theme.preferredDark = value.theme
	}
	theme.preferred = value.theme
	theme.syncAcrossDevices = value.sync_theme_across_devices
	theme.advancedRendering = value.advanced_rendering
	appSettings.nativeDecorations = value.native_decorations
}

const settingsMutation = useMutation({
	mutationKey: appSettingsKeys.update,
	scope: { id: 'app-settings' },
	networkMode: 'always',
	mutationFn: async (patch: Partial<AppearanceSettings>) => {
		await set({ ...(await get()), ...patch })
	},
	onMutate: async (patch) => {
		const previousAppearance = readAppearanceSettings()
		const previousPreferredDark = theme.preferredDark
		applyAppearanceSettings({ ...previousAppearance, ...patch })

		await queryClient.cancelQueries({ queryKey: appSettingsKeys.all })
		const previousSettings = queryClient.getQueryData<AppSettings>(appSettingsKeys.all)
		if (previousSettings) {
			queryClient.setQueryData(appSettingsKeys.all, { ...previousSettings, ...patch })
		}

		return { previousAppearance, previousPreferredDark, previousSettings }
	},
	onError: (error, _patch, context) => {
		if (context) {
			applyAppearanceSettings(context.previousAppearance, context.previousPreferredDark)
			if (context.previousSettings) {
				queryClient.setQueryData(appSettingsKeys.all, context.previousSettings)
			}
		}
		handleError(error)
	},
	onSettled: async () => {
		if (queryClient.isMutating({ mutationKey: appSettingsKeys.update }) === 1) {
			await queryClient.invalidateQueries({ queryKey: appSettingsKeys.all })
		}
	},
})

const themeOptions = computed(() =>
	theme.options.filter(
		(option) => option !== 'retro' || appSettings.devMode || theme.preferred === 'retro',
	),
)

const preferredDarkTheme = computed(() => theme.preferredDark)

function setTheme(value: ColorTheme): Promise<void> {
	return settingsMutation.mutateAsync({ theme: value })
}

function setSyncAcrossDevices(enabled: boolean): Promise<void> {
	return settingsMutation.mutateAsync({ sync_theme_across_devices: enabled })
}

function setAdvancedRendering(enabled: boolean): Promise<void> {
	return settingsMutation.mutateAsync({ advanced_rendering: enabled })
}

function setNativeDecorations(enabled: boolean): Promise<void> {
	return settingsMutation.mutateAsync({ native_decorations: enabled })
}

provideAppearanceSettings({
	theme: {
		current: computed(() => theme.preferred),
		options: themeOptions,
		system: computed(() => (theme.native === 'light' ? 'light' : preferredDarkTheme.value)),
		preferredDark: preferredDarkTheme,
		set: setTheme,
		syncAcrossDevices: {
			value: computed(() => theme.syncAcrossDevices),
			set: setSyncAcrossDevices,
		},
		syncDisabled: computed(() => !auth.user.value),
	},
	advancedRendering: {
		value: computed(() => theme.advancedRendering),
		set: setAdvancedRendering,
	},
	nativeDecorations:
		os !== 'macos'
			? {
					value: computed(() => appSettings.nativeDecorations),
					set: setNativeDecorations,
				}
			: undefined,
	updatePreferences,
})
</script>

<template>
	<AppearanceSettingsLayout />
</template>
