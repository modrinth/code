<script setup lang="ts">
import { defineMessages, useFormatBytes, useVIntl } from '../../../composables'
import { injectNotificationManager } from '../../../providers'
import ButtonFrame from './ButtonFrame.vue'
import type { ButtonColor, ButtonInteraction, ButtonSize, ButtonType } from './types'

const props = withDefaults(
	defineProps<{
		prompt?: string
		multiple?: boolean
		accept?: string
		maxSize?: number | null
		disabled?: boolean
		allowDrop?: boolean
		type?: ButtonType
		color?: ButtonColor
		size?: ButtonSize
		interaction?: ButtonInteraction
	}>(),
	{
		prompt: 'Select file',
		multiple: false,
		accept: undefined,
		maxSize: undefined,
		disabled: false,
		allowDrop: true,
		type: 'base',
		size: 'md',
	},
)

const emit = defineEmits<{
	change: [files: File[]]
}>()

const formatBytes = useFormatBytes()
const { formatMessage } = useVIntl()
const notificationManager = injectNotificationManager()
const messages = defineMessages({
	fileTooLarge: {
		id: 'file-button.file-too-large.title',
		defaultMessage: 'File too large',
	},
	fileTooLargeDescription: {
		id: 'file-button.file-too-large.description',
		defaultMessage: 'File {filename} is too big. The maximum file size is {maxSize}.',
	},
})

function selectFiles(incoming: FileList) {
	if (props.disabled) return

	const validFiles = Array.from(incoming).filter((file) => {
		if (props.maxSize != null && file.size > props.maxSize) {
			notificationManager.addNotification({
				type: 'error',
				title: formatMessage(messages.fileTooLarge),
				text: formatMessage(messages.fileTooLargeDescription, {
					filename: file.name,
					maxSize: formatBytes(props.maxSize),
				}),
			})
			return false
		}
		return true
	})

	if (validFiles.length > 0) emit('change', validFiles)
}

function handleChange(event: Event) {
	const input = event.target as HTMLInputElement
	if (input.files) selectFiles(input.files)
	input.value = ''
}

function handleDrop(event: DragEvent) {
	if (!props.allowDrop || !event.dataTransfer) return
	selectFiles(event.dataTransfer.files)
}
</script>

<template>
	<ButtonFrame
		as="label"
		:type="props.type"
		:color="props.color"
		:size="props.size"
		:interaction="props.interaction"
		:aria-disabled="props.disabled || undefined"
		class="focus-within:outline-none focus-within:ring-4 focus-within:ring-brand-shadow"
		@drop.prevent="handleDrop"
		@dragover.prevent
	>
		<slot />
		{{ props.prompt }}
		<input
			type="file"
			:multiple="props.multiple"
			:accept="props.accept"
			:disabled="props.disabled"
			class="absolute size-px overflow-hidden whitespace-nowrap [clip:rect(0,0,0,0)]"
			@change="handleChange"
		/>
	</ButtonFrame>
</template>
