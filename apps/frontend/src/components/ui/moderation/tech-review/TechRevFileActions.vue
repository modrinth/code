<script setup lang="ts">
import { DownloadIcon, ExternalIcon } from '@modrinth/assets'
import { ButtonLink, defineMessages, useFormatBytes, useVIntl } from '@modrinth/ui'

import type { FlattenedFileReport } from './types'

defineProps<{
	file: FlattenedFileReport
}>()

const formatBytes = useFormatBytes()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	download: { id: 'moderation.tech-review.file.download', defaultMessage: 'Download' },
	downloadFile: {
		id: 'moderation.tech-review.file.download-file',
		defaultMessage: 'Download {filename} ({size})',
	},
	slicer: { id: 'moderation.tech-review.file.slicer', defaultMessage: 'Slicer' },
	openSlicer: { id: 'moderation.tech-review.file.open-slicer', defaultMessage: 'Open in Slicer' },
})
</script>

<template>
	<div class="flex items-center gap-2">
		<ButtonLink
			v-tooltip="
				formatMessage(messages.downloadFile, {
					filename: file.file_name,
					size: formatBytes(file.file_size),
				})
			"
			type="outlined"
			target="_blank"
			:href="file.download_url"
			:download="file.file_name"
			:aria-label="formatMessage(messages.download)"
			icon-only
			circular
		>
			<DownloadIcon aria-hidden="true" />
		</ButtonLink>
		<ButtonLink
			v-tooltip="formatMessage(messages.openSlicer)"
			type="outlined"
			:target="file.file_id"
			:href="`https://slicer.run/?url=${encodeURIComponent(file.download_url)}`"
		>
			<ExternalIcon aria-hidden="true" /> {{ formatMessage(messages.slicer) }}
		</ButtonLink>
	</div>
</template>
