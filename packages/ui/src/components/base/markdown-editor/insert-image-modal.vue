<template>
	<InsertModal
		ref="modal"
		:header="formatMessage(messages.imageModalHeader)"
		:markdown="markdown"
		:can-insert="!!url && !!altText && !uploadError"
		:error="uploadError || urlError"
		@insert="emit('insert', $event)"
	>
		<label class="label" :for="altId">
			<span class="label__title">
				{{ formatMessage(messages.imageModalDescriptionFieldTitle) }}
				<span class="required">*</span>
			</span>
			<span class="label__description">
				{{ formatMessage(messages.imageModalDescriptionFieldDescription) }}
			</span>
		</label>
		<Input
			:id="altId"
			v-model="altText"
			:icon="AlignLeftIcon"
			type="text"
			:placeholder="formatMessage(messages.imageModalDescriptionFieldPlaceholder)"
			clearable
			wrapper-class="w-full"
		/>
		<label class="label" :for="urlId">
			<span class="label__title">
				{{ formatMessage(urlLabel) }}<span class="required">*</span>
			</span>
		</label>
		<div v-if="props.onImageUpload" class="image-strategy-chips">
			<Chips
				v-model="imageUploadOption"
				:items="['upload', 'link']"
				:format-label="formatImageUploadOption"
				:aria-label="formatMessage(messages.imageModalUploadModeLabel)"
			/>
		</div>
		<div v-if="props.onImageUpload && imageUploadOption === 'upload'" class="btn-input-alternative">
			<FileInput
				accept="image/png,image/jpeg,image/gif,image/webp"
				:prompt="formatMessage(messages.imageModalUploadPrompt)"
				long-style
				should-always-reset
				class="file-input"
				@change="handleImageUpload"
			>
				<UploadIcon />
			</FileInput>
		</div>
		<Input
			v-if="!props.onImageUpload || imageUploadOption === 'link'"
			:id="urlId"
			v-model="input"
			:icon="ImageIcon"
			type="text"
			:placeholder="formatMessage(messages.imageModalUrlFieldPlaceholder)"
			clearable
			wrapper-class="w-full"
			@update:model-value="resetUpload"
		/>
	</InsertModal>
</template>

<script setup lang="ts">
import { AlignLeftIcon, ImageIcon, UploadIcon } from '@modrinth/assets'
import { computed, ref, useId, useTemplateRef } from 'vue'

import { defineMessages, useVIntl } from '../../../composables/i18n'
import Chips from '../Chips.vue'
import FileInput from '../FileInput.vue'
import Input from '../inputs/Input.vue'
import InsertModal from './insert-modal.vue'
import { urlLabel, useMarkdownUrl } from './markdown-url'

const props = defineProps<{ onImageUpload?: (file: File) => Promise<string | undefined> }>()
const emit = defineEmits<{ insert: [markdown: string] }>()

const { formatMessage } = useVIntl()
const messages = defineMessages({
	imageModalHeader: {
		id: 'markdown-editor.image-modal.header',
		defaultMessage: 'Insert image',
	},
	imageModalDescriptionFieldTitle: {
		id: 'markdown-editor.image-modal.description-field.title',
		defaultMessage: 'Description (alt text)',
	},
	imageModalDescriptionFieldDescription: {
		id: 'markdown-editor.image-modal.description-field.description',
		defaultMessage:
			'Describe the image completely as you would to someone who could not see the image.',
	},
	imageModalDescriptionFieldPlaceholder: {
		id: 'markdown-editor.image-modal.description-field.placeholder',
		defaultMessage: 'Describe the image...',
	},
	imageModalUploadModeLabel: {
		id: 'markdown-editor.image-modal.upload-mode.label',
		defaultMessage: 'Image source',
	},
	imageModalUploadModeUpload: {
		id: 'markdown-editor.image-modal.upload-mode.upload',
		defaultMessage: 'Upload',
	},
	imageModalUploadModeLink: {
		id: 'markdown-editor.image-modal.upload-mode.link',
		defaultMessage: 'Link',
	},
	imageModalUploadPrompt: {
		id: 'markdown-editor.image-modal.upload.prompt',
		defaultMessage: 'Drag and drop to upload or click to select file',
	},
	imageModalUrlFieldPlaceholder: {
		id: 'markdown-editor.image-modal.url-field.placeholder',
		defaultMessage: 'Enter the image URL...',
	},
})

const modal = useTemplateRef('modal')
const altId = useId()
const urlId = useId()
const altText = ref('')
const { input, url, error: urlError } = useMarkdownUrl()
const uploadError = ref('')
const imageUploadOption = ref('upload')
const markdown = computed(() => (url.value ? `![${altText.value || url.value}](${url.value})` : ''))
let uploadRequest = 0

function formatImageUploadOption(option: string) {
	if (option === 'upload') return formatMessage(messages.imageModalUploadModeUpload)
	if (option === 'link') return formatMessage(messages.imageModalUploadModeLink)
	return option
}

async function handleImageUpload(files: File[]) {
	const file = files[0]
	if (!file || !props.onImageUpload) return

	const request = ++uploadRequest
	try {
		const uploadedUrl = await props.onImageUpload(file)
		if (request !== uploadRequest || !uploadedUrl) return
		input.value = uploadedUrl
		uploadError.value = ''
	} catch (error) {
		if (request !== uploadRequest) return
		if (error instanceof Error) uploadError.value = error.message
		console.error(error)
	}
}

function resetUpload() {
	uploadRequest++
	uploadError.value = ''
}

function show() {
	resetUpload()
	altText.value = ''
	input.value = ''
	modal.value?.show()
}

defineExpose({ show })
</script>

<style lang="scss" scoped>
.file-input {
	width: 100%;
	padding: 1.5rem;
	padding-left: 2.5rem;
	background: var(--color-button-bg);
	border: 2px dashed var(--color-gray);
	border-radius: var(--radius-md);
	cursor: pointer;
	transition:
		opacity 0.5s ease-in-out,
		filter 0.2s ease-in-out,
		scale 0.05s ease-in-out,
		outline 0.2s ease-in-out;

	&:hover {
		filter: brightness(0.85);
	}
}

.image-strategy-chips {
	display: flex;
	align-items: center;
	justify-content: space-between;
	gap: var(--gap-xs);
	padding-bottom: var(--gap-md);
}

.btn-input-alternative {
	display: flex;
	flex-direction: column;
	align-items: center;
	justify-content: space-between;
	gap: var(--gap-xs);
	padding-bottom: var(--gap-xs);

	.btn {
		width: 100%;
		padding-left: 2.5rem;
		min-height: 4rem;
		display: flex;
		align-items: center;
		justify-content: start;
	}
}
</style>
