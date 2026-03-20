<template>
	<div
		@dragenter.prevent="handleDragEnter"
		@dragover.prevent="handleDragOver"
		@dragleave.prevent="handleDragLeave"
		@drop.prevent="handleDrop"
	>
		<slot />
		<div
			v-if="isDragging"
			:class="[
				'absolute inset-0 flex items-center justify-center rounded-2xl bg-black/60 text-contrast shadow',
				overlayClass,
			]"
		>
			<div class="text-center">
				<UploadIcon class="mx-auto h-16 w-16 shadow-2xl" />
				<p class="mt-2 text-xl">
					{{
						formatMessage(messages.dropToUpload, {
							type: type ? type.toLocaleLowerCase() : undefined,
						})
					}}
				</p>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { UploadIcon } from '@modrinth/assets'
import { ref } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'

const { formatMessage } = useVIntl()

const emit = defineEmits<{
	filesDropped: [files: File[]]
}>()

defineProps<{
	overlayClass?: string
	type?: string
}>()

const messages = defineMessages({
	dropToUpload: {
		id: 'files.upload.drag-and-drop.drop-to-upload',
		defaultMessage: 'Drop {type, select, undefined {files} other {{type}s}} here to upload',
	},
})

const isDragging = ref(false)
const dragCounter = ref(0)

const handleDragEnter = (event: DragEvent) => {
	event.preventDefault()
	if (!event.dataTransfer?.types.includes('application/modrinth-file-move')) {
		dragCounter.value++
		isDragging.value = true
	}
}

const handleDragOver = (event: DragEvent) => {
	event.preventDefault()
}

const handleDragLeave = (event: DragEvent) => {
	event.preventDefault()
	dragCounter.value--
	if (dragCounter.value === 0) {
		isDragging.value = false
	}
}

const handleDrop = (event: DragEvent) => {
	event.preventDefault()
	isDragging.value = false
	dragCounter.value = 0

	const isInternalMove = event.dataTransfer?.types.includes('application/modrinth-file-move')
	if (isInternalMove) return

	const files = event.dataTransfer?.files
	if (files) {
		emit('filesDropped', Array.from(files))
	}
}
</script>
