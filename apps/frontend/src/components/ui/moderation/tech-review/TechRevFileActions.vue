<script setup lang="ts">
import { DownloadIcon, ExternalIcon } from '@modrinth/assets'
import { ButtonLink, useFormatBytes } from '@modrinth/ui'

import type { FlattenedFileReport } from './types'

defineProps<{
	file: FlattenedFileReport
}>()

const formatBytes = useFormatBytes()
</script>

<template>
	<div class="flex items-center gap-2">
		<ButtonLink
			v-tooltip="`Download ${file.file_name} (${formatBytes(file.file_size)})`"
			type="outlined"
			target="_blank"
			:href="file.download_url"
			:download="file.file_name"
			aria-label="Download"
			icon-only
			circular
		>
			<DownloadIcon aria-hidden="true" />
		</ButtonLink>
		<ButtonLink
			v-tooltip="'Open in Slicer'"
			type="outlined"
			:target="file.file_id"
			:href="`https://slicer.run/?url=${encodeURIComponent(file.download_url)}`"
			aria-label="Open in Slicer"
		>
			<ExternalIcon aria-hidden="true" /> Slicer
		</ButtonLink>
	</div>
</template>
