<script setup lang="ts">
import { defineMessages, injectNotificationManager, Input, Toggle, useVIntl } from '@modrinth/ui'
import { computed, type Ref, ref, watch } from 'vue'

import { edit } from '@/helpers/instance'
import { get } from '@/helpers/settings.ts'

import type { AppSettings } from '../../../../helpers/types'
import { injectInstanceSettings } from './instance-settings-context'
import SettingsOptionsTransition from './settings-options-transition.vue'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const { instance } = injectInstanceSettings()

const globalSettings = (await get().catch(handleError)) as AppSettings

const overrideWindowSettings = ref(
	!!instance.value.game_resolution || !!instance.value.force_fullscreen,
)
const resolution: Ref<[number, number]> = ref(
	instance.value.game_resolution ?? (globalSettings.game_resolution.slice() as [number, number]),
)
const fullscreenSetting: Ref<boolean> = ref(
	instance.value.force_fullscreen ?? globalSettings.force_fullscreen,
)

const editInstanceObject = computed(() => {
	if (!overrideWindowSettings.value) {
		return {
			force_fullscreen: null,
			game_resolution: null,
		}
	}
	return {
		force_fullscreen: fullscreenSetting.value,
		game_resolution: fullscreenSetting.value ? null : resolution.value,
	}
})

watch(
	[overrideWindowSettings, resolution, fullscreenSetting],
	async () => {
		await edit(instance.value.id, editInstanceObject.value)
	},
	{ deep: true },
)

const messages = defineMessages({
	window: {
		id: 'instance.settings.tabs.window',
		defaultMessage: 'Window',
	},
	customWindowSettings: {
		id: 'instance.settings.tabs.window.custom-window-settings',
		defaultMessage: 'Use custom window settings for this instance.',
	},
	fullscreen: {
		id: 'instance.settings.tabs.window.fullscreen',
		defaultMessage: 'Fullscreen',
	},
	fullscreenDescription: {
		id: 'instance.settings.tabs.window.fullscreen.description',
		defaultMessage: 'Make the game start in full screen when launched (using options.txt).',
	},
	width: {
		id: 'instance.settings.tabs.window.width',
		defaultMessage: 'Width',
	},
	widthDescription: {
		id: 'instance.settings.tabs.window.width.description',
		defaultMessage: 'The width of the game window when launched.',
	},
	enterWidth: {
		id: 'instance.settings.tabs.window.width.enter',
		defaultMessage: 'Enter width...',
	},
	height: {
		id: 'instance.settings.tabs.window.height',
		defaultMessage: 'Height',
	},
	heightDescription: {
		id: 'instance.settings.tabs.window.height.description',
		defaultMessage: 'The height of the game window when launched.',
	},
	enterHeight: {
		id: 'instance.settings.tabs.window.height.enter',
		defaultMessage: 'Enter height...',
	},
})
</script>

<template>
	<div class="flex flex-col">
		<div class="flex items-center justify-between gap-4">
			<div class="flex min-w-0 flex-col gap-1">
				<h2 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.window) }}
				</h2>
				<p class="m-0">{{ formatMessage(messages.customWindowSettings) }}</p>
			</div>
			<Toggle id="override-window-settings" v-model="overrideWindowSettings" />
		</div>
		<SettingsOptionsTransition :show="overrideWindowSettings">
			<div class="flex flex-col gap-6 pt-6">
				<div class="flex items-center gap-4 justify-between">
					<div class="flex flex-col gap-1">
						<h2 class="m-0 text-lg font-semibold text-contrast">
							{{ formatMessage(messages.fullscreen) }}
						</h2>
						<p class="m-0">
							{{ formatMessage(messages.fullscreenDescription) }}
						</p>
					</div>
					<Toggle id="fullscreen" v-model="fullscreenSetting" />
				</div>

				<div class="flex items-center gap-4 justify-between">
					<div class="flex flex-col gap-1">
						<h2 class="m-0 text-lg font-semibold text-contrast">
							{{ formatMessage(messages.width) }}
						</h2>
						<p class="m-0">
							{{ formatMessage(messages.widthDescription) }}
						</p>
					</div>
					<Input
						id="width"
						v-model="resolution[0]"
						autocomplete="off"
						:disabled="fullscreenSetting"
						type="number"
						:placeholder="formatMessage(messages.enterWidth)"
					/>
				</div>

				<div class="flex items-center gap-4 justify-between">
					<div class="flex flex-col gap-1">
						<h2 class="m-0 text-lg font-semibold text-contrast">
							{{ formatMessage(messages.height) }}
						</h2>
						<p class="m-0">
							{{ formatMessage(messages.heightDescription) }}
						</p>
					</div>
					<Input
						id="height"
						v-model="resolution[1]"
						autocomplete="off"
						:disabled="fullscreenSetting"
						type="number"
						:placeholder="formatMessage(messages.enterHeight)"
					/>
				</div>
			</div>
		</SettingsOptionsTransition>
	</div>
</template>
