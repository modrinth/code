<template>
	<label
		ref="dropTargetRef"
		:class="[
			'flex flex-col items-center justify-center border-2 text-contrast transition-colors',
			size === 'small' ? 'p-5' : size === 'medium' ? 'p-10' : 'p-12',
			size === 'small' ? 'gap-2' : 'gap-4',
			size === 'small' ? 'rounded-2xl' : 'rounded-3xl',
			isProcessing ? 'border-solid cursor-default' : 'border-dashed cursor-pointer',
			isHighlighted ? 'border-brand bg-brand-highlight' : 'border-surface-5 bg-surface-4',
		]"
		:aria-busy="isProcessing || undefined"
		:data-highlighted="isHighlighted || undefined"
		@click="isProcessing && $event.preventDefault()"
		@dragenter="dropTargetProps.onDragenter"
		@dragover="dropTargetProps.onDragover"
		@dragleave="dropTargetProps.onDragleave"
		@drop="dropTargetProps.onDrop"
	>
		<div
			:class="[
				'grid place-content-center  text-brand border-brand border-solid border bg-brand-highlight',
				size === 'small' ? 'w-10 h-10' : 'h-14 w-14',
				size === 'small' ? 'rounded-xl' : 'rounded-2xl',
			]"
		>
			<SpinnerIcon
				v-if="isProcessing"
				aria-hidden="true"
				:class="['animate-spin text-brand', size === 'small' ? 'w-6 h-6' : 'w-8 h-8']"
			/>
			<FolderUpIcon
				v-else
				aria-hidden="true"
				:class="['text-brand', size === 'small' ? 'w-6 h-6' : 'w-8 h-8']"
			/>
		</div>

		<div class="flex flex-col items-center justify-center gap-1 text-contrast text-center">
			<div
				v-if="primaryPrompt && !isProcessing"
				:class="['font-medium text-pretty', isHighlighted ? 'text-brand' : 'text-contrast']"
			>
				{{ primaryPrompt }}
			</div>
			<span
				:class="['text-sm text-pretty', isHighlighted ? 'text-brand' : 'text-primary']"
				role="status"
			>
				{{ isProcessing ? processingPrompt : secondaryPrompt }}
			</span>
		</div>

		<input
			ref="fileInput"
			type="file"
			:multiple="multiple"
			:accept="accept"
			:disabled="disabled || isProcessing"
			class="sr-only"
			@change="handleChange"
		/>
	</label>
</template>

<script setup lang="ts">
import type { LocalFile } from '@modrinth/api-client'
import { FolderUpIcon, SpinnerIcon } from '@modrinth/assets'
import { fileIsValid } from '@modrinth/utils'
import { computed, onBeforeUnmount, ref, watch } from 'vue'

import { useFormatBytes } from '../../composables'
import { useFileDropTarget } from '../../composables/file-drop'
import { injectNotificationManager } from '../../providers'

const { addNotification } = injectNotificationManager()

const fileInput = ref<HTMLInputElement | null>(null)

const emit = defineEmits<{
	(e: 'change', files: File[]): void
	(e: 'local-change', files: LocalFile[]): void
}>()

const props = withDefaults(
	defineProps<{
		primaryPrompt?: string | null
		secondaryPrompt?: string | null
		processingPrompt?: string
		multiple?: boolean
		accept?: string
		maxSize?: number | null
		shouldAlwaysReset?: boolean
		disabled?: boolean
		size?: 'small' | 'medium' | 'large'
		/** Emits native drops as local files through `local-change`, without reading them into memory. */
		localFiles?: boolean
	}>(),
	{
		primaryPrompt: 'Drop files here or click to upload',
		secondaryPrompt: 'Only supported file types will be accepted',
		processingPrompt: 'Processing...',
		size: 'large',
	},
)

const processing = defineModel<boolean>('processing', { default: false })

const formatBytes = useFormatBytes()

const files = ref<File[]>([])

function matchesAccept(file: { name: string; type?: string }, accept?: string): boolean {
	if (!accept || accept.trim() === '') return true

	const fileType = file.type ?? '' // e.g. "image/png"
	const fileName = file.name.toLowerCase()

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

function addFiles(incoming: FileList | File[], shouldNotReset = false) {
	if (!shouldNotReset || props.shouldAlwaysReset) {
		files.value = Array.from(incoming)
	}

	// Filter out files that don't match the accept prop
	const invalidFiles = files.value.filter((file) => !matchesAccept(file, props.accept))
	if (invalidFiles.length > 0) {
		for (const file of invalidFiles) {
			addNotification({
				title: 'Invalid file',
				text: `The file "${file.name}" is not a valid file type for this project.`,
				type: 'error',
			})
		}
		files.value = files.value.filter((file) => matchesAccept(file, props.accept))
	}

	const validationOptions = {
		maxSize: props.maxSize ?? undefined,
		alertOnInvalid: true,
	}

	files.value = files.value.filter((file) => fileIsValid(file, validationOptions, formatBytes))

	if (files.value.length > 0) {
		emit('change', files.value)
	}

	if (fileInput.value) fileInput.value.value = ''
}

function addLocalFiles(incoming: LocalFile[]) {
	const valid = incoming.filter((file) => {
		if (!matchesAccept(file, props.accept)) {
			addNotification({
				title: 'Invalid file',
				text: `The file "${file.name}" is not a valid file type for this project.`,
				type: 'error',
			})
			return false
		}
		return fileIsValid(
			file,
			{ maxSize: props.maxSize ?? undefined, alertOnInvalid: true },
			formatBytes,
		)
	})

	if (valid.length > 0) {
		emit('local-change', valid)
	}
}

const dropTargetRef = ref<HTMLElement | null>(null)
const { isDragging, isProcessing, dropTargetProps } = useFileDropTarget({
	target: dropTargetRef,
	disabled: computed(() => props.disabled ?? false),
	onFiles: (droppedFiles) => addFiles(droppedFiles),
	onLocalFiles: props.localFiles ? addLocalFiles : undefined,
})

const isHighlighted = computed(() => isDragging.value || isProcessing.value)

watch(isProcessing, (value) => (processing.value = value), { flush: 'sync' })
onBeforeUnmount(() => (processing.value = false))

function handleChange(e: Event) {
	const input = e.target as HTMLInputElement
	if (!input.files) return
	addFiles(input.files)
}
</script>
