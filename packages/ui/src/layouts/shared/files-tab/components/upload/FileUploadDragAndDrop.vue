<template>
	<div
		ref="dropTargetRef"
		@dragenter="dropTargetProps.onDragenter"
		@dragover="dropTargetProps.onDragover"
		@dragleave="dropTargetProps.onDragleave"
		@drop="dropTargetProps.onDrop"
	>
		<slot />
		<div
			v-if="showOverlay"
			:class="[
				'absolute inset-0 flex items-center justify-center rounded-2xl bg-black/60 text-contrast shadow',
				props.overlayClass,
			]"
		>
			<div class="text-center">
				<UploadIcon class="mx-auto h-16 w-16 shadow-2xl" />
				<p class="mt-2 text-xl">
					{{
						formatMessage(messages.dropToUpload, {
							type: formatFileItemType(formatMessage, props.type?.toLocaleLowerCase(), true),
						})
					}}
				</p>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { UploadIcon } from '@modrinth/assets'
import { computed, ref } from 'vue'

import { useFileDropTarget } from '#ui/composables/file-drop'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { formatFileItemType } from '#ui/utils/common-messages'

const { formatMessage } = useVIntl()

const emit = defineEmits<{
	filesDropped: [files: File[]]
	dropError: [error: unknown]
}>()

const props = defineProps<{
	disabled?: boolean
	overlayClass?: string
	type?: string
}>()

const messages = defineMessages({
	dropToUpload: {
		id: 'files.upload.drag-and-drop.drop-to-upload',
		defaultMessage: 'Drop {type} here to upload',
	},
})

const dropTargetRef = ref<HTMLElement | null>(null)
const { isDragging, dropTargetProps } = useFileDropTarget({
	target: dropTargetRef,
	disabled: computed(() => props.disabled ?? false),
	onFiles: (files) => emit('filesDropped', files),
	onError: (error) => emit('dropError', error),
})
const showOverlay = computed(() => isDragging.value)
</script>
