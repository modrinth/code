<script setup lang="ts">
import { InfoIcon } from '@modrinth/assets'
import {
	defineMessages,
	FloatingMenu,
	IconButton,
	type ImageViewerEditorMetadata,
	useFormatBytes,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

const props = defineProps<{
	metadata?: ImageViewerEditorMetadata
}>()

const { formatMessage } = useVIntl()
const formatBytes = useFormatBytes()
const menu = ref<InstanceType<typeof FloatingMenu>>()
const open = ref(false)

const messages = defineMessages({
	info: {
		id: 'project-review.image.info',
		defaultMessage: 'Image information',
	},
	dimensions: {
		id: 'project-review.image.dimensions',
		defaultMessage: 'Dimensions',
	},
	size: {
		id: 'project-review.image.file-size',
		defaultMessage: 'File size',
	},
	format: {
		id: 'project-review.image.format',
		defaultMessage: 'Format',
	},
	unknown: {
		id: 'project-review.image.info-unknown',
		defaultMessage: 'Unknown',
	},
})

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
			:label="formatMessage(messages.info)"
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
				:aria-label="formatMessage(messages.info)"
				class="w-64 max-w-[calc(100vw_-_2rem)] rounded-xl border border-solid border-surface-5 bg-surface-3 p-4 text-sm text-primary shadow-2xl"
				@keydown.escape.stop="menu?.hide()"
			>
				<dl v-if="metadata" class="m-0 grid grid-cols-[auto_1fr] gap-x-4 gap-y-2">
					<dt class="text-secondary">{{ formatMessage(messages.dimensions) }}</dt>
					<dd class="m-0 text-right tabular-nums">
						{{ metadata.width }} × {{ metadata.height }} px
					</dd>
					<dt class="text-secondary">{{ formatMessage(messages.size) }}</dt>
					<dd class="m-0 text-right tabular-nums">{{ formatBytes(metadata.size) }}</dd>
					<dt class="text-secondary">{{ formatMessage(messages.format) }}</dt>
					<dd class="m-0 text-right">{{ format || formatMessage(messages.unknown) }}</dd>
				</dl>
			</section>
		</template>
	</FloatingMenu>
</template>
