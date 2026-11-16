<script setup lang="ts">
import { BoxIcon, FolderOpenIcon, PlusIcon } from '@modrinth/assets'

import { Button } from '#ui/components/base/buttons'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

defineProps<{
	heading: string
	description: string
	addDisabled: boolean
	addTooltip?: string
}>()

const emit = defineEmits<{
	add: []
	upload: []
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	addContent: {
		id: 'content.page-layout.add-content',
		defaultMessage: 'Add content',
	},
	uploadFiles: {
		id: 'content.page-layout.upload-files',
		defaultMessage: 'Upload files',
	},
})
</script>

<template>
	<div
		class="relative mx-auto flex w-full max-w-[448px] flex-col items-center px-4 py-12 text-center"
	>
		<div
			aria-hidden="true"
			class="dot-pattern pointer-events-none absolute left-1/2 top-[-143px] h-[478px] w-[415px] -translate-x-1/2"
		/>
		<BoxIcon class="relative size-[72px] text-primary" aria-hidden="true" />
		<div class="relative mt-6 flex flex-col items-center gap-2">
			<h2 class="m-0 text-2xl font-semibold leading-8 text-contrast">{{ heading }}</h2>
			<p class="m-0 text-base leading-6 text-primary">{{ description }}</p>
		</div>
		<div class="relative mt-6 flex flex-wrap justify-center gap-2">
			<Button
				v-tooltip="addTooltip"
				type="outlined"
				size="lg"
				:disabled="addDisabled"
				@click="emit('upload')"
			>
				<FolderOpenIcon />
				{{ formatMessage(messages.uploadFiles) }}
			</Button>
			<Button
				v-tooltip="addTooltip"
				type="colored"
				color="brand"
				size="lg"
				:disabled="addDisabled"
				@click="emit('add')"
			>
				<PlusIcon />
				{{ formatMessage(messages.addContent) }}
			</Button>
		</div>
	</div>
</template>

<style scoped>
.dot-pattern {
	background-image: radial-gradient(var(--surface-5) 1px, transparent 1px);
	background-size: 12px 12px;
	mask-image: radial-gradient(closest-side, black, transparent);
}
</style>
