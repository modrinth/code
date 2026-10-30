<script setup lang="ts">
import { InfoIcon } from '@modrinth/assets'
import { FloatingMenu, IconButton, type ImageViewerEditorMetadata } from '@modrinth/ui'
import { computed, ref } from 'vue'

import { formatBytes } from './format'

const props = defineProps<{
	metadata?: ImageViewerEditorMetadata
}>()

const menu = ref<InstanceType<typeof FloatingMenu>>()
const open = ref(false)

const format = computed(() => {
	const mime = props.metadata?.mimeType.toLowerCase().split(';')[0]?.trim() ?? ''
	const formats: Record<string, string> = {
		'image/jpeg': 'JPEG',
		'image/png': 'PNG',
		'image/gif': 'GIF',
		'image/webp': 'WebP',
		'image/avif': 'AVIF',
		'image/svg+xml': 'SVG',
		'image/bmp': 'BMP',
		'image/x-icon': 'ICO',
		'image/vnd.microsoft.icon': 'ICO',
	}
	return formats[mime] ?? (mime.startsWith('image/') ? mime.slice(6).toUpperCase() : '')
})
</script>

<template>
	<FloatingMenu
		ref="menu"
		bare
		trigger="hover"
		placement="top"
		:disabled="!metadata"
		@open="open = true"
		@close="open = false"
	>
		<IconButton
			label="Image information"
			type="quiet"
			class="!cursor-default !bg-transparent"
			:aria-expanded="open"
			:disabled="!metadata"
			@click="menu?.show()"
			@keydown.escape.stop="menu?.hide()"
		>
			<InfoIcon aria-hidden="true" />
		</IconButton>
		<template #popper>
			<section
				aria-label="Image information"
				class="w-64 max-w-[calc(100vw_-_2rem)] rounded-xl border border-solid border-surface-5 bg-surface-3 p-4 text-sm text-primary shadow-2xl"
				@keydown.escape.stop="menu?.hide()"
			>
				<dl v-if="metadata" class="m-0 grid grid-cols-[auto_1fr] gap-x-4 gap-y-2">
					<dt class="text-secondary">Dimensions</dt>
					<dd class="m-0 text-right tabular-nums">
						{{ metadata.width }} × {{ metadata.height }} px
					</dd>
					<dt class="text-secondary">File size</dt>
					<dd class="m-0 text-right tabular-nums">
						{{ formatBytes(metadata.size) }}
					</dd>
					<dt class="text-secondary">Format</dt>
					<dd class="m-0 text-right">{{ format || 'Unknown' }}</dd>
				</dl>
			</section>
		</template>
	</FloatingMenu>
</template>
