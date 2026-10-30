<script setup lang="ts">
import { DownloadIcon, ExternalIcon, GridIcon } from '@modrinth/assets'
import {
	ButtonLink,
	IconButton,
	type ImageViewerEditorMetadata,
	injectNotificationManager,
} from '@modrinth/ui'
import { ref } from 'vue'

import ImageViewerInfo from './image-viewer-info.vue'

const props = defineProps<{
	src: string
	metadata?: ImageViewerEditorMetadata
}>()
const pixelated = defineModel<boolean>('pixelated', { default: false })
const downloading = ref(false)

const { handleError } = injectNotificationManager()

async function downloadImage() {
	if (downloading.value) return
	const src = props.src
	downloading.value = true
	try {
		const response = await fetch(src)
		if (!response.ok) throw new Error(`Could not load image: ${response.statusText}`)
		const url = URL.createObjectURL(await response.blob())
		const link = document.createElement('a')
		link.href = url
		link.download = new URL(src, window.location.href).pathname.split('/').pop() || 'image'
		document.body.appendChild(link)
		link.click()
		link.remove()
		setTimeout(() => URL.revokeObjectURL(url), 1000)
	} catch (error) {
		handleError(error)
	} finally {
		downloading.value = false
	}
}
</script>

<template>
	<IconButton
		v-tooltip="pixelated ? 'Disable pixel-art rendering' : 'Enable pixel-art rendering'"
		:label="pixelated ? 'Disable pixel-art rendering' : 'Enable pixel-art rendering'"
		:type="pixelated ? 'colored' : 'quiet'"
		:color="pixelated ? 'brand' : undefined"
		:aria-pressed="pixelated"
		@click="pixelated = !pixelated"
	>
		<GridIcon aria-hidden="true" />
	</IconButton>
	<IconButton
		v-tooltip="'Download image'"
		label="Download image"
		type="quiet"
		:loading="downloading"
		@click="downloadImage"
	>
		<DownloadIcon aria-hidden="true" />
	</IconButton>
	<ButtonLink
		v-tooltip="'Open image in new tab'"
		type="quiet"
		class="!w-9 !rounded-full !p-0"
		aria-label="Open image in new tab"
		:href="src"
		target="_blank"
	>
		<ExternalIcon aria-hidden="true" />
	</ButtonLink>
	<ImageViewerInfo :key="src" :metadata="metadata" />
</template>
