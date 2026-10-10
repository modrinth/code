<template>
	<div v-if="project" class="flex shrink-0 flex-wrap gap-1.5">
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
		<TeleportOverflowMenu
			:label="formatMessage(commonMessages.moreOptionsButton)"
			:options="overflowActions"
			:circular="false"
		>
			<MoreHorizontalIcon aria-hidden="true" />
			<template #send-to-review>
				<ScaleIcon aria-hidden="true" />
				{{ formatMessage(messages.sendToReview) }}
			</template>
			<template #set-to-draft>
				<FileTextIcon aria-hidden="true" />
				{{ formatMessage(messages.setToDraft) }}
			</template>
		</TeleportOverflowMenu>
	</div>
</template>

<script setup lang="ts">
import {
	FileTextIcon,
	MoreHorizontalIcon,
	PlusIcon,
	ScaleIcon,
	SpinnerIcon,
} from '@modrinth/assets'
import { moderationSettings } from '@modrinth/moderation'
import {
	Button,
	commonMessages,
	defineMessages,
	TeleportOverflowMenu,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import { useModerationSettings } from '~/composables/moderation'
import { injectProjectReviewPageContext } from '~/providers/project-review'
import { injectReviewMessages } from '~/providers/project-review/review-messages'
import { injectReviewSubmission } from '~/providers/project-review/review-submission'

import { useReviewShortcut } from './shortcuts'

const { project, navigation, queue } = injectProjectReviewPageContext()
const settings = useModerationSettings()
const {
	canSubmit,
	canApprove,
	canAddIssues,
	pending,
	loadingAction,
	submitDecision,
	submitIssues,
} = injectReviewSubmission()
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
	sendToReview: {
		id: 'project-review.decision.send-to-review',
		defaultMessage: 'Send to review',
	},
	setToDraft: { id: 'project-review.decision.set-to-draft', defaultMessage: 'Set to draft' },
	addIssues: {
		id: 'project-review.decision.add-issues',
		defaultMessage: 'Add issues on project',
	},
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
const overflowActions = computed(() => [
	{
		id: 'add-issues',
		label: formatMessage(messages.addIssues),
		icon: PlusIcon,
		action: () => submitIssues(),
		disabled: !canAddIssues.value || generating.value || advancingAction.value !== undefined,
	},
	{
		id: 'send-to-review',
		label: formatMessage(messages.sendToReview),
		tone: 'orange' as const,
		hoverFilled: true,
		action: () => submitDecisionAndContinue('processing', false),
		disabled: !actionAvailable('processing'),
	},
	{
		id: 'set-to-draft',
		label: formatMessage(messages.setToDraft),
		tone: 'orange' as const,
		hoverFilled: true,
		action: () => submitDecisionAndContinue('draft', false),
		disabled: !actionAvailable('draft'),
	},
])

function actionAvailable(status: Parameters<typeof submitDecision>[0]) {
	if (status === 'processing')
		return (
			project.value?.status !== 'processing' &&
			!pending.value &&
			advancingAction.value === undefined
		)
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

async function submitDecisionAndContinue(
	status: Parameters<typeof submitDecision>[0],
	advance = true,
) {
	if (!actionAvailable(status) || advancingAction.value !== undefined) return
	const id = project.value?.id
	if (!id) return
	advancingAction.value = status
	try {
		if (await submitDecision(status)) {
			if (advance && settings.value.get(moderationSettings.General.AutoGoNextOnReviewOutcome)) {
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
