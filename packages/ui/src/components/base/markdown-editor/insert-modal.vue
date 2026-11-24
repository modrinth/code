<template>
	<NewModal ref="modal" :header="header" class="!w-[40rem]">
		<div class="modal-insert">
			<slot />
			<span v-if="error" class="label">
				<span class="label__title">{{ formatMessage(messages.errorLabel) }}</span>
				<span class="label__description">{{ error }}</span>
			</span>
			<span class="label">
				<span class="label__title">{{ formatMessage(messages.previewLabel) }}</span>
			</span>
			<div class="markdown-body-wrapper">
				<div class="markdown-body w-full" v-html="renderHighlightedString(markdown)" />
			</div>
			<div class="mt-4 flex justify-end gap-2">
				<Button type="outlined" @click="modal?.hide()">
					<XIcon /> {{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button type="colored" color="brand" :disabled="!canInsert" @click="insert">
					<PlusIcon /> {{ formatMessage(messages.insertButton) }}
				</Button>
			</div>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import { PlusIcon, XIcon } from '@modrinth/assets'
import { renderHighlightedString } from '@modrinth/utils/highlightjs/index'
import { useTemplateRef } from 'vue'

import { defineMessages, useVIntl } from '../../../composables/i18n'
import { commonMessages } from '../../../utils/common-messages.ts'
import NewModal from '../../modal/NewModal.vue'
import Button from '../buttons/Button.vue'

const props = defineProps<{
	header: string
	markdown: string
	canInsert: boolean
	error?: string
}>()
const emit = defineEmits<{ insert: [markdown: string] }>()

const { formatMessage } = useVIntl()
const messages = defineMessages({
	insertButton: {
		id: 'markdown-editor.insert-button',
		defaultMessage: 'Insert',
	},
	errorLabel: {
		id: 'markdown-editor.error-label',
		defaultMessage: 'Error',
	},
	previewLabel: {
		id: 'markdown-editor.preview-label',
		defaultMessage: 'Preview',
	},
})

const modal = useTemplateRef('modal')

function insert() {
	emit('insert', props.markdown)
	modal.value?.hide()
}

defineExpose({ show: () => modal.value?.show() })
</script>

<style lang="scss" scoped>
.modal-insert {
	:deep(.label) {
		margin-block: var(--gap-lg) var(--gap-sm);
		display: block;

		&:first-child {
			margin-top: 0;
		}
	}

	:deep(.label__title) {
		color: var(--color-contrast);
		display: block;
		font-size: 1.17rem;
		font-weight: 700;

		.required {
			color: var(--color-red);
		}
	}

	:deep(.input-group) {
		margin-top: var(--gap-lg);
	}
}

.markdown-body-wrapper {
	border: 1px solid var(--color-divider);
	border-radius: var(--radius-md);
	width: 100%;
	padding: var(--radius-md);
	min-height: 6rem;
}
</style>
