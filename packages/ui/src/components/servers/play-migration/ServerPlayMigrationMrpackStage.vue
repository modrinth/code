<template>
	<div class="flex w-full flex-col gap-6">
		<Admonition type="circle-warning">
			<IntlFormatted :message-id="messages.warning">
				<template #exact="{ children }">
					<span class="underline"><component :is="() => children" /></span>
				</template>
			</IntlFormatted>
		</Admonition>
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">{{ formatMessage(messages.fileLabel) }}</span>
			<ArchiveInput
				v-model="file"
				v-model:processing="processing"
				accept=".mrpack"
				:local-files="false"
			/>
			<p class="m-0 leading-6 text-primary">{{ formatMessage(messages.fileDescription) }}</p>
		</div>
		<div class="flex flex-wrap items-center justify-end gap-2">
			<Button type="outlined" size="lg" @click="emit('back')">
				<ChevronLeftIcon aria-hidden="true" />
				{{ formatMessage(commonMessages.backButton) }}
			</Button>
			<Button
				type="colored"
				color="brand"
				size="lg"
				:disabled="!mrpack || processing || disabled"
				@click="confirm"
			>
				<RotateCounterClockwiseIcon aria-hidden="true" />
				{{ formatMessage(serverPlayMigrationMessages.confirm) }}
			</Button>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ChevronLeftIcon, RotateCounterClockwiseIcon } from '@modrinth/assets'
import { computed, ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import { Button } from '#ui/components/base/buttons'
import IntlFormatted from '#ui/components/base/IntlFormatted.vue'
import ArchiveInput from '#ui/components/flows/creation-flow-modal/components/curseforge-stage/archive-input.vue'
import type { CurseForgePackFile } from '#ui/components/flows/creation-flow-modal/creation-flow-context'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import { serverPlayMigrationMessages } from './messages'

defineProps<{
	disabled?: boolean
}>()

const emit = defineEmits<{
	back: []
	confirm: [file: File]
}>()

const { formatMessage } = useVIntl()
const file = ref<CurseForgePackFile | null>(null)
const processing = ref(false)
const mrpack = computed(() => (file.value instanceof File ? file.value : null))

const messages = defineMessages({
	warning: {
		id: 'servers.play-migration.mrpack.warning',
		defaultMessage:
			'It looks like your server was set up using an external .mrpack. You’ll need to upload the <exact>exact</exact> same pack to avoid corrupting your world.',
	},
	fileLabel: {
		id: 'servers.play-migration.mrpack.file-label',
		defaultMessage: '.mrpack file',
	},
	fileDescription: {
		id: 'servers.play-migration.mrpack.file-description',
		defaultMessage:
			'The pack can include client-side content, like mods and resource packs, so we can share it with your players.',
	},
})

function confirm() {
	if (mrpack.value) emit('confirm', mrpack.value)
}
</script>
