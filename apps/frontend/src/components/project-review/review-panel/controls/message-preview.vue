<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { moderationSettings } from '@modrinth/moderation'
import { Tooltip } from '@modrinth/ui'
import { useEventListener } from '@vueuse/core'
import { computed, h, onBeforeUnmount, ref, useTemplateRef, watch } from 'vue'

import ProjectIssueCard from '~/components/ui/project-issue-card/index.vue'
import { useModerationSettings } from '~/composables/moderation'
import { injectReviewMessages } from '~/providers/project-review/review-messages'
import {
	injectReviewPanels,
	type ReviewIssueControl,
} from '~/providers/project-review/review-panels'

import { useReviewEventListener } from '../../shortcuts'
import { getPreviewHover } from './preview-hover'

const props = defineProps<{ entry?: ReviewIssueControl; disabled?: boolean }>()
const settings = useModerationSettings()
const disabled = computed(
	() =>
		props.disabled ||
		!settings.value.get(moderationSettings.General.EnableDelayedHoverMessagePreviews),
)
const reviewMessages = injectReviewMessages()
const panels = injectReviewPanels()
const trigger = useTemplateRef<HTMLElement>('trigger')
const open = ref(false)
let hover: ReturnType<typeof getPreviewHover> | undefined
const preview = {
	show: () => {
		if (trigger.value?.isConnected) open.value = true
	},
	hide: () => {
		open.value = false
	},
}
const issue = computed<Labrinth.Threads.v3.ThreadIssue | undefined>(() => {
	const entry = props.entry
	if (!open.value || !entry) return undefined
	const resolved = panels.previewIssue(entry)
	if (!resolved) return undefined
	const definition = panels.availableIssues.value.find(({ id }) => id === entry.control.issueId)
	return {
		id: `preview-${entry.control.issueId}`,
		created_by: null,
		created_at: '',
		verdict: 'open',
		why: {
			issue_id: entry.control.issueId,
			title: definition?.title ?? entry.control.label,
			message: reviewMessages.previewIssueMessage(entry),
			locations: resolved.locations,
		},
		facets: resolved.facets.map((facet, index) => ({
			...facet,
			id: `preview-${entry.control.issueId}-${index}`,
			user_addressed: false,
			moderator_verified: false,
			verdict: 'open',
		})),
	}
})

function leave() {
	hover?.leave(preview)
}

function dismiss() {
	hover?.dismiss(preview)
}

function enter() {
	if (disabled.value || !props.entry || !trigger.value) return
	const next = getPreviewHover(trigger.value.ownerDocument)
	if (hover !== next) leave()
	hover = next
	hover.enter(preview)
}

useEventListener(trigger, 'mouseenter', enter)
useEventListener(trigger, 'mouseleave', leave)
useEventListener(trigger, 'focusin', enter)
useEventListener(trigger, 'focusout', leave)
useEventListener(trigger, 'click', dismiss)
watch(() => [props.entry?.control, disabled.value], dismiss, { flush: 'sync' })
onBeforeUnmount(leave)
useReviewEventListener('keydown', (event) => {
	if (event.key === 'Escape') dismiss()
})
useReviewEventListener('blur', dismiss)
</script>

<template>
	<div ref="trigger">
		<slot />
		<Tooltip
			:animated="false"
			:show-arrow="false"
			:reference="open && issue ? trigger : null"
			:content="() => (issue ? h(ProjectIssueCard, { issues: [issue], preview: true }) : null)"
			placement="right"
			panel-class="w-[min(28rem,calc(100vw-1rem))] max-h-[calc(100vh-1rem)] overflow-hidden !rounded-2xl !border-0 !bg-surface-2 !p-0 font-normal"
		/>
	</div>
</template>
