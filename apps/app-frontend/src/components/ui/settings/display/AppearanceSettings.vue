<script setup lang="ts">
import {
	AppearanceSettingsLayout,
	injectAuth,
	injectUserPreferences,
	provideAppearanceSettings,
} from '@modrinth/ui'
import { computed, ref, watch } from 'vue'

import { type ColorTheme, useTheme } from '@/composables/use-theme.ts'
import { get, set } from '@/helpers/settings.ts'
import { getOS } from '@/helpers/utils'

const theme = useTheme()
const auth = injectAuth()
const { updatePreferences } = injectUserPreferences()
const os = await getOS()
const settings = ref(await get())
const themeOptions = computed(() =>
	theme.options.filter(
		(option) => option !== 'retro' || settings.value.developer_mode || theme.preferred === 'retro',
	),
)

function setTheme(value: ColorTheme): void {
	theme.preferred = value
	settings.value.theme = value
}

function setSyncAcrossDevices(enabled: boolean): void {
	theme.syncAcrossDevices = enabled
	settings.value.sync_theme_across_devices = enabled
}

function setAdvancedRendering(enabled: boolean): void {
	theme.advancedRendering = enabled
	settings.value.advanced_rendering = enabled
}

function setNativeDecorations(enabled: boolean): void {
	settings.value.native_decorations = enabled
}

provideAppearanceSettings({
	theme: {
		current: computed(() => theme.preferred),
		options: themeOptions,
		system: computed(() => theme.native),
		set: setTheme,
		syncAcrossDevices: {
			value: computed(() => theme.syncAcrossDevices),
			set: setSyncAcrossDevices,
		},
		syncDisabled: computed(() => !auth.user.value),
	},
	advancedRendering: {
		value: computed(() => settings.value.advanced_rendering),
		set: setAdvancedRendering,
	},
	nativeDecorations:
		os !== 'MacOS'
			? {
					value: computed(() => settings.value.native_decorations),
					set: setNativeDecorations,
				}
			: undefined,
	updatePreferences,
})

watch(
	settings,
	async () => {
		await set(settings.value)
	},
	{ deep: true },
)
</script>

<template>
	<AppearanceSettingsLayout />
</template>
