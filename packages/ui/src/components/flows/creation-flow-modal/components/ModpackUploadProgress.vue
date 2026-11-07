<template>
	<div class="flex flex-col gap-2.5">
		<div class="flex flex-col gap-3">
			<div class="flex items-center justify-between text-contrast">
				<span :id="labelId" class="font-semibold">{{ formatMessage(messages.header) }}</span>
				<span class="font-medium">{{ percent }}%</span>
			</div>
			<div
				role="progressbar"
				:aria-labelledby="labelId"
				:aria-valuenow="percent"
				aria-valuemin="0"
				aria-valuemax="100"
				class="h-2.5 w-full overflow-hidden rounded-full bg-surface-2"
			>
				<div
					class="h-full rounded-full bg-brand transition-[width] duration-200 ease-in-out"
					:style="{ width: `${percent}%` }"
				/>
			</div>
		</div>
		<p class="m-0 leading-6 text-primary">{{ formatMessage(messages.warningText) }}</p>
	</div>
</template>

<script setup lang="ts">
import type { UploadProgress } from '@modrinth/api-client'
import { computed, useId } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'

const props = defineProps<{
	progress: UploadProgress
}>()

const { formatMessage } = useVIntl()
const labelId = useId()

const percent = computed(() => Math.round(Math.min(Math.max(props.progress.progress, 0), 1) * 100))

const messages = defineMessages({
	header: {
		id: 'servers.setup.uploading-modpack.header',
		defaultMessage: 'Uploading modpack',
	},
	warningText: {
		id: 'servers.setup.upload-warning',
		defaultMessage: "Please don't close this page while uploading.",
	},
})
</script>
