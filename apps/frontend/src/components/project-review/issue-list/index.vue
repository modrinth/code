<template>
	<div class="min-h-0 min-w-0 flex-1 overflow-y-auto overflow-x-hidden overscroll-contain">
		<div
			v-if="reReview && panels.resolve({ kind: 're-review' })"
			:inert="pending"
			class="mb-3 min-w-0 p-1"
		>
			<ReviewPanel mode="inline" :target="{ kind: 're-review' }" hide-title />
		</div>
		<div
			v-if="wasReviewed && threadQuery.isPending.value"
			class="m-0 p-2 text-sm text-secondary"
			role="status"
		>
			{{ formatMessage(projectReviewMessages.loading) }}
		</div>
		<div v-else-if="wasReviewed && threadQuery.isError.value" class="flex flex-col gap-2 p-2">
			<p class="m-0 text-sm text-red" role="alert">
				{{ formatMessage(projectReviewMessages.loadError) }}
			</p>
			<Button class="w-fit" @click="() => threadQuery.refetch()">{{
				formatMessage(projectReviewMessages.retry)
			}}</Button>
		</div>
		<div v-if="issues.length" class="flex flex-col gap-1">
			<IssueCard
				v-for="issue in issues"
				:key="'controls' in issue ? `new:${issue.id}` : issue.id"
				:issue="issue"
				:resolved="reReview"
				@add="emit('add')"
			/>
		</div>
		<p
			v-else-if="!wasReviewed || (!threadQuery.isPending.value && !threadQuery.isError.value)"
			class="m-0 p-2 text-sm text-secondary"
		>
			{{ formatMessage(reReview ? messages.emptyReReview : messages.empty) }}
		</p>
		<Accordion
			v-if="reReview && resolvedIssues.length"
			class="mt-2"
			button-class="w-full border-0 bg-transparent text-sm font-medium py-2 px-1"
			content-class="flex flex-col gap-1 pt-1"
			overflow-visible
		>
			<template #button="{ open }">
				<span class="flex items-center gap-1 text-primary">
					<span> {{ formatMessage(messages.resolved) }} ({{ resolvedIssues.length }}) </span>
					<DropdownIcon
						class="size-5 shrink-0 text-primary transition-transform duration-300 motion-reduce:transition-none"
						:class="{ 'rotate-180': open }"
						aria-hidden="true"
					/>
				</span>
			</template>
			<IssueCard
				v-for="issue in resolvedIssues"
				:key="issue.id"
				:issue="issue"
				resolved
				@add="emit('add')"
			/>
		</Accordion>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { DropdownIcon } from '@modrinth/assets'
import { IssuePriority } from '@modrinth/moderation/src/data/issues'
import { Accordion, Button, defineMessages, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'
import { injectReviewPanels, type ReviewIssue } from '~/providers/project-review/review-panels'
import { injectReviewPreviousIssues } from '~/providers/project-review/review-previous-issues'
import { injectReviewSubmission } from '~/providers/project-review/review-submission'

import { projectReviewMessages } from '../messages'
import ReviewPanel from '../review-panel/index.vue'
import IssueCard from './issue-card.vue'

const props = withDefaults(defineProps<{ reReview?: boolean }>(), { reReview: false })
const emit = defineEmits<{ add: [] }>()
const { wasReviewed, threadQuery } = injectProjectReviewPageContext()
const panels = injectReviewPanels()
const previousIssues = injectReviewPreviousIssues()
const { pending } = injectReviewSubmission()
function priority(issue: ReviewIssue | Labrinth.Threads.v3.ThreadIssue) {
	return (
		('controls' in issue ? issue : previousIssues.cardIssue(issue)).priority ??
		IssuePriority.Default
	)
}
function compareIssues(
	a: ReviewIssue | Labrinth.Threads.v3.ThreadIssue,
	b: ReviewIssue | Labrinth.Threads.v3.ThreadIssue,
) {
	return (
		Number('moderator_verified' in a && a.moderator_verified) -
			Number('moderator_verified' in b && b.moderator_verified) || priority(a) - priority(b)
	)
}
const resolvedIssues = computed(() => [...previousIssues.resolvedIssues.value].sort(compareIssues))
const issues = computed(() => {
	if (props.reReview) return [...previousIssues.reReviewIssues.value].sort(compareIssues)
	const available = new Map(panels.availableIssues.value.map((issue) => [issue.id, issue]))
	const previousIds = previousIssues.associatedIssueIds.value
	const activeIssues = panels.activeIssues.value.flatMap(({ id }) => {
		const issue = available.get(id)
		return issue ? [issue] : []
	})
	return [
		...previousIssues.appliedIssues.value,
		...activeIssues.filter(({ id }) => !previousIds.has(id)),
	].sort(compareIssues)
})
const { formatMessage } = useVIntl()
const messages = defineMessages({
	resolved: {
		id: 'project-review.issues.resolved',
		defaultMessage: 'Resolved issues',
	},
	emptyReReview: {
		id: 'project-review.issues.empty-re-review',
		defaultMessage: 'No previous issues to re-review.',
	},
	empty: {
		id: 'project-review.issues.empty',
		defaultMessage: 'No issues flagged.',
	},
})
</script>
