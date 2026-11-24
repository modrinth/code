<template>
	<InsertModal
		ref="modal"
		:header="formatMessage(messages.videoModalHeader)"
		:markdown="markdown"
		:can-insert="!!url"
		:error="error"
		@insert="emit('insert', $event)"
	>
		<label class="label" :for="urlId">
			<span class="label__title">
				{{ formatMessage(messages.videoModalUrlFieldTitle) }}<span class="required">*</span>
			</span>
			<span class="label__description">
				{{ formatMessage(messages.videoModalUrlFieldDescription) }}
			</span>
		</label>
		<Input
			:id="urlId"
			v-model="input"
			:icon="YouTubeIcon"
			type="text"
			:placeholder="formatMessage(messages.videoModalUrlFieldPlaceholder)"
			clearable
			wrapper-class="w-full"
		/>
	</InsertModal>
</template>

<script setup lang="ts">
import { YouTubeIcon } from '@modrinth/assets'
import { computed, useId, useTemplateRef } from 'vue'

import { defineMessages, useVIntl } from '../../../composables/i18n'
import Input from '../inputs/Input.vue'
import InsertModal from './insert-modal.vue'
import { useMarkdownUrl } from './markdown-url'

const emit = defineEmits<{ insert: [markdown: string] }>()

const { formatMessage } = useVIntl()
const messages = defineMessages({
	videoModalHeader: {
		id: 'markdown-editor.video-modal.header',
		defaultMessage: 'Insert YouTube video',
	},
	videoModalUrlFieldTitle: {
		id: 'markdown-editor.video-modal.url-field.title',
		defaultMessage: 'YouTube video URL',
	},
	videoModalUrlFieldDescription: {
		id: 'markdown-editor.video-modal.url-field.description',
		defaultMessage: 'Enter a valid link to a YouTube video.',
	},
	videoModalUrlFieldPlaceholder: {
		id: 'markdown-editor.video-modal.url-field.placeholder',
		defaultMessage: 'Enter YouTube video URL',
	},
	videoEmbedTitle: {
		id: 'markdown-editor.video-embed.title',
		defaultMessage: 'YouTube video player',
	},
})

const modal = useTemplateRef('modal')
const urlId = useId()
const { input, url, error } = useMarkdownUrl()

const youtubeRegex =
	/^(?:https?:)?(?:\/\/)?(?:youtu\.be\/|(?:www\.|m\.)?youtube\.com\/(?:watch|v|embed)(?:\.php)?(?:\?.*v=|\/))(?<temp1>[a-zA-Z0-9_-]{7,15})(?:[?&][a-zA-Z0-9_-]+=[a-zA-Z0-9_-]+)*$/

const markdown = computed(() => {
	const match = youtubeRegex.exec(input.value)
	if (match) {
		return `<iframe width="560" height="315" src="https://www.youtube-nocookie.com/embed/${match[1]}" title="${formatMessage(messages.videoEmbedTitle)}" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" allowfullscreen></iframe>`
	}
	return ''
})

function show() {
	input.value = ''
	modal.value?.show()
}

defineExpose({ show })
</script>
