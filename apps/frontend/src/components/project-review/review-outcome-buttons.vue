<template>
	<div v-if="project" class="flex shrink-0 flex-wrap gap-2">
		<Button
			v-for="action in actions"
			:key="action.status"
			class="grow"
			size="sm"
			:type="action.type"
			:color="action.color"
			:disabled="
				!canSubmit ||
				generating ||
				advancingAction !== undefined ||
				(project.status === action.status && pendingDecisionStatus !== action.status)
			"
			@click="submitDecisionAndContinue(action.status)"
		>
			<SpinnerIcon
				v-if="loadingAction === action.status || advancingAction === action.status"
				class="animate-spin"
				aria-hidden="true"
			/>
			{{ formatMessage(action.label) }}
		</Button>
	</div>
</template>

<script setup lang="ts">
import { SpinnerIcon } from '@modrinth/assets'
import { Button, defineMessages, useVIntl } from '@modrinth/ui'
import { computed, ref } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'
import { injectReviewMessages } from '~/providers/project-review/review-messages'
import { injectReviewSubmission } from '~/providers/project-review/review-submission'

const { project, navigation } = injectProjectReviewPageContext()
const { canSubmit, loadingAction, pendingDecisionStatus, submitDecision } =
	injectReviewSubmission()
const { generating } = injectReviewMessages()
const advancingAction = ref<Parameters<typeof submitDecision>[0]>()
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

async function submitDecisionAndContinue(status: Parameters<typeof submitDecision>[0]) {
	const id = project.value?.id
	if (!id) return
	advancingAction.value = status
	try {
		if (await submitDecision(status)) await navigation.completeAndNext(id)
	} finally {
		advancingAction.value = undefined
	}
}
</script>
