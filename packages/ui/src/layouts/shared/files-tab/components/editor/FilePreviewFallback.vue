<template>
	<div
		class="flex h-[750px] flex-col items-center justify-center gap-3 overflow-hidden rounded-[20px] bg-surface-2 p-6 text-center"
	>
		<component :is="iconStyle.icon" class="size-12" :class="iconStyle.color" aria-hidden="true" />
		<span class="max-w-full truncate text-lg font-semibold text-contrast">{{ file.name }}</span>
		<p class="m-0 max-w-md text-secondary">{{ formatMessage(messages.description) }}</p>
		<span class="text-sm text-secondary tabular-nums">{{ formatBytes(size) }}</span>
		<Button v-if="ctx.downloadFile" type="outlined" @click="ctx.downloadFile(file)">
			<DownloadIcon aria-hidden="true" />
			{{ formatMessage(commonMessages.downloadButton) }}
		</Button>
	</div>
</template>

<script setup lang="ts">
import { DownloadIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { Button } from '#ui/components/base/buttons'
import { useFormatBytes } from '#ui/composables/format-bytes'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import { type FileInfo, injectFileManager } from '../../providers/file-manager'
import { fileIconFor } from '../../utils'

/** Stands in for a file whose contents can't be shown, e.g. a binary one. */
const props = defineProps<{
	file: FileInfo<'file'>
	size: number
}>()

const { formatMessage } = useVIntl()
const formatBytes = useFormatBytes()
const ctx = injectFileManager()

const messages = defineMessages({
	description: {
		id: 'files.preview-fallback.description',
		defaultMessage: "This file isn't text, an image or media, so it can't be previewed here.",
	},
})

const iconStyle = computed(() => fileIconFor({ name: props.file.name, type: 'file' }))
</script>
