<template>
	<div class="min-h-0 flex-1 overflow-y-auto overscroll-contain">
		<div v-if="issues.length" class="flex flex-col gap-1">
			<IssueCard v-for="issue in issues" :key="issue.id" :issue="issue" />
		</div>
		<p v-else class="m-0 p-2 text-sm text-secondary">
			{{ formatMessage(messages.empty) }}
		</p>
	</div>
</template>

<script setup lang="ts">
import { defineMessages, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import { injectReviewPanels } from '~/providers/project-review/review-panels'

import IssueCard from './issue-card.vue'

const panels = injectReviewPanels()
const issues = computed(() => {
	const available = new Map(panels.availableIssues.value.map((issue) => [issue.id, issue]))
	return panels.activeIssues.value.flatMap(({ id }) => {
		const issue = available.get(id)
		return issue ? [issue] : []
	})
})
const { formatMessage } = useVIntl()
const messages = defineMessages({
	empty: {
		id: 'project-review.issues.empty',
		defaultMessage: 'No issues flagged.',
	},
})
</script>
