<template>
	<Teleport to="body">
		<div
			ref="dropAreaRef"
			class="drop-area"
			@drop.stop.prevent="handleDrop"
			@dragenter.prevent="allowDrag"
			@dragover.prevent="allowDrag"
			@dragleave.prevent="hideDropArea"
		/>
	</Teleport>
	<slot />
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'

import { injectNotificationManager } from '../../providers'

const { addNotification } = injectNotificationManager()

const props = withDefaults(
	defineProps<{
		accept: string
	}>(),
	{
		accept: '*',
	},
)

const emit = defineEmits(['change'])

const dropAreaRef = ref<HTMLDivElement>()

const hideDropArea = () => {
	if (dropAreaRef.value) {
		dropAreaRef.value.style.visibility = 'hidden'
	}
}

const handleDrop = (event: DragEvent) => {
	event.preventDefault()
	hideDropArea()

	const files = event.dataTransfer?.files
	if (!files || files.length === 0) return

	const file = files[0]

	if (!matchesAccept({ getAsFile: () => file } as DataTransferItem, props.accept)) {
		addNotification({
			title: 'Invalid file',
			text: `The file "${file.name}" is not a valid file type for this project.`,
			type: 'error',
		})
		return
	}

	emit('change', files)
}

function matchesAccept(file: DataTransferItem, accept?: string): boolean {
	if (!accept || accept.trim() === '') return true

	const fileType = file.type // e.g. "image/png"
	const fileName = file.getAsFile()?.name.toLowerCase() ?? ''

	return accept
		.split(',')
		.map((t) => t.trim().toLowerCase())
		.some((token) => {
			// .png, .jpg
			if (token.startsWith('.')) {
				return fileName.endsWith(token)
			}

			// image/*
			if (token.endsWith('/*')) {
				const base = token.slice(0, -1) // "image/"
				return fileType.startsWith(base)
			}

			// image/png
			return fileType === token
		})
}

const allowDrag = (event: DragEvent) => {
	const item = event.dataTransfer?.items?.[0]
	if (!item || item.kind !== 'file') return

	event.preventDefault()
	event.dataTransfer!.dropEffect = 'copy'

	if (dropAreaRef.value) {
		dropAreaRef.value.style.visibility = 'visible'
	}
}

onMounted(() => {
	document.addEventListener('dragenter', allowDrag)
})
</script>

<style lang="scss" scoped>
.drop-area {
	position: fixed;
	top: 0;
	left: 0;
	width: 100%;
	height: 100%;
	z-index: 10;
	visibility: hidden;
	background-color: hsla(0, 0%, 0%, 0.5);
	transition:
		visibility 0.2s ease-in-out,
		background-color 0.1s ease-in-out;
	display: flex;
	&::before {
		--indent: 4rem;
		content: ' ';
		position: relative;
		top: var(--indent);
		left: var(--indent);
		width: calc(100% - (2 * var(--indent)));
		height: calc(100% - (2 * var(--indent)));
		border-radius: 1rem;
		border: 0.25rem dashed var(--color-button-bg);
	}

	@media (prefers-reduced-motion) {
		transition: none !important;
	}
}
</style>
