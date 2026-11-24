<template>
	<InsertModal
		ref="modal"
		:header="formatMessage(messages.linkModalHeader)"
		:markdown="markdown"
		:can-insert="!!url"
		:error="error"
		@insert="emit('insert', $event)"
	>
		<label class="label" :for="labelId">
			<span class="label__title">{{ formatMessage(messages.linkModalLabelFieldTitle) }}</span>
		</label>
		<Input
			:id="labelId"
			v-model="label"
			:icon="AlignLeftIcon"
			type="text"
			:placeholder="formatMessage(messages.linkModalLabelFieldPlaceholder)"
			clearable
			wrapper-class="w-full"
		/>
		<label class="label" :for="urlId">
			<span class="label__title">
				{{ formatMessage(urlLabel) }}<span class="required">*</span>
			</span>
		</label>
		<Input
			:id="urlId"
			v-model="input"
			:icon="LinkIcon"
			type="text"
			:placeholder="formatMessage(messages.linkModalUrlFieldPlaceholder)"
			clearable
			wrapper-class="w-full"
		/>
	</InsertModal>
</template>

<script setup lang="ts">
import { AlignLeftIcon, LinkIcon } from '@modrinth/assets'
import { computed, ref, useId, useTemplateRef } from 'vue'

import { defineMessages, useVIntl } from '../../../composables/i18n'
import Input from '../inputs/Input.vue'
import InsertModal from './insert-modal.vue'
import { urlLabel, useMarkdownUrl } from './markdown-url'

const emit = defineEmits<{ insert: [markdown: string] }>()

const { formatMessage } = useVIntl()
const messages = defineMessages({
	linkModalHeader: {
		id: 'markdown-editor.link-modal.header',
		defaultMessage: 'Insert link',
	},
	linkModalLabelFieldTitle: {
		id: 'markdown-editor.link-modal.label-field.title',
		defaultMessage: 'Label',
	},
	linkModalLabelFieldPlaceholder: {
		id: 'markdown-editor.link-modal.label-field.placeholder',
		defaultMessage: 'Enter label...',
	},
	linkModalUrlFieldPlaceholder: {
		id: 'markdown-editor.link-modal.url-field.placeholder',
		defaultMessage: "Enter the link's URL...",
	},
})

const modal = useTemplateRef('modal')
const labelId = useId()
const urlId = useId()
const label = ref('')
const { input, url, error } = useMarkdownUrl()
const markdown = computed(() => (url.value ? `[${label.value || url.value}](${url.value})` : ''))

function show() {
	label.value = ''
	input.value = ''
	modal.value?.show()
}

defineExpose({ show })
</script>
