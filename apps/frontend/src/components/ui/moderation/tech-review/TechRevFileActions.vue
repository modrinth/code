<script setup lang="ts">
import { DownloadIcon, ExternalIcon, FolderArchiveIcon } from '@modrinth/assets'
import { Button, ButtonLink, defineMessages, useFormatBytes, useVIntl } from '@modrinth/ui'

import { archiveViewerUrl } from '~/helpers/archive-viewer'
import { injectReviewArchive } from '~/providers/project-review/review-archive'

import type { FlattenedFileReport } from './types'

defineProps<{
	file: FlattenedFileReport
}>()

const archive = injectReviewArchive(null)
const formatBytes = useFormatBytes()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	download: { id: 'moderation.tech-review.file.download', defaultMessage: 'Download' },
	downloadFile: {
		id: 'moderation.tech-review.file.download-file',
		defaultMessage: 'Download {filename} ({size})',
	},
	archive: { id: 'moderation.tech-review.file.archive', defaultMessage: 'Archive' },
	openArchive: {
		id: 'moderation.tech-review.file.open-archive',
		defaultMessage: 'Open in the archive viewer',
	},
	openArchivePanel: {
		id: 'moderation.tech-review.file.open-archive-panel',
		defaultMessage: "Open in the review's archive panels",
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
			v-if="archive"
			v-tooltip="formatMessage(messages.openArchivePanel)"
			:aria-label="formatMessage(messages.openArchivePanel)"
			type="outlined"
			icon-only
			circular
			@click="archive.open(file.download_url)"
		>
			<FolderArchiveIcon aria-hidden="true" />
		</Button>
		<ButtonLink
			v-tooltip="formatMessage(messages.openArchive)"
			type="outlined"
			:target="file.file_id"
			:href="archiveViewerUrl({ url: file.download_url })"
		>
			<ExternalIcon aria-hidden="true" /> {{ formatMessage(messages.archive) }}
		</ButtonLink>
	</div>
</template>
