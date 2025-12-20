<template>
	<label
		:class="[
			'flex flex-col items-center justify-center  cursor-pointer border-2 border-dashed bg-surface-4 text-contrast transition-colors',
			size === 'small' ? 'p-5' : 'p-12',
			size === 'small' ? 'gap-2' : 'gap-4',
			size === 'small' ? 'rounded-2xl' : 'rounded-3xl',
			isDragOver ? 'border-purple' : 'border-surface-5',
		]"
		@dragover.prevent="onDragOver"
		@dragleave.prevent="onDragLeave"
		@drop.prevent="handleDrop"
	>
		<div
			:class="[
				'grid place-content-center  text-brand border-brand border-solid border bg-highlight-green',
				size === 'small' ? 'w-10 h-10' : 'h-14 w-14',
				size === 'small' ? 'rounded-xl' : 'rounded-2xl',
			]"
		>
			<FolderUpIcon
				aria-hidden="true"
				:class="['text-brand', size === 'small' ? 'w-6 h-6' : 'w-8 h-8']"
			/>
		</div>

		<div class="flex flex-col items-center justify-center gap-1 text-contrast text-center">
			<div class="text-contrast font-medium text-pretty">
				{{ primaryPrompt }}
			</div>
			<span class="text-primary text-sm text-pretty">
				{{ secondaryPrompt }}
			</span>
		</div>

		<input
			ref="fileInput"
			type="file"
			:multiple="multiple"
			:accept="accept"
			:disabled="disabled"
			class="hidden"
			@change="handleChange"
		/>
	</label>
</template>

<script setup lang="ts">
import { FolderUpIcon } from '@modrinth/assets'
import { fileIsValid } from '@modrinth/utils'
import { ref } from 'vue'

const fileInput = ref<HTMLInputElement | null>(null)

const emit = defineEmits<{
	(e: 'change', files: File[]): void
}>()

const props = withDefaults(
	defineProps<{
		prompt?: string
		primaryPrompt?: string | null
		secondaryPrompt?: string | null
		multiple?: boolean
		accept?: string
		maxSize?: number | null
		shouldAlwaysReset?: boolean
		disabled?: boolean
		size?: 'small' | 'standard'
	}>(),
	{
		prompt: 'Drag and drop files or click to browse',
		primaryPrompt: 'Drag and drop files or click to browse',
		secondaryPrompt: 'You can try to drag files or folder or click this area to select it',
		size: 'standard',
	},
)

const files = ref<File[]>([])

function addFiles(incoming: FileList, shouldNotReset = false) {
	if (!shouldNotReset || props.shouldAlwaysReset) {
		files.value = Array.from(incoming)
	}

	const validationOptions = {
		maxSize: props.maxSize ?? undefined,
		alertOnInvalid: true,
	}

	files.value = files.value.filter((file) => fileIsValid(file, validationOptions))

	if (files.value.length > 0) {
		emit('change', files.value)
	}

	if (fileInput.value) fileInput.value.value = ''
}

const isDragOver = ref(false)

function onDragOver() {
	isDragOver.value = true
}

function onDragLeave() {
	isDragOver.value = false
}

function handleDrop(e: DragEvent) {
	isDragOver.value = false

	if (!e.dataTransfer) return
	addFiles(e.dataTransfer.files)
}

function handleChange(e: Event) {
	const input = e.target as HTMLInputElement
	if (!input.files) return
	addFiles(input.files)
}
</script>
