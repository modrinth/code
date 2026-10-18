<script setup lang="ts">
import { CoffeeIcon, DownloadIcon, ExternalIcon } from '@modrinth/assets'
import { Button, ButtonLink, defineMessages, useFormatBytes, useVIntl } from '@modrinth/ui'

import { injectReviewSlicer } from '~/providers/project-review/review-slicer'

import type { FlattenedFileReport } from './types'

defineProps<{
	file: FlattenedFileReport
}>()

const slicer = injectReviewSlicer(null)
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
	openSlicerTab: {
		id: 'moderation.tech-review.file.open-slicer-tab',
		defaultMessage: 'Open in Slicer tab',
	},
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
		<Button
			v-if="slicer"
			v-tooltip="formatMessage(messages.openSlicerTab)"
			:aria-label="formatMessage(messages.openSlicerTab)"
			type="outlined"
			icon-only
			circular
			@click="slicer.open([file.download_url])"
		>
			<CoffeeIcon aria-hidden="true" />
		</Button>
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
