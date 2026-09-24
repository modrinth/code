<template>
	<div v-if="project" class="flex shrink-0 flex-wrap gap-2">
		<Button
			v-for="action in actions"
			:key="action.status"
			class="grow"
			size="sm"
			:type="action.type"
			:color="action.color"
			:disabled="!canSubmit || generating || project.status === action.status"
			@click="submitDecision(action.status)"
		>
			<SpinnerIcon v-if="loadingAction === action.status" class="animate-spin" aria-hidden="true" />
			{{ formatMessage(action.label) }}
		</Button>
	</div>
</template>

<script setup lang="ts">
import { SpinnerIcon } from '@modrinth/assets'
import { Button, defineMessages, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'
import { injectReviewMessages } from '~/providers/project-review/review-messages'
import { injectReviewSubmission } from '~/providers/project-review/review-submission'

const { project } = injectProjectReviewPageContext()
const { canSubmit, loadingAction, submitDecision } = injectReviewSubmission()
const { generating } = injectReviewMessages()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	approve: { id: 'project-review.decision.approve', defaultMessage: 'Approve' },
	withhold: {
		id: 'project-review.decision.withhold',
		defaultMessage: 'Withhold',
	},
	reject: { id: 'project-review.decision.reject', defaultMessage: 'Reject' },
})
const actions = computed(() => [
	{
		status: project.value?.requested_status ?? 'approved',
		label: messages.approve,
		type: 'colored' as const,
		color: 'green' as const,
	},
	{
		status: 'withheld' as const,
		label: messages.withhold,
		type: 'outlined' as const,
		color: 'orange' as const,
	},
	{
		status: 'rejected' as const,
		label: messages.reject,
		type: 'outlined' as const,
		color: 'red' as const,
	},
])
</script>
