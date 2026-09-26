<template>
	<article
		class="flex min-w-0 flex-col gap-2 rounded-xl border border-solid border-surface-3 bg-surface-2 p-2.5 py-2 text-sm"
	>
		<div class="min-w-0">
			<div class="flex items-center gap-2">
				<button
					type="button"
					class="text-normal flex min-w-0 flex-1 items-center gap-1 border-0 bg-transparent p-0 text-left font-medium text-contrast"
					:aria-expanded="expanded"
					:aria-controls="contentId"
					@click="expanded = !expanded"
				>
					{{ issue.title }}
					<ChevronDownIcon
						class="size-4 shrink-0 transition-transform duration-150 ease-in-out motion-reduce:transition-none"
						:class="{ 'rotate-180': expanded }"
						aria-hidden="true"
					/>
				</button>
				<Button
					size="sm"
					type="quiet"
					circular
					:disabled="pending"
					:aria-label="formatMessage(messages.remove, { issue: issue.title })"
					class="-my-1.5 -mr-1.5 size-8"
					@click="removeIssue"
				>
					<XIcon aria-hidden="true" />
				</Button>
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
								v-if="fields.length"
								:disabled="pending"
								:inert="pending"
								class="m-0 flex min-w-0 flex-col gap-2 border-0 p-0"
							>
								<Controls v-for="binding in fields" :key="binding.key" :binding="binding" />
							</fieldset>
							<div class="flex min-w-0 flex-col gap-1">
								<div class="relative min-w-0" :class="{ 'issue-message-editor': editingMessage }">
									<MarkdownEditor
										v-if="editingMessage"
										:model-value="reviewMessages.issueMessage(issue.id)"
										:disabled="pending || generating"
										:heading-buttons="false"
										:hide-formatting-buttons="
											settings.get(moderationSettings.General.HideMarkdownFormattingButtons)
										"
										:max-height="240"
										:min-height="96"
										hide-markdown-hint
										@update:model-value="reviewMessages.editIssueMessage(issue.id, $event)"
									/>
									<div
										v-else
										class="issue-message markdown-body min-h-12 rounded-xl border border-solid border-surface-4 px-2.5 pb-2.5 text-xs [overflow-wrap:anywhere]"
										v-html="renderHighlightedString(reviewMessages.issueMessage(issue.id))"
									/>
									<div
										class="absolute right-1.5 z-10"
										:class="editingMessage ? '-top-0' : 'top-1.5'"
									>
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
											:disabled="pending || generating"
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
								<Button
									v-if="editingMessage && reviewMessages.hasIssueOverride(issue.id)"
									size="sm"
									type="quiet"
									class="self-end"
									:disabled="pending"
									@click="reviewMessages.resetIssueMessage(issue.id)"
								>
									<RefreshCwIcon class="size-4" aria-hidden="true" />{{
										formatMessage(messages.reset)
									}}
								</Button>
							</div>
						</div>
					</div>
				</div>
			</Transition>
		</div>
		<IssueToggles :issue="issue" />
		<p v-if="needsToggle" class="m-0 text-xs text-orange" role="status">
			{{ formatMessage(messages.chooseOption) }}
		</p>
	</article>
</template>

<script setup lang="ts">
import { ChevronDownIcon, EditIcon, RefreshCwIcon, XIcon } from '@modrinth/assets'
import { moderationSettings } from '@modrinth/moderation'
import { Button, commonMessages, defineMessages, MarkdownEditor, useVIntl } from '@modrinth/ui'
import { renderHighlightedString } from '@modrinth/utils/highlightjs/index'
import { computed, ref, useId } from 'vue'

import { useModerationSettings } from '~/composables/moderation'
import { injectReviewMessages } from '~/providers/project-review/review-messages'
import {
	injectReviewPanels,
	type ReviewIssue,
	type ReviewPanelBinding,
} from '~/providers/project-review/review-panels'
import { injectReviewSubmission } from '~/providers/project-review/review-submission'

import Controls from '../review-panel/controls.vue'
import IssueToggles from './issue-toggles.vue'

const props = defineProps<{ issue: ReviewIssue }>()
const settings = useModerationSettings()
const panels = injectReviewPanels()
const reviewMessages = injectReviewMessages()
const { generating } = reviewMessages
const { pending } = injectReviewSubmission()
const expanded = ref(false)
const editingMessage = ref(false)
const contentId = useId()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	remove: {
		id: 'project-review.issues.remove',
		defaultMessage: 'Remove {issue}',
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
const fields = computed(() => {
	const bindings = new Map<string, ReviewPanelBinding>()
	for (const { binding, control } of props.issue.controls) {
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
		({ issueId, key }) => issueId === props.issue.id && key === 'toggle',
	),
)
function removeIssue() {
	panels.removeIssue(props.issue.id)
	reviewMessages.resetIssueMessage(props.issue.id)
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
	padding-right: 4.5rem;
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
</style>
