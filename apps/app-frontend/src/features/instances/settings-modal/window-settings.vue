<script setup lang="ts">
import { defineMessages, injectNotificationManager, Input, Toggle, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import { edit } from '@/helpers/instance'
import { get } from '@/helpers/settings.ts'
import type { AppSettings } from '@/helpers/types'

import DefaultSettingsIntro from './default-settings-intro.vue'
import { injectInstanceSettings } from './instance-settings-context'
import ResetToDefaultButton from './reset-to-default-button.vue'
import { useInstanceSettingsDraft } from './use-instance-settings-draft'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const { instance } = injectInstanceSettings()

const globalSettings = (await get().catch(handleError)) as AppSettings

const defaultFullscreen = globalSettings.force_fullscreen
const defaultResolution = globalSettings.game_resolution.slice() as [number, number]

function isResolutionOverridden(resolution: [number, number]) {
	return resolution.some((dimension, index) => Number(dimension) !== defaultResolution[index])
}

const { draft } = useInstanceSettingsDraft(
	{
		fullscreen: instance.value.force_fullscreen ?? defaultFullscreen,
		resolution: instance.value.game_resolution ?? defaultResolution,
	},
	(value) =>
		edit(instance.value.id, {
			force_fullscreen: value.fullscreen !== defaultFullscreen ? value.fullscreen : null,
			game_resolution: isResolutionOverridden(value.resolution) ? value.resolution : null,
		}),
)

const fullscreenOverridden = computed(() => draft.value.fullscreen !== defaultFullscreen)
const widthOverridden = computed(() => Number(draft.value.resolution[0]) !== defaultResolution[0])
const heightOverridden = computed(() => Number(draft.value.resolution[1]) !== defaultResolution[1])

const messages = defineMessages({
	defaultSettingsDescription: {
		id: 'instance.settings.tabs.window.default-settings-description',
		defaultMessage: 'Your instance uses the default window settings unless you override them here.',
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
	<div class="flex flex-col gap-6">
		<DefaultSettingsIntro
			tab="window"
			:description="formatMessage(messages.defaultSettingsDescription)"
		/>
		<div class="flex items-center gap-4 justify-between">
			<div class="flex flex-col gap-1">
				<h2 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.fullscreen) }}
				</h2>
				<p class="m-0">
					{{ formatMessage(messages.fullscreenDescription) }}
				</p>
			</div>
			<div class="flex shrink-0 items-center gap-2">
				<Toggle id="fullscreen" v-model="draft.fullscreen" />
				<ResetToDefaultButton
					:disabled="!fullscreenOverridden"
					:setting="formatMessage(messages.fullscreen)"
					@reset="draft.fullscreen = defaultFullscreen"
				/>
			</div>
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
			<div class="flex shrink-0 items-center gap-2">
				<Input
					id="width"
					v-model="draft.resolution[0]"
					autocomplete="off"
					:disabled="draft.fullscreen"
					type="number"
					:placeholder="formatMessage(messages.enterWidth)"
				/>
				<ResetToDefaultButton
					:disabled="!widthOverridden"
					:setting="formatMessage(messages.width)"
					@reset="draft.resolution[0] = defaultResolution[0]"
				/>
			</div>
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
			<div class="flex shrink-0 items-center gap-2">
				<Input
					id="height"
					v-model="draft.resolution[1]"
					autocomplete="off"
					:disabled="draft.fullscreen"
					type="number"
					:placeholder="formatMessage(messages.enterHeight)"
				/>
				<ResetToDefaultButton
					:disabled="!heightOverridden"
					:setting="formatMessage(messages.height)"
					@reset="draft.resolution[1] = defaultResolution[1]"
				/>
			</div>
		</div>
	</div>
</template>
