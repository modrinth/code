<script setup lang="ts">
import { defineMessages, injectNotificationManager, Input, Toggle, useVIntl } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import { appSettingsQueryOptions } from '@/helpers/settings.ts'

import SettingsSourceHeader from './settings-source-header.vue'
import { useSettingsSource } from './use-settings-source'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const settingsQuery = useQuery(appSettingsQueryOptions())
await settingsQuery.suspense().catch(handleError)

const defaultFullscreen = computed(() => settingsQuery.data.value?.force_fullscreen ?? false)
const defaultResolution = computed(
	() => (settingsQuery.data.value?.game_resolution.slice() ?? [0, 0]) as [number, number],
)

const {
	source,
	values: windowSettings,
	update,
	debouncedField,
} = useSettingsSource<{
	fullscreen: boolean | null
	resolution: [number, number] | null
}>({
	read: (current) => ({
		fullscreen: current.force_fullscreen ?? null,
		resolution: current.game_resolution ?? null,
	}),
	write: (value) => ({
		force_fullscreen: value.fullscreen,
		game_resolution: value.resolution,
	}),
	defaults: () => ({
		fullscreen: defaultFullscreen.value,
		resolution: defaultResolution.value,
	}),
})

const fullscreen = computed(() => windowSettings.value.fullscreen ?? defaultFullscreen.value)
const resolution = computed(() => windowSettings.value.resolution ?? defaultResolution.value)

function isDimension(value: number | undefined): value is number {
	return typeof value === 'number' && Number.isFinite(value)
}

function saveResolution() {
	if (!isDimension(width.model) || !isDimension(height.model)) return
	update({ resolution: [width.model, height.model] })
}

const width = debouncedField<number | undefined>(() => resolution.value[0], saveResolution)
const height = debouncedField<number | undefined>(() => resolution.value[1], saveResolution)

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
					@update:model-value="(value) => update({ fullscreen: value })"
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
					:model-value="width.model"
					autocomplete="off"
					:disabled="fullscreen"
					type="number"
					:placeholder="formatMessage(messages.enterWidth)"
					@focus="width.onFocus"
					@blur="width.onBlur"
					@update:model-value="(value) => (width.model = typeof value === 'number' ? value : undefined)"
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
					:model-value="height.model"
					autocomplete="off"
					:disabled="fullscreen"
					type="number"
					:placeholder="formatMessage(messages.enterHeight)"
					@focus="height.onFocus"
					@blur="height.onBlur"
					@update:model-value="(value) => (height.model = typeof value === 'number' ? value : undefined)"
				/>
			</div>
		</div>
	</div>
</template>
