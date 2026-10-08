<template>
	<ClientOnly>
		<ConfirmModal
			ref="resetIssuesModal"
			:title="formatMessage(reviewTabMessages.resetAllIssues)"
			:description="formatMessage(reviewTabMessages.resetIssuesDescription)"
			:proceed-label="formatMessage(reviewTabMessages.resetIssues)"
			@proceed="resetIssues"
		/>
		<ProjectReviewLayout :tabs="visibleTabs" :reset-key="selection">
			<template #left><ProjectInfo /></template>
			<template #right>
				<div
					:key="projectId"
					class="flex h-full min-h-0 min-w-0 flex-col gap-2.5 [&_.cm-editor]:text-xs"
				>
					<ProjectWideChecks :inert="pending" />
					<section class="flex min-h-0 min-w-0 flex-1 flex-col">
						<div class="flex shrink-0 flex-wrap items-center gap-2 pb-2.5">
							<Tabs v-model:value="activeReviewTab" wrap :tabs="rightPanelTabs" />
							<div class="ml-auto flex items-center gap-1">
								<Tooltip
									v-if="!pending && project"
									:text="formatMessage(reviewTabMessages.resetIssues)"
								>
									<Button
										type="quiet"
										size="sm"
										icon-only
										:disabled="resetting"
										:aria-label="formatMessage(reviewTabMessages.resetIssues)"
										@click="resetIssuesModal?.show()"
									>
										<RotateCounterClockwiseIcon aria-hidden="true" />
									</Button>
								</Tooltip>
								<IssuePicker @custom="activeReviewTab = 'issues'" />
							</div>
						</div>
						<IssueList v-show="activeReviewTab === 'issues'" />
						<IssueList v-show="activeReviewTab === 're-review'" re-review />
						<MessageThread v-show="activeReviewTab === 'thread'" ref="messageThread" />
					</section>
					<ReviewOutcomeButtons />
				</div>
			</template>
			<template #footer><QueueBar /></template>
			<template #description><Description :key="projectId" /></template>
			<template #gallery><Gallery :key="projectId" /></template>
			<template #disclosures><Disclosures :key="projectId" /></template>
			<template #permissions><Permissions :key="projectId" /></template>
			<template #versions><Versions :key="projectId" /></template>
			<template #tech-review><TechReview :key="projectId" /></template>
		</ProjectReviewLayout>
		<template #fallback>
			<p class="m-0 p-4 text-secondary" role="status">
				{{ formatMessage(projectReviewMessages.loading) }}
			</p>
		</template>
	</ClientOnly>
</template>

<script setup lang="ts">
import { RotateCounterClockwiseIcon } from '@modrinth/assets'
import {
	Button,
	ConfirmModal,
	defineMessages,
	injectLoadingState,
	Tabs,
	Tooltip,
	useVIntl,
} from '@modrinth/ui'
import { useEventListener } from '@vueuse/core'
import { computed, nextTick, onScopeDispose, ref, watch } from 'vue'

import { useModerationKeybinds } from '~/composables/moderation'
import { isStaff } from '~/helpers/users.js'
import { injectProjectReviewPageContext } from '~/providers/project-review'
import { previousReviewLinkUrls } from '~/providers/project-review/project-links'
import {
	createReviewMessages,
	provideReviewMessages,
} from '~/providers/project-review/review-messages'
import { createReviewPanels, provideReviewPanels } from '~/providers/project-review/review-panels'
import {
	createReviewPreviousIssues,
	provideReviewPreviousIssues,
} from '~/providers/project-review/review-previous-issues'
import {
	createReviewSession,
	provideReviewSession,
} from '~/providers/project-review/review-session'
import {
	createReviewSubmission,
	provideReviewSubmission,
} from '~/providers/project-review/review-submission'

import Description from './description/index.vue'
import Disclosures from './disclosures/index.vue'
import Gallery from './gallery/index.vue'
import IssueList from './issue-list/index.vue'
import IssuePicker from './issue-list/issue-picker.vue'
import { useReReviewIssues } from './issue-list/re-review-issues'
import ProjectReviewLayout from './layout/index.client.vue'
import { type ProjectReviewTab, projectReviewTabs } from './layout/types'
import MessageThread from './message-thread/index.vue'
import { projectReviewMessages } from './messages'
import Permissions from './permissions/index.vue'
import ProjectInfo from './project-info/index.vue'
import ProjectWideChecks from './project-wide-checks.vue'
import QueueBar from './queue-bar.vue'
import ReviewOutcomeButtons from './review-outcome-buttons.vue'
import { createReviewContext, provideReviewContext } from './review-panel/context'
import TechReview from './tech-review/index.vue'
import Versions from './versions/index.vue'

const { formatMessage } = useVIntl()
const {
	projectId,
	project,
	projectV2,
	disclosures,
	members,
	organization,
	organizationMembers,
	wasReviewed,
	threadQuery,
	permissions,
	selection,
	isLoading,
	navigation,
} = injectProjectReviewPageContext()
const visibleTabs = computed<readonly ProjectReviewTab[]>((previousTabs) => {
	if (!project.value) {
		return previousTabs ?? projectReviewTabs.filter((tab) => tab !== 'permissions')
	}
	return projectReviewTabs.filter(
		(tab) => tab !== 'permissions' || project.value?.project_types.includes('modpack'),
	)
})
const reviewProjectId = computed(() => project.value?.id)
const session = provideReviewSession(createReviewSession())
const panels = provideReviewPanels(
	createReviewPanels(
		project,
		session,
		computed(() => ({
			projectV2: projectV2.value,
			disclosures: disclosures.disclosuresQuery.data.value?.disclosures ?? [],
			members: members.value,
			organization: organization.value,
			organizationMembers: organizationMembers.value,
			wasReviewed: wasReviewed.value,
			permissions: permissions.value,
			previousIssueIds: (project.value?.thread_id === threadQuery.data.value?.id
				? (threadQuery.data.value?.issues ?? [])
				: []
			).flatMap(({ why }) =>
				why && typeof why === 'object' && 'issue_id' in why && typeof why.issue_id === 'string'
					? [why.issue_id]
					: [],
			),
			previousLinks: previousReviewLinkUrls(
				wasReviewed.value && project.value?.thread_id === threadQuery.data.value?.id
					? (threadQuery.data.value?.issues ?? [])
					: [],
			),
		})),
	),
)
const messages = provideReviewMessages(createReviewMessages(project, projectV2, panels))
const previousIssues = provideReviewPreviousIssues(
	createReviewPreviousIssues(project, threadQuery.data, wasReviewed, session, panels, messages),
)
const issueCount = computed(() => {
	const previousIds = previousIssues.associatedIssueIds.value
	return (
		panels.activeIssues.value.filter(({ id }) => !previousIds.has(id)).length +
		previousIssues.appliedIssues.value.length
	)
})
const reviewContext = provideReviewContext(
	createReviewContext(reviewProjectId, (target) => !!panels.resolve(target)),
)
const { pending, loadingAction } = provideReviewSubmission(
	createReviewSubmission(messages, panels, session, previousIssues),
)
const { resetting, resetIssues } = useReReviewIssues(
	{ project, threadQuery },
	session,
	messages,
	pending,
)
const loadingState = injectLoadingState()
watch(
	() =>
		isLoading.value ||
		navigation.busy.value ||
		(loadingAction.value !== undefined &&
			loadingAction.value !== 'reply' &&
			loadingAction.value !== 'note'),
	(loading, _, onCleanup) => {
		if (!loading) return
		const token = loadingState.begin()
		onCleanup(() => loadingState.end(token))
	},
	{ immediate: true },
)
const hasPreviousIssues = computed(() => previousIssues.issues.value.length > 0)
const activeReviewTab = ref('thread')
onScopeDispose(
	reviewContext.registerRoute('re-review', () => {
		activeReviewTab.value = hasPreviousIssues.value ? 're-review' : 'thread'
	}),
)
const resetIssuesModal = ref<InstanceType<typeof ConfirmModal>>()
const messageThread = ref<InstanceType<typeof MessageThread>>()
const auth = useAuthState()
const keybinds = useModerationKeybinds()
const reviewTabMessages = defineMessages({
	issues: {
		id: 'project-review.right-panel.issues',
		defaultMessage: 'Issues ({count})',
	},
	thread: { id: 'project-review.right-panel.thread', defaultMessage: 'Thread' },
	reReview: {
		id: 'project-review.right-panel.re-review',
		defaultMessage: 'Re-review ({count})',
	},
	resetIssues: {
		id: 'project-review.issues.reset',
		defaultMessage: 'Reset issues',
	},
	resetAllIssues: {
		id: 'project-review.issues.reset-all',
		defaultMessage: 'Reset all issues',
	},
	resetIssuesDescription: {
		id: 'project-review.issues.reset-description',
		defaultMessage:
			'This will discard your issue changes and move all previous issues back to Re-rev. issues.',
	},
})

const rightPanelTabs = computed(() => [
	...(hasPreviousIssues.value
		? [
				{
					value: 're-review',
					label: formatMessage(reviewTabMessages.reReview, {
						count: previousIssues.reReviewIssues.value.length,
					}),
				},
			]
		: []),
	{ value: 'thread', label: formatMessage(reviewTabMessages.thread) },
	{
		value: 'issues',
		label: formatMessage(reviewTabMessages.issues, { count: issueCount.value }),
	},
])

watch(
	[projectId, hasPreviousIssues],
	([, hasIssues]) => {
		activeReviewTab.value = hasIssues ? 're-review' : 'thread'
	},
	{ immediate: true },
)

watch(projectId, () => {
	resetIssuesModal.value?.hide()
})

async function openEditor() {
	activeReviewTab.value = 'thread'
	await nextTick()
	await messageThread.value?.openEditor()
}

useEventListener('keydown', (event) => {
	if (!isStaff(auth.value.user) || event.defaultPrevented || event.repeat || event.isComposing)
		return
	const target = event.target
	if (
		target instanceof HTMLElement &&
		(target.isContentEditable ||
			target.closest('input, textarea, select, [role="textbox"], [role="dialog"]'))
	)
		return
	keybinds.value.handle(event, { scope: 'review-conversation', openEditor })
})
</script>
