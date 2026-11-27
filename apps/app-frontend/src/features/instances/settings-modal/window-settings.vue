<script setup lang="ts">
import { defineMessages, injectNotificationManager, Input, Toggle, useVIntl } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import { edit } from '@/helpers/instance'
import { appSettingsQueryOptions } from '@/helpers/settings.ts'

import { injectInstanceSettings } from './instance-settings-context'
import SettingsSourceHeader from './settings-source-header.vue'
import { useInstanceSettingsDraft } from './use-instance-settings-draft'
import { useSettingsSource } from './use-settings-source'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const { instance } = injectInstanceSettings()

const draft = useInstanceSettingsDraft<{
	fullscreen: boolean | null
	resolution: [number, number] | null
}>(
	() => ({
		fullscreen: instance.value.force_fullscreen ?? null,
		resolution: instance.value.game_resolution ?? null,
	}),
	(value) =>
		edit(instance.value.id, {
			force_fullscreen: value.fullscreen,
			game_resolution: value.resolution,
		}),
)

const settingsQuery = useQuery(appSettingsQueryOptions())
await settingsQuery.suspense().catch(handleError)

const defaultFullscreen = computed(() => settingsQuery.data.value?.force_fullscreen ?? false)
const defaultResolution = computed(
	() => (settingsQuery.data.value?.game_resolution.slice() ?? [0, 0]) as [number, number],
)

const source = useSettingsSource(draft, () => ({
	fullscreen: defaultFullscreen.value,
	resolution: defaultResolution.value,
}))

const fullscreen = computed(() => draft.value.fullscreen ?? defaultFullscreen.value)
const resolution = computed(() => draft.value.resolution ?? defaultResolution.value)

function setDimension(index: 0 | 1, value: number) {
	const next = resolution.value.slice() as [number, number]
	next[index] = value
	draft.value.resolution = next
}

const messages = defineMessages({
	settingsHeading: {
		id: 'instance.settings.tabs.window.settings-heading',
		defaultMessage: 'Window settings',
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
		<SettingsSourceHeader
			v-model="source"
			tab="window"
			:title="formatMessage(messages.settingsHeading)"
		/>

		<div
			class="flex flex-col gap-6 transition-opacity"
			:class="{ 'opacity-60': source === 'app' }"
			:inert="source === 'app'"
		>
			<div class="flex items-center gap-4 justify-between">
				<div class="flex flex-col gap-1">
					<h2 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.fullscreen) }}
					</h2>
					<p class="m-0">
						{{ formatMessage(messages.fullscreenDescription) }}
					</p>
				</div>
				<Toggle
					id="fullscreen"
					:model-value="fullscreen"
					@update:model-value="(value) => (draft.fullscreen = value)"
				/>
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
					:model-value="resolution[0]"
					autocomplete="off"
					:disabled="fullscreen"
					type="number"
					:placeholder="formatMessage(messages.enterWidth)"
					@update:model-value="(value) => setDimension(0, Number(value))"
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
					:model-value="resolution[1]"
					autocomplete="off"
					:disabled="fullscreen"
					type="number"
					:placeholder="formatMessage(messages.enterHeight)"
					@update:model-value="(value) => setDimension(1, Number(value))"
				/>
			</div>
		</div>
	</div>
</template>
