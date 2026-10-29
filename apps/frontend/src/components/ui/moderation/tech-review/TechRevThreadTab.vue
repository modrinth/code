<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	CheckIcon,
	DropdownIcon,
	EyeOffIcon,
	ScaleIcon,
	ShieldAlertIcon,
	ShieldCheckIcon,
	SpinnerIcon,
	XIcon,
} from '@modrinth/assets'
import { type TechReviewContext, techReviewQuickReplies } from '@modrinth/moderation'
import {
	Button,
	type ButtonMenuOption,
	CollapsibleRegion,
	commonMessages,
	injectModrinthClient,
	injectNotificationManager,
	TeleportOverflowMenu,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import type { UnsafeFile } from '~/components/ui/moderation/MaliciousSummaryModal.vue'
import ThreadView from '~/components/ui/thread/ThreadView.vue'

import type { FlattenedFileReport } from './types'
import { injectTechReviewDecisions } from './use-tech-review-decisions'

const props = defineProps<{
	project: Labrinth.Projects.v3.Project
	projectOwner: Labrinth.TechReview.Internal.Ownership
	thread: Labrinth.TechReview.Internal.Thread
	reports: FlattenedFileReport[]
	disableCollapsing?: boolean
}>()

const isThreadCollapsed = defineModel<boolean>('collapsed', { required: true })

const emit = defineEmits<{
	refetch: []
	markComplete: [projectId: string]
	showMaliciousSummary: [unsafeFiles: UnsafeFile[]]
	statusChanged: [status: Labrinth.Projects.v2.ProjectStatus]
}>()

const featureFlags = useFeatureFlags()
const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const { getDetailDecision } = injectTechReviewDecisions()

const remainingMessageCount = computed(() => {
	if (!props.thread?.messages) return 0
	return Math.max(0, props.thread.messages.length - 1)
})

const threadExpandText = computed(() => {
	if (remainingMessageCount.value === 0) return 'Expand'
	if (remainingMessageCount.value === 1) return 'Show 1 more message'
	return `Show ${remainingMessageCount.value} more messages`
})

const projectStatus = ref<Labrinth.Projects.v2.ProjectStatus>(props.project.status)
const isLoadingStatusAction = ref(false)

function isStatusActionDisabled(status: Labrinth.Projects.v2.ProjectStatus): boolean {
	return projectStatus.value === status || isLoadingStatusAction.value
}

async function setStatus(status: Labrinth.Projects.v2.ProjectStatus) {
	isLoadingStatusAction.value = true
	try {
		await client.labrinth.projects_v2.edit(props.project.id, { status })
		emit('refetch')
		projectStatus.value = status
		emit('statusChanged', status)
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: (err as any)?.data?.description ? (err as any).data.description : String(err),
			type: 'error',
		})
	}
	isLoadingStatusAction.value = false
}

const projectStatusActions = computed<ButtonMenuOption[]>(() => [
	{
		id: 'approve',
		label: 'Approve',
		icon: CheckIcon,
		tone: 'green',
		hoverFilled: true,
		action: () => setStatus('approved'),
		disabled: isStatusActionDisabled('approved'),
	},
	{
		id: 'withhold',
		label: 'Withhold',
		icon: EyeOffIcon,
		tone: 'orange',
		hoverFilled: true,
		action: () => setStatus('withheld'),
		disabled: isStatusActionDisabled('withheld'),
	},
	{
		id: 'send-to-review',
		label: 'Send to review',
		icon: ScaleIcon,
		action: () => setStatus('processing'),
		disabled: isStatusActionDisabled('processing'),
	},
	{
		id: 'reject',
		label: 'Reject',
		icon: XIcon,
		tone: 'red',
		hoverFilled: true,
		action: () => setStatus('rejected'),
		disabled: isStatusActionDisabled('rejected'),
	},
])

const techReviewContext = computed<TechReviewContext>(() => ({
	project: props.project,
	project_owner: props.projectOwner,
	reports: props.reports,
}))

const threadViewRef = ref<{
	setReplyContent: (content: string) => void
	getReplyContent: () => string
} | null>(null)

const unsafeFiles = computed<UnsafeFile[]>(() => {
	return props.reports
		.filter((report) =>
			report.issues.some((issue) =>
				issue.details.some((detail) => getDetailDecision(detail.id, detail.status) === 'malware'),
			),
		)
		.map((report) => ({
			file: report,
			projectName: props.project.name,
			projectId: props.project.id,
			userId: props.projectOwner.id,
			username: props.projectOwner.name,
		}))
})

const allIssuesResolved = computed(() => {
	for (const report of props.reports) {
		for (const issue of report.issues) {
			for (const detail of issue.details) {
				if (getDetailDecision(detail.id, detail.status) === 'pending') return false
			}
		}
	}
	return true
})

const canSubmitReview = computed(() => {
	const totalIssues = props.reports.reduce((sum, r) => sum + r.issues.length, 0)
	if (totalIssues === 0) return true
	return allIssuesResolved.value
})
const hasSubmittedPassReview = ref(false)

async function handleSubmitReview(verdict: 'safe' | 'unsafe') {
	hasSubmittedPassReview.value = verdict === 'safe'
	const editorContent = threadViewRef.value?.getReplyContent() || ''

	try {
		await client.labrinth.tech_review_internal.submitProject(props.project.id, {
			verdict,
			message: editorContent || undefined,
		})
		emit('markComplete', props.project.id)
		addNotification({
			type: 'success',
			title: 'Review submitted',
			text: 'Technical review completed successfully.',
		})

		if (verdict === 'unsafe') {
			emit('showMaliciousSummary', unsafeFiles.value)
		}
	} catch (error: unknown) {
		const err = error as { response?: { data?: { issues?: string[] } } }
		if (err.response?.data?.issues) {
			const missedCount = err.response.data.issues.length
			addNotification({
				type: 'error',
				title: 'Pending issues remain',
				text: `${missedCount} issue(s) still need a verdict before submitting.`,
			})
		} else {
			addNotification({
				type: 'error',
				title: 'Submit failed',
				text: 'Failed to submit review. Please try again.',
			})
		}
	}
}
</script>

<template>
	<CollapsibleRegion
		v-model:collapsed="isThreadCollapsed"
		:expand-text="threadExpandText"
		:disabled="disableCollapsing"
		collapse-text="Collapse thread"
	>
		<div class="bg-surface-2 pt-0">
			<!-- DEV-531 -->
			<!-- @vue-expect-error TODO: will convert ThreadView to use api-client types at a later date -->
			<ThreadView
				ref="threadViewRef"
				:thread="thread"
				:quick-replies="techReviewQuickReplies"
				:quick-reply-context="techReviewContext"
				primary-action="note"
				@update-thread="emit('refetch')"
			>
				<template #additionalActions>
					<Button
						v-tooltip="
							!canSubmitReview
								? 'There are still pending flags!'
								: hasSubmittedPassReview
									? 'Project already passed!'
									: undefined
						"
						type="colored"
						color="brand"
						:disabled="!canSubmitReview || hasSubmittedPassReview"
						@click="handleSubmitReview('safe')"
					>
						<ShieldCheckIcon /> Pass
					</Button>
					<Button
						v-tooltip="!canSubmitReview ? 'There are still pending flags!' : undefined"
						type="colored"
						color="red"
						:disabled="!canSubmitReview"
						@click="handleSubmitReview('unsafe')"
					>
						<ShieldAlertIcon /> Fail
					</Button>
					<TeleportOverflowMenu
						label="More options"
						class="btn-dropdown-animation !w-auto !rounded-xl !px-2.5"
						:disabled="isLoadingStatusAction"
						:options="projectStatusActions"
					>
						<SpinnerIcon v-if="isLoadingStatusAction" class="animate-spin" aria-hidden="true" />
						<ScaleIcon v-else aria-hidden="true" />
						Set status
						<DropdownIcon aria-hidden="true" />
					</TeleportOverflowMenu>
					<Button
						v-if="featureFlags.developerMode"
						type="outlined"
						@click="emit('showMaliciousSummary', unsafeFiles)"
						>Debug</Button
					>
				</template>
			</ThreadView>
		</div>
	</CollapsibleRegion>
</template>
