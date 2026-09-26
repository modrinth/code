<template>
	<div
		class="flex max-h-[60%] min-h-0 shrink-0 flex-col gap-2 border-0 border-t border-solid border-divider py-2.5"
	>
		<div class="min-h-0 overflow-y-auto" @keydown="onKeydown">
			<MarkdownEditor
				ref="editor"
				v-model="draft"
				:disabled="pending"
				:placeholder="formatMessage(messages.replyPlaceholder)"
				:on-image-upload="uploadImage"
				:heading-buttons="false"
				:hide-formatting-buttons="
					settings.get(moderationSettings.General.HideMarkdownFormattingButtons)
				"
				:max-height="200"
				:min-height="100"
				hide-markdown-hint
			/>
		</div>
		<div class="flex shrink-0 flex-wrap justify-end gap-2">
			<Button :disabled="!canSubmit || !draft.trim()" @click="submit('note')">
				<SpinnerIcon v-if="loadingAction === 'note'" class="animate-spin" aria-hidden="true" />
				<StickyNotePlusIcon v-else aria-hidden="true" />
				{{ formatMessage(messages.addNote) }}
			</Button>
			<Button :disabled="!canSubmit || !draft.trim()" @click="submit('reply')">
				<SpinnerIcon v-if="loadingAction === 'reply'" class="animate-spin" aria-hidden="true" />
				<ReplyIcon v-else aria-hidden="true" />
				{{ formatMessage(messages.reply) }}
			</Button>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ReplyIcon, SpinnerIcon, StickyNotePlusIcon } from '@modrinth/assets'
import { moderationSettings } from '@modrinth/moderation'
import { Button, defineMessages, MarkdownEditor, useVIntl } from '@modrinth/ui'
import { nextTick, ref } from 'vue'

import { useModerationSettings } from '~/composables/moderation'
import { injectReviewSubmission } from '~/providers/project-review/review-submission'

const settings = useModerationSettings()
const { draft, pending, canSubmit, loadingAction, submit, uploadImage } = injectReviewSubmission()
const editor = ref<InstanceType<typeof MarkdownEditor>>()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	reply: { id: 'project-review.composer.reply', defaultMessage: 'Reply' },
	addNote: { id: 'project-review.composer.add-note', defaultMessage: 'Add private note' },
	replyPlaceholder: {
		id: 'project-review.composer.reply-placeholder',
		defaultMessage: 'Reply to thread…',
	},
})
async function openEditor() {
	await nextTick()
	await editor.value?.focus()
}
function onKeydown(event: KeyboardEvent) {
	if (event.defaultPrevented || event.repeat || event.isComposing) return
	if (event.key === 'Escape' && event.target instanceof HTMLElement) {
		event.preventDefault()
		event.stopPropagation()
		event.target.blur()
	} else if (
		event.key === 'Enter' &&
		(event.ctrlKey || event.metaKey) &&
		!event.altKey &&
		!event.shiftKey
	) {
		event.preventDefault()
		event.stopPropagation()
		void submit('reply')
	}
}
defineExpose({ openEditor })
</script>
