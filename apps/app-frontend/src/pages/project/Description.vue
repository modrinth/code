<template>
	<Card>
		<div class="markdown-body" v-html="descriptionHtml" />
	</Card>
</template>

<script setup>
import { Card } from '@modrinth/ui'
import { renderHighlightedString } from '@modrinth/utils'
import { invoke } from '@tauri-apps/api/core'
import { computedAsync } from '@vueuse/core'

const props = defineProps({
	project: {
		type: Object,
		default: () => {},
	},
})

const youtubeFrame =
	/(<iframe\b[^>]*\bsrc=")https?:\/\/(?:www\.)?youtube(?:-nocookie)?\.com\/embed\/([a-zA-Z0-9_-]{11})(?=[?#"])/g
const descriptionHtml = computedAsync(async () => {
	const html = renderHighlightedString(props.project?.body ?? '')
	if (!html.match(youtubeFrame)) return html
	try {
		const origin = await invoke('youtube_embed_origin')
		return html.replace(youtubeFrame, `$1${origin}/$2`)
	} catch (error) {
		console.error('Failed to initialize YouTube embeds', error)
		return html
	}
}, '')
</script>

<script>
export default {
	name: 'Description',
}
</script>
