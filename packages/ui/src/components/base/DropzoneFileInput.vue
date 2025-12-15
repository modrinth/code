<template>
	<label
		:class="[
			'flex flex-col items-center justify-center gap-4 cursor-pointer p-8 rounded-3xl border-2 border-dashed bg-surface-4 text-contrast transition-colors',
			isDragOver ? 'border-purple' : 'border-surface-5',
		]"
		@dragover.prevent="onDragOver"
		@dragleave.prevent="onDragLeave"
		@drop.prevent="handleDrop"
	>
		<div
			class="h-14 w-14 grid place-content-center rounded-2xl text-brand border-brand border-solid border bg-highlight-green"
		>
			<FolderUpIcon aria-hidden="true" class="text-brand w-8 h-8" />
		</div>

		<div class="flex flex-col items-center justify-center gap-1 text-contrast text-center">
			<div class="text-contrast font-medium text-pretty">
				{{ prompt }}
			</div>
			<span class="text-primary text-sm text-pretty">
				You can try to drag files or folder or <br />
				click this area to select it
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
		multiple?: boolean
		accept?: string
		maxSize?: number | null
		shouldAlwaysReset?: boolean
		disabled?: boolean
	}>(),
	{
		prompt: 'Drag and drop files or click to browse',
		multiple: false,
		accept: '*',
		maxSize: null,
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
