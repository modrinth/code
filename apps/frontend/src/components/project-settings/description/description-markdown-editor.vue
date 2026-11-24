<template>
	<div class="description-editor">
		<MarkdownEditor
			v-model="description"
			v-model:mode="mode"
			:placeholder="placeholder"
			:disabled="disabled"
			:on-image-upload="onImageUpload"
			:min-height="480"
			hide-markdown-hint
			class="!p-0"
		>
			<template #toolbar="{ actions }">
				<MarkdownEditorToolbar v-bind="actions" button-type="quiet" class="description-toolbar">
					<ModeTabs v-model="mode" class="ml-auto shrink-0" />
				</MarkdownEditorToolbar>
			</template>
		</MarkdownEditor>
	</div>
</template>

<script setup lang="ts">
import { MarkdownEditor, type MarkdownEditorMode, MarkdownEditorToolbar } from '@modrinth/ui'
import { ref } from 'vue'

import ModeTabs from './mode-tabs.vue'

defineProps<{
	placeholder?: string
	disabled?: boolean
	onImageUpload?: (file: File) => Promise<string | undefined>
}>()

const description = defineModel<string>({ required: true })
const mode = ref<MarkdownEditorMode>('split')
</script>

<style lang="scss" scoped>
.description-editor {
	@apply bg-surface-3;
	border: 1px solid var(--surface-5);
	border-radius: 16px;
	overflow: hidden;
}

.description-toolbar {
	padding: 8px 12px;
	border-bottom: 1px solid var(--surface-5);

	:deep(.editor-actions) {
		gap: 0;
	}

	:deep(.divider) {
		width: 1px;
		height: 20px;
		margin-inline: 4px;
	}
}

.description-editor :deep(.editor-input) {
	@apply bg-surface-2;
	border: 0;
	border-radius: 0;
	padding: 20px;
	font-weight: 400;

	&:focus-within {
		box-shadow: none;
		outline: none;
	}
}

.description-editor :deep(.markdown-body-wrapper) {
	box-sizing: border-box;
	border: 0;
	border-radius: 0;
	padding: 20px;
}

.description-editor :deep(.is-split .editor-preview) {
	border-top: 1px solid var(--surface-5);

	@media (min-width: 1024px) {
		border-top: 0;
		border-left: 1px solid var(--surface-5);
	}
}
</style>
