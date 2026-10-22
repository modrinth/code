<template>
	<ModerationLockedCard
		v-if="project && heldBy"
		:disabled="pending || advancingAction !== undefined"
	/>
	<div v-else-if="project" class="flex shrink-0 flex-wrap gap-1.5">
		<span v-if="sendIssuesAsThreadMessage" class="w-full text-xs text-secondary">
			Send issues as thread message
		</span>
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
			{{ action.label }}
		</Button>
		<TeleportOverflowMenu label="More options" :options="overflowActions" :circular="false">
			<MoreHorizontalIcon aria-hidden="true" />
			<template #send-to-review>
				<ScaleIcon aria-hidden="true" />
				Send to review
			</template>
			<template #set-to-draft>
				<FileTextIcon aria-hidden="true" />
				Set to draft
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
import { Button, TeleportOverflowMenu } from '@modrinth/ui'
import { computed, ref } from 'vue'

import { useModerationSettings } from '~/composables/moderation'
import { injectProjectReviewPageContext } from '~/providers/project-review'
import { injectReviewMessages } from '~/providers/project-review/review-messages'
import { injectReviewSubmission } from '~/providers/project-review/review-submission'

import { useReviewShortcut } from '../shortcuts'
import ModerationLockedCard from './moderation-locked-card.vue'

const { project, navigation, queue, lock } = injectProjectReviewPageContext()
const { heldBy } = lock
const settings = useModerationSettings()
const {
	canSubmit,
	canApprove,
	canAddIssues,
	sendIssuesAsThreadMessage,
	pending,
	loadingAction,
	submitDecision,
	submitIssues,
} = injectReviewSubmission()
const { generating } = injectReviewMessages()
const advancingAction = ref<Parameters<typeof submitDecision>[0]>()

const actions = computed(() => [
	{
		status: ['approved', 'unlisted', 'private'].includes(project.value?.requested_status ?? '')
			? project.value!.requested_status!
			: ('approved' as const),
		label: 'Approve',
		type: 'colored' as const,
		color: 'green' as const,
	},
	{
		status: 'withheld' as const,
		label: 'Withhold',
		type: 'outlined' as const,
		color: 'orange' as const,
	},
	{
		status: 'rejected' as const,
		label: 'Reject',
		type: 'outlined' as const,
		color: 'red' as const,
	},
])
const overflowActions = computed(() => [
	{
		id: 'add-issues',
		label: sendIssuesAsThreadMessage.value ? 'Send issues thread message' : 'Add issues on project',
		tone: 'orange' as const,
		icon: PlusIcon,
		action: () => submitIssues(),
		hoverFilled: true,
		disabled:
			!!heldBy.value ||
			!canAddIssues.value ||
			generating.value ||
			advancingAction.value !== undefined,
	},
	{
		id: 'send-to-review',
		label: 'Send to review',
		tone: 'orange' as const,
		hoverFilled: true,
		action: () => submitDecisionAndContinue('processing', false),
		disabled: !actionAvailable('processing'),
	},
	{
		id: 'set-to-draft',
		label: 'Set to draft',
		tone: 'orange' as const,
		hoverFilled: true,
		action: () => submitDecisionAndContinue('draft', false),
		disabled: !actionAvailable('draft'),
	},
])

function actionAvailable(status: Parameters<typeof submitDecision>[0]) {
	if (heldBy.value) return false
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
