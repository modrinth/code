<template>
	<div v-if="project" class="flex shrink-0 flex-wrap gap-2">
		<Button
			v-for="action in actions"
			:key="action.status"
			class="grow"
			:type="action.type"
			:color="action.color"
			:disabled="!actionAvailable(action.status)"
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
import { moderationSettings } from '@modrinth/moderation'
import { Button, defineMessages, useVIntl } from '@modrinth/ui'
import { computed, ref } from 'vue'

import { useModerationSettings } from '~/composables/moderation'
import { injectProjectReviewPageContext } from '~/providers/project-review'
import { injectReviewMessages } from '~/providers/project-review/review-messages'
import { injectReviewSubmission } from '~/providers/project-review/review-submission'

import { useReviewShortcut } from './shortcuts'

const { project, navigation, queue } = injectProjectReviewPageContext()
const settings = useModerationSettings()
const { canSubmit, canApprove, loadingAction, submitDecision } = injectReviewSubmission()
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
		status: ['approved', 'unlisted', 'private'].includes(project.value?.requested_status ?? '')
			? project.value!.requested_status!
			: ('approved' as const),
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

function actionAvailable(status: Parameters<typeof submitDecision>[0]) {
	return (
		canSubmit.value &&
		!generating.value &&
		advancingAction.value === undefined &&
		project.value?.status !== status &&
		(!['approved', 'unlisted', 'private'].includes(status) || canApprove.value)
	)
}
for (const [index, action] of (['approve', 'withhold', 'reject'] as const).entries()) {
	useReviewShortcut(
		action,
		() => {
			void submitDecisionAndContinue(actions.value[index].status)
		},
		() => actionAvailable(actions.value[index].status),
	)
}

async function submitDecisionAndContinue(status: Parameters<typeof submitDecision>[0]) {
	if (!actionAvailable(status)) return
	const id = project.value?.id
	if (!id) return
	advancingAction.value = status
	try {
		if (await submitDecision(status)) {
			if (settings.value.get(moderationSettings.General.AutoGoNextOnReviewOutcome)) {
				await navigation.completeAndNext(id)
			} else {
				await queue.completeProject(id)
			}
		}
	} finally {
		advancingAction.value = undefined
	}
}
</script>
