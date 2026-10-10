<script setup lang="ts">
import { DownloadIcon, ExternalIcon, GridIcon } from '@modrinth/assets'
import {
	ButtonLink,
	defineMessages,
	IconButton,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { ref } from 'vue'

import { projectReviewMessages } from './messages'

const props = defineProps<{ src: string }>()
const pixelated = defineModel<boolean>('pixelated', { default: false })
const downloading = ref(false)
const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const messages = defineMessages({
	enablePixelated: {
		id: 'project-review.image.enable-pixelated',
		defaultMessage: 'Enable pixel-art rendering',
	},
	disablePixelated: {
		id: 'project-review.image.disable-pixelated',
		defaultMessage: 'Disable pixel-art rendering',
	},
	download: {
		id: 'project-review.image.download',
		defaultMessage: 'Download image',
	},
})

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
		v-tooltip="formatMessage(pixelated ? messages.disablePixelated : messages.enablePixelated)"
		:label="formatMessage(pixelated ? messages.disablePixelated : messages.enablePixelated)"
		:type="pixelated ? 'colored' : 'quiet'"
		:color="pixelated ? 'brand' : undefined"
		:aria-pressed="pixelated"
		@click="pixelated = !pixelated"
	>
		<GridIcon aria-hidden="true" />
	</IconButton>
	<IconButton
		v-tooltip="formatMessage(messages.download)"
		:label="formatMessage(messages.download)"
		type="quiet"
		:loading="downloading"
		@click="downloadImage"
	>
		<DownloadIcon aria-hidden="true" />
	</IconButton>
	<ButtonLink
		v-tooltip="formatMessage(projectReviewMessages.openImageInNewTab)"
		type="quiet"
		class="!w-9 !rounded-full !p-0"
		:aria-label="formatMessage(projectReviewMessages.openImageInNewTab)"
		:href="src"
		target="_blank"
	>
		<ExternalIcon aria-hidden="true" />
	</ButtonLink>
</template>
