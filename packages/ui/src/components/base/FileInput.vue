<template>
	<label :class="{ 'long-style': longStyle }" @drop.prevent="handleDrop" @dragover.prevent>
		<slot />
		{{ prompt }}
		<input
			type="file"
			:multiple="multiple"
			:accept="accept"
			:disabled="disabled"
			@change="handleChange"
		/>
	</label>
</template>

<script setup lang="ts">
import { fileIsValid } from '@modrinth/utils'
import { ref } from 'vue'

import { useFormatBytes } from '../../composables'

const props = withDefaults(
	defineProps<{
		prompt?: string
		multiple?: boolean
		accept?: string
		/**
		 * The max file size in bytes
		 */
		maxSize?: number | null
		showIcon?: boolean
		shouldAlwaysReset?: boolean
		longStyle?: boolean
		disabled?: boolean
	}>(),
	{
		prompt: 'Select file',
		multiple: false,
		showIcon: true,
		shouldAlwaysReset: false,
		longStyle: false,
		disabled: false,
	},
)

const emit = defineEmits<{ change: [files: File[]] }>()

const formatBytes = useFormatBytes()

const files = ref<File[]>([])

function addFiles(incoming: FileList, shouldNotReset = false) {
	if (!shouldNotReset || props.shouldAlwaysReset) {
		files.value = Array.from(incoming)
	}
	const validationOptions = { maxSize: props.maxSize, alertOnInvalid: true }
	files.value = files.value.filter((file) => fileIsValid(file, validationOptions, formatBytes))
	if (files.value.length > 0) {
		emit('change', files.value)
	}
}

function handleDrop(e: DragEvent) {
	addFiles(e.dataTransfer!.files)
}

function handleChange(e: Event) {
	const input = e.target as HTMLInputElement
	if (!input.files) return
	addFiles(input.files)
}
</script>

<style lang="scss" scoped>
label {
	flex-direction: unset;
	max-height: unset;
	&:focus-within {
		outline: 0.25rem solid var(--color-focus-ring);
	}

	svg {
		height: 1rem;
	}
	input {
		position: absolute;
		height: 1px;
		width: 1px;
		overflow: hidden;
		clip: rect(1px, 1px, 1px, 1px);
	}
	&.long-style {
		display: flex;
		padding: 1.5rem 2rem;
		justify-content: center;
		align-items: center;
		grid-gap: 0.5rem;
		background-color: var(--color-button-bg);
		border-radius: var(--radius-sm);
		border: dashed 2px var(--color-contrast);
		cursor: pointer;
		color: var(--color-contrast);
	}
}
</style>
