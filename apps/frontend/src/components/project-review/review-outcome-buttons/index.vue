<template>
	<ModerationLockedCard
		v-if="project && heldBy"
		:disabled="pending || advancingAction !== undefined"
	/>
	<div v-else-if="project" class="flex shrink-0 flex-wrap gap-1.5">
		<span v-if="sendIssuesAsThreadMessage" class="w-full text-xs text-secondary">
			Send issues as thread message
		</span>
		<Tooltip
			v-for="action in actions"
			:key="action.status"
			:text="actionTooltip(action.status)"
			:disabled="!actionTooltip(action.status)"
			class="flex grow"
		>
			<Button
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
				<ShieldCheckIcon v-else-if="action.status === 'tech-review-pass'" aria-hidden="true" />
				<ShieldAlertIcon
					v-else-if="techReviewMode && action.status === 'rejected'"
					aria-hidden="true"
				/>
				{{ action.label }}
			</Button>
		</Tooltip>
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
	ShieldAlertIcon,
	ShieldCheckIcon,
	SpinnerIcon,
	XIcon,
} from '@modrinth/assets'
import { moderationSettings } from '@modrinth/moderation'
import { Button, TeleportOverflowMenu, Tooltip } from '@modrinth/ui'
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
	approvalDisabledReason,
	techReviewMode,
	canPassTechReview,
	techReviewPassDisabledReason,
	canRejectTechReview,
	techReviewRejectDisabledReason,
	submitTechReviewPass,
	submitProjectReject,
	canAddIssues,
	sendIssuesAsThreadMessage,
	pending,
	loadingAction,
	submitDecision,
	submitIssues,
} = injectReviewSubmission()
const { generating } = injectReviewMessages()
type ReviewOutcome =
	| Parameters<typeof submitDecision>[0]
	| 'tech-review-pass'
	| 'project-reject'
const advancingAction = ref<ReviewOutcome>()

const standardActions = computed(() => [
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
const actions = computed(() =>
	techReviewMode.value
		? [
				{
					status: 'tech-review-pass' as const,
					label: 'Pass',
					type: 'colored' as const,
					color: 'green' as const,
				},
				{
					...standardActions.value[2],
					label: 'Fail',
					type: 'colored' as const,
				},
			]
		: standardActions.value,
)

function actionTooltip(status: ReviewOutcome) {
	if (status === 'tech-review-pass') return techReviewPassDisabledReason.value
	if (status === 'rejected' && techReviewMode.value) return techReviewRejectDisabledReason.value
	if (['approved', 'unlisted', 'private'].includes(status)) return approvalDisabledReason.value
	return undefined
}

const overflowActions = computed(() => [
	...(techReviewMode.value
		? [
				{
					id: 'reject',
					label: 'Reject',
					tone: 'red' as const,
					icon: XIcon,
					hoverFilled: true,
					action: () => submitDecisionAndContinue('project-reject'),
					disabled: !actionAvailable('project-reject'),
				},
			]
		: []),
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

function actionAvailable(status: ReviewOutcome) {
	if (status === 'tech-review-pass')
		return (
			techReviewMode.value &&
			!heldBy.value &&
			canPassTechReview.value &&
			!generating.value &&
			advancingAction.value === undefined
		)
	if (status === 'project-reject')
		return (
			techReviewMode.value &&
			!heldBy.value &&
			canSubmit.value &&
			!generating.value &&
			advancingAction.value === undefined &&
			project.value?.status !== 'rejected'
		)
	if (status === 'withheld' && techReviewMode.value) return false
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
		(project.value?.status !== status || (status === 'rejected' && techReviewMode.value)) &&
		(status !== 'rejected' || !techReviewMode.value || canRejectTechReview.value) &&
		(!['approved', 'unlisted', 'private'].includes(status) || canApprove.value)
	)
}
for (const [index, action] of (['approve', 'withhold', 'reject'] as const).entries()) {
	useReviewShortcut(
		action,
		() => {
			void submitDecisionAndContinue(standardActions.value[index].status)
		},
		() =>
			actionAvailable(standardActions.value[index].status) &&
			(action !== 'approve' || !techReviewMode.value),
	)
}

async function submitDecisionAndContinue(status: ReviewOutcome, advance = true) {
	if (!actionAvailable(status) || advancingAction.value !== undefined) return
	const id = project.value?.id
	if (!id) return
	advancingAction.value = status
	try {
		const submitted = await (status === 'tech-review-pass'
			? submitTechReviewPass()
			: status === 'project-reject'
				? submitProjectReject()
				: submitDecision(status))
		if (submitted) {
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
