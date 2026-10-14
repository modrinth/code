<script setup lang="ts">
import { SpinnerIcon } from '@modrinth/assets'
import { Button, commonMessages, defineMessages, Input, Toggle, useVIntl } from '@modrinth/ui'

import { useDefaultInstanceSettings } from '@/composables/use-default-instance-settings'

const { formatMessage } = useVIntl()
const { settings, settingsQuery } = useDefaultInstanceSettings(
	(value) => ({
		force_fullscreen: value.force_fullscreen,
		game_resolution: [...value.game_resolution] as [number, number],
	}),
	(value) => {
		if (!value.game_resolution.every((dimension) => Number.isInteger(dimension) && dimension > 0)) {
			return null
		}
		return {
			force_fullscreen: value.force_fullscreen,
			game_resolution: [...value.game_resolution],
		}
	},
)

const messages = defineMessages({
	fullscreenTitle: {
		id: 'app.settings.default-instance-options.fullscreen.title',
		defaultMessage: 'Fullscreen',
	},
	fullscreenDescription: {
		id: 'app.settings.default-instance-options.fullscreen.description',
		defaultMessage: 'Start instances in fullscreen by updating their options.txt file.',
	},
	widthTitle: {
		id: 'app.settings.default-instance-options.width.title',
		defaultMessage: 'Width',
	},
	widthDescription: {
		id: 'app.settings.default-instance-options.width.description',
		defaultMessage: 'The width of the game window when launched.',
	},
	widthPlaceholder: {
		id: 'app.settings.default-instance-options.width.placeholder',
		defaultMessage: 'Enter width...',
	},
	heightTitle: {
		id: 'app.settings.default-instance-options.height.title',
		defaultMessage: 'Height',
	},
	heightDescription: {
		id: 'app.settings.default-instance-options.height.description',
		defaultMessage: 'The height of the game window when launched.',
	},
	heightPlaceholder: {
		id: 'app.settings.default-instance-options.height.placeholder',
		defaultMessage: 'Enter height...',
	},
})
</script>

<template>
	<div class="flex flex-col gap-6">
		<template v-if="settings">
			<div class="flex items-center justify-between gap-4">
				<div class="flex flex-col gap-1">
					<h2 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.fullscreenTitle) }}
					</h2>
					<p class="m-0 leading-tight">
						{{ formatMessage(messages.fullscreenDescription) }}
					</p>
				</div>
				<Toggle
					id="fullscreen"
					v-model="settings.force_fullscreen"
					:aria-label="formatMessage(messages.fullscreenTitle)"
				/>
			</div>
			<div class="flex items-center justify-between gap-4">
				<div class="flex flex-col gap-1">
					<h2 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.widthTitle) }}
					</h2>
					<p class="m-0 leading-tight">
						{{ formatMessage(messages.widthDescription) }}
					</p>
				</div>
				<Input
					id="width"
					v-model="settings.game_resolution[0]"
					:aria-label="formatMessage(messages.widthTitle)"
					:disabled="settings.force_fullscreen"
					autocomplete="off"
					type="number"
					:placeholder="formatMessage(messages.widthPlaceholder)"
				/>
			</div>
			<div class="flex items-center justify-between gap-4">
				<div class="flex flex-col gap-1">
					<h2 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.heightTitle) }}
					</h2>
					<p class="m-0 leading-tight">
						{{ formatMessage(messages.heightDescription) }}
					</p>
				</div>
				<Input
					id="height"
					v-model="settings.game_resolution[1]"
					:aria-label="formatMessage(messages.heightTitle)"
					:disabled="settings.force_fullscreen"
					autocomplete="off"
					type="number"
					:placeholder="formatMessage(messages.heightPlaceholder)"
				/>
			</div>
		</template>
		<Button v-else-if="settingsQuery.isError.value" @click="settingsQuery.refetch()">
			{{ formatMessage(commonMessages.refreshButton) }}
		</Button>
		<div v-else class="flex items-center gap-2 text-secondary">
			<SpinnerIcon class="size-5 animate-spin" aria-hidden="true" />
			{{ formatMessage(commonMessages.loadingLabel) }}
		</div>
	</div>
</template>
