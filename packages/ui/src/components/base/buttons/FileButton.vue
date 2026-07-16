<script setup lang="ts">
import { fileIsValid } from '@modrinth/utils'

import { useFormatBytes } from '../../../composables'
import ButtonFrame from './ButtonFrame.vue'
import type { ButtonSize, ButtonTone, ButtonVariant } from './types'

const props = withDefaults(
	defineProps<{
		prompt?: string
		multiple?: boolean
		accept?: string
		maxSize?: number | null
		disabled?: boolean
		allowDrop?: boolean
		variant?: ButtonVariant
		tone?: ButtonTone
		size?: ButtonSize
	}>(),
	{
		prompt: 'Select file',
		multiple: false,
		accept: undefined,
		maxSize: undefined,
		disabled: false,
		allowDrop: true,
		variant: 'base',
		size: 'default',
	},
)

const emit = defineEmits<{
	change: [files: File[]]
}>()

const formatBytes = useFormatBytes()

function selectFiles(incoming: FileList) {
	if (props.disabled) return

	const validationOptions = { maxSize: props.maxSize, alertOnInvalid: true }
	const validFiles = Array.from(incoming).filter((file) =>
		fileIsValid(file, validationOptions, formatBytes),
	)

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
		:variant="props.variant"
		:tone="props.tone"
		:size="props.size"
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
