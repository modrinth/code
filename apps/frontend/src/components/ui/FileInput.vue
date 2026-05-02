<template>
	<label
		:class="{ 'long-style': longStyle }"
		:disabled="disabled"
		@drop.prevent="handleDrop"
		@dragover.prevent
	>
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
import { useFormatBytes } from '@modrinth/ui'
import { fileIsValid } from '@modrinth/utils'
import { ref } from 'vue'

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

	svg {
		height: 1rem;
	}

	input {
		display: none;
	}

	&.long-style {
		display: flex;
		padding: 1.5rem 2rem;
		justify-content: center;
		align-items: center;
		grid-gap: 0.5rem;
		background-color: var(--color-button-bg);
		border-radius: var(--size-rounded-sm);
		border: dashed 0.3rem var(--color-text);
		cursor: pointer;
		color: var(--color-text-dark);
	}
}
</style>
