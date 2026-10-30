<template>
	<div class="flex w-full flex-col gap-6">
		<Admonition type="circle-warning">
			<IntlFormatted :message-id="messages.warning">
				<template #exact="{ children }">
					<span class="underline"><component :is="() => children" /></span>
				</template>
			</IntlFormatted>
		</Admonition>
		<ModpackPicker />
		<div class="flex flex-wrap items-center justify-end gap-2">
			<Button
				type="outlined"
				size="lg"
				:disabled="selection.curseforgeProcessing.value"
				@click="emit('back')"
			>
				<ChevronLeftIcon aria-hidden="true" />
				{{ formatMessage(commonMessages.backButton) }}
			</Button>
			<Button
				v-if="picked"
				type="colored"
				color="brand"
				size="lg"
				:disabled="!isCurseForgeSelectionReady(selection) || disabled"
				@click="emit('confirm')"
			>
				<RotateCounterClockwiseIcon aria-hidden="true" />
				{{ formatMessage(serverPlayMigrationMessages.confirm) }}
			</Button>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ChevronLeftIcon, RotateCounterClockwiseIcon } from '@modrinth/assets'
import { computed } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import { Button } from '#ui/components/base/buttons'
import IntlFormatted from '#ui/components/base/IntlFormatted.vue'
import ModpackPicker from '#ui/components/flows/creation-flow-modal/components/curseforge-stage/modpack-picker.vue'
import {
	type CurseForgeSelection,
	isCurseForgeSelectionReady,
	provideCurseForgeSelection,
} from '#ui/components/flows/creation-flow-modal/curseforge'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import { serverPlayMigrationMessages } from './messages'

const { selection } = defineProps<{
	selection: CurseForgeSelection
	disabled?: boolean
}>()

const emit = defineEmits<{
	back: []
	confirm: []
}>()

const { formatMessage } = useVIntl()
provideCurseForgeSelection(selection)

const picked = computed(
	() => !!selection.curseforgeModpackFile.value || !!selection.curseforgeModpack.value,
)

const messages = defineMessages({
	warning: {
		id: 'servers.play-migration.curseforge.warning',
		defaultMessage:
			'It looks like your server was set up using a CurseForge modpack. You’ll need to upload the <exact>exact</exact> same pack again to avoid corrupting your world.',
	},
})
</script>
