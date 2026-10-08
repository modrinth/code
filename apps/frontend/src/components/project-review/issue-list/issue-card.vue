<template>
	<article
		class="flex min-w-0 flex-col gap-2 rounded-xl border border-solid border-surface-3 bg-surface-2 p-2.5 py-2 text-sm"
	>
		<div class="min-w-0">
			<div class="flex flex-col gap-1">
				<div class="flex items-center gap-2">
					<button
						type="button"
						class="text-normal flex min-w-0 flex-1 items-center gap-1 border-0 bg-transparent p-0 text-left font-medium text-contrast"
						:aria-expanded="expanded"
						:aria-controls="contentId"
						@click="expanded = !expanded"
					>
						<Tooltip
							v-if="previousIssue && previousIssue.verdict !== 'resolved'"
							:text="formatMessage(messages.reReview)"
						>
							<span
								class="flex shrink-0 items-center text-orange"
								role="img"
								:aria-label="formatMessage(messages.reReview)"
							>
								<TagCategoryRefreshCcwIcon class="size-4" aria-hidden="true" />
							</span>
						</Tooltip>
						{{ issue.title }}
						<ChevronDownIcon
							class="size-4 shrink-0 transition-transform duration-150 ease-in-out motion-reduce:transition-none"
							:class="{ 'rotate-180': expanded }"
							aria-hidden="true"
						/>
					</button>
					<div class="flex items-center gap-1">
						<Tooltip
							v-if="reviewMessages.hasIssueOverride(messageKey)"
							:text="formatMessage(messages.editedTooltip)"
						>
							<span class="text-xs">{{ formatMessage(messages.edited) }}</span>
						</Tooltip>
						<slot name="action">
							<Button
								size="sm"
								type="quiet"
								:disabled="pending || disabled"
								:aria-label="
									formatMessage(resolved ? messages.add : messages.remove, {
										issue: issue.title,
									})
								"
								@click="resolved ? restoreIssue() : removeIssue()"
							>
								<template v-if="resolved">
									<PlusIcon aria-hidden="true" />
									{{ formatMessage(messages.notResolved) }}
								</template>
								<XIcon v-else aria-hidden="true" />
							</Button>
						</slot>
					</div>
				</div>
				<div v-if="issueBindings.length || facetLabels.length" class="flex flex-wrap gap-1">
					<Button
						v-for="binding in issueBindings"
						:key="binding.key"
						size="xs"
						type="quiet"
						class="!h-auto !min-h-0 !rounded-full !bg-surface-3 !px-2.5 !py-1 !text-xs !font-medium hover:!bg-surface-4"
						:aria-label="formatMessage(messages.openPanel, { panel: binding.panel.title })"
						@click="revealPanel(binding.key)"
					>
						<PanelTopIcon
							class="!mb-px -ml-px !size-3.5 !min-h-0 !min-w-0 shrink-0"
							aria-hidden="true"
						/>
						{{ binding.panel.title }}
					</Button>
					<span
						v-for="facet in facetLabels"
						:key="facet.type"
						class="inline-flex min-w-0 items-center rounded-full bg-surface-3 px-2.5 py-1 text-xs font-medium [overflow-wrap:anywhere]"
					>
						{{ facet.label }}
					</span>
				</div>
			</div>

			<Transition
				name="issue-content"
				@before-leave="(element) => element.setAttribute('inert', '')"
			>
				<div
					v-if="expanded"
					:id="contentId"
					class="issue-content"
					role="group"
					:aria-label="formatMessage(messages.message, { issue: issue.title })"
				>
					<div class="min-h-0 min-w-0">
						<div class="flex flex-col gap-2 pt-2">
							<fieldset
								v-if="!resolved && fields.length"
								:disabled="pending || disabled || resolved"
								:inert="pending || disabled || resolved"
								class="m-0 flex min-w-0 flex-col gap-2 border-0 p-0"
							>
								<Controls v-for="binding in fields" :key="binding.key" :binding="binding" />
							</fieldset>
							<div class="flex min-w-0 flex-col gap-1">
								<div class="relative min-w-0" :class="{ 'issue-message-editor': editingMessage }">
									<MarkdownEditor
										v-if="editingMessage"
										:model-value="displayMessage"
										:disabled="pending || disabled || generating"
										:heading-buttons="false"
										:hide-formatting-buttons="
											settings.get(moderationSettings.General.HideMarkdownFormattingButtons)
										"
										:max-height="240"
										:min-height="96"
										hide-markdown-hint
										@update:model-value="reviewMessages.editIssueMessage(messageKey, $event)"
									/>
									<div
										v-else
										class="issue-message markdown-body min-h-12 rounded-xl border border-solid border-surface-4 px-2.5 pb-2.5 text-xs [overflow-wrap:anywhere]"
										v-html="renderHighlightedString(displayMessage)"
									/>
									<div
										v-if="!resolved"
										class="absolute right-1.5 z-10 flex items-center gap-1"
										:class="editingMessage ? '-top-0' : 'top-1.5'"
									>
										<Tooltip
											v-if="editingMessage && reviewMessages.hasIssueOverride(messageKey)"
											:text="formatMessage(messages.reset)"
										>
											<Button
												size="sm"
												type="quiet"
												circular
												icon-only
												class="!size-7"
												:aria-label="formatMessage(messages.reset)"
												:disabled="pending || disabled || generating"
												@click="reviewMessages.resetIssueMessage(messageKey)"
											>
												<RefreshCwIcon class="size-4" aria-hidden="true" />
											</Button>
										</Tooltip>
										<Button
											size="sm"
											:type="editingMessage ? 'colored' : 'quiet'"
											:color="editingMessage ? 'brand' : undefined"
											:circular="!editingMessage"
											:icon-only="!editingMessage"
											:aria-label="
												formatMessage(
													editingMessage ? commonMessages.doneLabel : commonMessages.editButton,
												)
											"
											:disabled="pending || disabled || generating"
											:class="editingMessage ? '!h-7' : ''"
											@click="editingMessage = !editingMessage"
										>
											<template v-if="editingMessage">{{
												formatMessage(commonMessages.doneLabel)
											}}</template>
											<EditIcon v-else aria-hidden="true" />
										</Button>
									</div>
								</div>
							</div>
						</div>
					</div>
				</div>
			</Transition>
		</div>
		<IssueToggles v-if="!resolved" :issue="issue" :disabled="disabled" @remove="removeIssue" />
		<p v-if="!resolved && needsToggle" class="m-0 text-xs text-orange" role="status">
			{{ formatMessage(messages.chooseOption) }}
		</p>
	</article>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	ChevronDownIcon,
	EditIcon,
	PanelTopIcon,
	PlusIcon,
	RefreshCwIcon,
	TagCategoryRefreshCcwIcon,
	XIcon,
} from '@modrinth/assets'
import { moderationSettings } from '@modrinth/moderation'
import { issueTargetLabels } from '@modrinth/moderation/src/data/issues/component-builders/targets'
import {
	Button,
	commonMessages,
	defineMessages,
	MarkdownEditor,
	Tooltip,
	useVIntl,
} from '@modrinth/ui'
import { renderHighlightedString } from '@modrinth/utils/highlightjs/index'
import { computed, ref, useId, watch } from 'vue'

import { useModerationSettings } from '~/composables/moderation'
import { injectReviewMessages } from '~/providers/project-review/review-messages'
import {
	injectReviewPanels,
	type ReviewIssue,
	type ReviewPanelBinding,
} from '~/providers/project-review/review-panels'
import { injectReviewPreviousIssues } from '~/providers/project-review/review-previous-issues'
import { injectReviewSubmission } from '~/providers/project-review/review-submission'

import { injectReviewContext } from '../review-panel/context'
import Controls from '../review-panel/controls/index.vue'
import IssueToggles from './issue-toggles.vue'

const props = withDefaults(
	defineProps<{
		issue: ReviewIssue | Labrinth.Threads.v3.ThreadIssue
		disabled?: boolean
		resolved?: boolean
		previewMessage?: string
	}>(),
	{ disabled: false, resolved: false, previewMessage: undefined },
)
const emit = defineEmits<{ add: [] }>()
const settings = useModerationSettings()
const panels = injectReviewPanels()
const previousIssues = injectReviewPreviousIssues()
const previousIssue = computed(() => {
	if (!('controls' in props.issue)) return props.issue
	return previousIssues.issues.value.find(
		(entry) =>
			entry.verdict !== 'resolved' && previousIssues.cardIssue(entry).id === props.issue.id,
	)
})
const issue = computed(() =>
	'controls' in props.issue ? props.issue : previousIssues.cardIssue(props.issue),
)
const active = computed(() =>
	panels.activeIssues.value.some((entry) => entry.id === issue.value.id),
)
const { revealPanel } = injectReviewContext()
const issueBindings = computed(() =>
	previousIssue.value && (!active.value || panels.isRestoredIssue(issue.value.id))
		? previousIssues.issueBindings(previousIssue.value)
		: panels.issueBindings(issue.value.id),
)
const reviewMessages = injectReviewMessages()
const messageKey = computed(() =>
	previousIssue.value ? previousIssues.messageKey(previousIssue.value) : issue.value.id,
)
const { generating } = reviewMessages
const { pending } = injectReviewSubmission()
const expanded = ref(false)
const editingMessage = ref(false)
const contentId = useId()
watch(
	() => props.disabled,
	(disabled) => {
		if (disabled) editingMessage.value = false
	},
)
const { formatMessage } = useVIntl()
const messages = defineMessages({
	notResolved: {
		id: 'project-review.issues.not-resolved',
		defaultMessage: 'Not resolved',
	},
	reReview: {
		id: 'project-review.previous-issues.re-review',
		defaultMessage: 'Re-review',
	},
	openPanel: {
		id: 'project-review.issues.open-panel',
		defaultMessage: 'Open {panel} review panel',
	},
	edited: {
		id: 'project-review.issues.edited',
		defaultMessage: '(Edited)',
	},
	editedTooltip: {
		id: 'project-review.issues.edited-tooltip',
		defaultMessage: 'Issue message has been edited',
	},
	remove: {
		id: 'project-review.issues.remove',
		defaultMessage: 'Remove {issue}',
	},
	add: {
		id: 'project-review.issues.add',
		defaultMessage: 'Add {issue}',
	},
	message: {
		id: 'project-review.issues.message',
		defaultMessage: 'Message for {issue}',
	},
	reset: {
		id: 'project-review.issues.reset-message',
		defaultMessage: 'Reset message',
	},
	chooseOption: {
		id: 'project-review.issues.choose-option',
		defaultMessage: 'Choose at least one option for this issue.',
	},
})
const facetLabels = computed(() => {
	const facets =
		previousIssue.value?.facets ??
		panels.activeIssues.value.find((entry) => entry.id === issue.value.id)?.facets ??
		[]
	return [...new Set(facets.map(({ what }) => what.type))]
		.filter((type) => type !== 'mark_addressed')
		.map((type) => ({ type, label: formatMessage(issueTargetLabels[type]) }))
})
const displayMessage = computed(() => {
	if (props.disabled && props.previewMessage !== undefined) return props.previewMessage
	if (previousIssue.value) return previousIssues.reviewMessage(previousIssue.value)
	return reviewMessages.issueMessage(issue.value.id)
})
const fields = computed(() => {
	const bindings = new Map<string, ReviewPanelBinding>()
	for (const { binding, control } of issue.value.controls) {
		if (control.type === 'toggle') continue
		const entry: ReviewPanelBinding = bindings.get(binding.key) ?? {
			...binding,
			panel: { ...binding.panel, sections: [{ controls: [] }] },
		}
		entry.panel.sections[0].controls.push(control)
		bindings.set(binding.key, entry)
	}
	return [...bindings.values()]
})
const needsToggle = computed(() =>
	panels.validationErrors.value.some(
		({ issueId, key }) => issueId === issue.value.id && key === 'toggle',
	),
)
function removeIssue() {
	if (pending.value || props.disabled) return
	if (previousIssue.value) previousIssues.markNoLongerApplicable(previousIssue.value)
	else {
		panels.removeIssue(issue.value.id)
		reviewMessages.resetIssueMessage(messageKey.value)
	}
}

function restoreIssue() {
	if (pending.value || props.disabled || !previousIssue.value) return
	previousIssues.restoreIssue(previousIssue.value)
	emit('add')
}
</script>

<style scoped>
.issue-content {
	display: grid;
	grid-template-rows: 1fr;
}

.issue-content-enter-active,
.issue-content-leave-active {
	transition: grid-template-rows 0.2s ease-in-out;
}

.issue-content-enter-active > div,
.issue-content-leave-active > div {
	overflow: hidden;
}

.issue-content-enter-from,
.issue-content-leave-to {
	grid-template-rows: 0fr;
}

@media (prefers-reduced-motion: reduce) {
	.issue-content-enter-active,
	.issue-content-leave-active {
		transition: none;
	}
}

.issue-message-editor :deep(.editor-action-row) {
	box-sizing: border-box;
	padding-right: 6.5rem;
}
.issue-message :deep(h1),
.issue-message :deep(h2),
.issue-message :deep(h3) {
	font-size: 0.875rem;
	margin-block: 0.5rem;
}
.issue-message :deep(p) {
	margin-block: 0.5rem;
}

.issue-message :deep(pre) {
	white-space: pre-wrap;
	overflow-wrap: anywhere;
}
.markdown-body :deep(.review-card-image-targets) {
	display: flex;
	flex-direction: column;
	gap: 1rem;
}

.markdown-body :deep(.review-card-image-target) {
	display: flex;
	flex-direction: column;
	align-items: flex-start;
	gap: 0.5rem;
}

.markdown-body :deep(.review-card-image-list) {
	display: flex;
	flex-wrap: wrap;
	gap: 0.75rem;
	margin: 0;
	padding: 0;
	list-style: none;
}

.markdown-body :deep(.review-card-image-entry) {
	width: 140px;
	max-width: 100%;
	margin: 0;
	padding: 0;
	list-style: none;
}

.markdown-body :deep(.review-card-image-entry > a) {
	display: block;
}

.markdown-body :deep(.review-card-image-entry .review-card-gallery-image) {
	display: block;
	box-sizing: border-box;
	width: 100%;
	max-width: 140px;
	height: 112px;
	padding: 0.5rem;
	border-radius: 0.5rem;
	background: var(--surface-3);
	object-fit: contain;
}

.markdown-body :deep(.review-card-image-caption) {
	display: block;
	margin-top: 0.375rem;
	overflow-wrap: anywhere;
	font-size: 0.875em;
}

.markdown-body :deep(.review-card-image-target .review-card-image-description) {
	margin: 0;
}
</style>
