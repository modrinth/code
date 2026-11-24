<template>
	<Tabs
		:value="mode"
		:tabs="tabs"
		:aria-label="formatMessage(messages.editorMode)"
		@update:value="mode = $event === 'split' || $event === 'preview' ? $event : 'write'"
	/>
</template>

<script setup lang="ts">
import { defineMessages, type MarkdownEditorMode, Tabs, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

const mode = defineModel<MarkdownEditorMode>({ required: true })
const { formatMessage } = useVIntl()
const messages = defineMessages({
	editorMode: { id: 'markdown-editor.mode', defaultMessage: 'Editor mode' },
	writeMode: { id: 'markdown-editor.mode.write', defaultMessage: 'Write' },
	splitMode: { id: 'markdown-editor.mode.split', defaultMessage: 'Split' },
	previewLabel: {
		id: 'markdown-editor.preview-label',
		defaultMessage: 'Preview',
	},
})

const tabs = computed(() => [
	{ value: 'write', label: formatMessage(messages.writeMode) },
	{ value: 'split', label: formatMessage(messages.splitMode) },
	{ value: 'preview', label: formatMessage(messages.previewLabel) },
])
</script>
