<template>
	<div
		ref="messageBox"
		class="relative flex min-h-0 shrink-0 flex-col gap-1.5 border-0 border-t border-solid border-divider py-2.5 pb-px"
		:style="{
			height: `${height ?? 300}px`,
			maxHeight: 'max(180px, calc(100% - 300px))',
		}"
	>
		<div
			role="separator"
			tabindex="0"
			aria-orientation="horizontal"
			:aria-label="formatMessage(messages.resizeMessageBox)"
			:aria-valuenow="Math.round(measuredHeight)"
			:aria-valuemin="180"
			:aria-valuemax="maxHeight"
			class="message-box-resize absolute -top-2 left-0 z-10 h-4 w-full cursor-row-resize touch-none"
			:class="{ 'is-resizing': resizeStart }"
			@pointerdown="startResize"
			@pointermove="resize"
			@pointerup="stopResize"
			@pointercancel="stopResize"
			@lostpointercapture="stopResize"
			@dblclick.prevent="height = null"
			@keydown.up.prevent="setHeight(measuredHeight + 20)"
			@keydown.down.prevent="setHeight(measuredHeight - 20)"
		/>
		<div class="message-editor min-h-0 flex-1 overflow-hidden" @keydown="onKeydown">
			<TemplateMarkdownEditor
				ref="editor"
				v-model="draft"
				:disabled="pending"
				:templates="templates"
				:placeholder="formatMessage(messages.replyPlaceholder)"
				:on-image-upload="uploadImage"
				:heading-buttons="false"
				:hide-formatting-buttons="
					settings.get(moderationSettings.General.HideMarkdownFormattingButtons)
				"
				:min-height="1"
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
import { getMessageTemplates, moderationSettings } from '@modrinth/moderation'
import { Button, defineMessages, useVIntl } from '@modrinth/ui'
import { useElementSize } from '@vueuse/core'
import { computed, nextTick, ref } from 'vue'

import { useModerationSettings } from '~/composables/moderation'
import { injectProjectReviewPageContext } from '~/providers/project-review'
import { injectReviewSubmission } from '~/providers/project-review/review-submission'

import TemplateMarkdownEditor from './template-markdown-editor.vue'

const settings = useModerationSettings()
const { project, projectV2 } = injectProjectReviewPageContext()
const { formatMessage } = useVIntl()
const templates = computed(() => {
	const current = project.value
	const legacy = projectV2.value
	if (!current || !legacy || current.id !== legacy.id) return []
	return getMessageTemplates({ project: current, projectV2: legacy })
})
const { draft, pending, canSubmit, loadingAction, submit, uploadImage } = injectReviewSubmission()
const editor = ref<InstanceType<typeof TemplateMarkdownEditor>>()
const messageBox = ref<HTMLElement | null>(null)
const { height: measuredHeight } = useElementSize(messageBox)
const height = ref<number | null>(null)
const resizeStart = ref<{ y: number; height: number } | null>(null)
const maxHeight = ref(400)
const messages = defineMessages({
	resizeMessageBox: {
		id: 'project-review.message-box.resize',
		defaultMessage: 'Resize message box',
	},
	reply: { id: 'project-review.composer.reply', defaultMessage: 'Reply' },
	addNote: { id: 'project-review.composer.add-note', defaultMessage: 'Private note' },
	replyPlaceholder: {
		id: 'project-review.composer.reply-placeholder',
		defaultMessage: 'Reply to thread…',
	},
})
function setHeight(value: number) {
	maxHeight.value = Math.max(180, (messageBox.value?.parentElement?.clientHeight ?? 700) - 300)
	height.value = Math.min(maxHeight.value, Math.max(180, value))
}
function startResize(event: PointerEvent) {
	if (event.button !== 0) return
	event.preventDefault()
	const target = event.currentTarget as HTMLElement
	target.setPointerCapture(event.pointerId)
	resizeStart.value = { y: event.clientY, height: measuredHeight.value }
}
function resize(event: PointerEvent) {
	if (resizeStart.value) {
		setHeight(resizeStart.value.height + resizeStart.value.y - event.clientY)
	}
}
function stopResize(event: PointerEvent) {
	resizeStart.value = null
	const target = event.currentTarget as HTMLElement
	if (target.hasPointerCapture(event.pointerId)) target.releasePointerCapture(event.pointerId)
}
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

<style scoped>
.message-box-resize::after {
	content: '';
	position: absolute;
	top: calc(50% - 1px);
	left: 0;
	width: 100%;
	height: 1px;
	pointer-events: none;
	background-color: var(--color-brand);
	opacity: 0;
}

.message-box-resize:hover::after {
	opacity: 1;
	transition: opacity 0s 0.5s;
}

.message-box-resize:focus-visible::after,
.message-box-resize.is-resizing::after {
	opacity: 1;
	transition: none;
}

.message-editor :deep(.block) {
	display: flex;
	height: 100%;
	min-height: 0;
	flex-direction: column;
}

.message-editor :deep(.editor-action-row) {
	flex-shrink: 0;
}

.message-editor :deep([class~='group/input']),
.message-editor :deep(.block > div:not([class])) {
	flex: 1;
	min-height: 0;
	overflow: hidden;
}

.message-editor :deep(.cm-editor),
.message-editor :deep(.cm-scroller) {
	height: 100%;
	min-height: 0;
}

.message-editor :deep(.markdown-body-wrapper) {
	box-sizing: border-box;
	height: 100%;
	min-height: 0;
	overflow-y: auto;
}
</style>
