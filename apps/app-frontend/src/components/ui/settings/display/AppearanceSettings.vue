<script setup lang="ts">
import { defineMessages, ThemeSelector, Toggle, useVIntl } from '@modrinth/ui'
import { computed, ref, watch } from 'vue'

import { get, set } from '@/helpers/settings.ts'
import { getOS } from '@/helpers/utils'
import { useTheming } from '@/store/state'
import type { ColorTheme } from '@/store/theme.ts'

const themeStore = useTheming()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	colorThemeTitle: {
		id: 'app.appearance-settings.color-theme.title',
		defaultMessage: 'Color theme',
	},
	colorThemeDescription: {
		id: 'app.appearance-settings.color-theme.description',
		defaultMessage: 'Choose the color theme used by Modrinth App.',
	},
	advancedRenderingTitle: {
		id: 'app.appearance-settings.advanced-rendering.title',
		defaultMessage: 'Advanced rendering',
	},
	advancedRenderingDescription: {
		id: 'app.appearance-settings.advanced-rendering.description',
		defaultMessage:
			'Enable visual effects such as background blur. This may reduce performance without hardware acceleration.',
	},
	nativeDecorationsTitle: {
		id: 'app.appearance-settings.native-decorations.title',
		defaultMessage: 'System window frame',
	},
	nativeDecorationsDescription: {
		id: 'app.appearance-settings.native-decorations.description',
		defaultMessage:
			"Use your operating system's title bar and window controls. Requires an app restart.",
	},
})

const os = ref(await getOS())
const settings = ref(await get())
const themeOptions = computed(() =>
	themeStore
		.getThemeOptions()
		.filter((theme) => theme !== 'retro' || themeStore.devMode || settings.value.theme === 'retro'),
)

watch(
	settings,
	async () => {
		await set(settings.value)
	},
	{ deep: true },
)
</script>
<template>
	<h2 class="m-0 text-lg font-semibold text-contrast">
		{{ formatMessage(messages.colorThemeTitle) }}
	</h2>

	<p class="m-0 mt-1">{{ formatMessage(messages.colorThemeDescription) }}</p>

	<ThemeSelector
		:update-color-theme="
			(theme: ColorTheme) => {
				themeStore.setThemeState(theme)
				settings.theme = theme
			}
		"
		:current-theme="settings.theme"
		:theme-options="themeOptions"
		system-theme-color="system"
	/>

	<div class="mt-6 flex items-center justify-between">
		<div>
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.advancedRenderingTitle) }}
			</h2>
			<p class="m-0 mt-1">
				{{ formatMessage(messages.advancedRenderingDescription) }}
			</p>
		</div>

		<Toggle
			id="advanced-rendering"
			:model-value="themeStore.advancedRendering"
			@update:model-value="
				(e) => {
					themeStore.advancedRendering = !!e
					settings.advanced_rendering = themeStore.advancedRendering
				}
			"
		/>
	</div>

	<div v-if="os !== 'MacOS'" class="mt-6 flex items-center justify-between gap-4">
		<div>
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.nativeDecorationsTitle) }}
			</h2>
			<p class="m-0 mt-1">{{ formatMessage(messages.nativeDecorationsDescription) }}</p>
		</div>
		<Toggle id="native-decorations" v-model="settings.native_decorations" />
	</div>
</template>
