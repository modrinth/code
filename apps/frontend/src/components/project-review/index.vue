<template>
	<ClientOnly>
		<ConfirmModal
			ref="clearIssuesModal"
			:title="formatMessage(reviewTabMessages.clearAllIssues)"
			:description="formatMessage(reviewTabMessages.clearAllIssuesDescription)"
			:proceed-label="formatMessage(reviewTabMessages.clearIssues)"
			@proceed="clearIssues"
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
						<div class="flex shrink-0 flex-wrap items-center justify-between gap-2 pb-2.5">
							<Tabs
								v-model:value="activeReviewTab"
								wrap
								:tabs="[
									{ value: 'thread', label: formatMessage(reviewTabMessages.thread) },
									{
										value: 'issues',
										label: formatMessage(reviewTabMessages.issues, {
											count: panels.activeIssues.value.length,
										}),
									},
								]"
							/>
							<div v-if="activeReviewTab === 'issues'" class="flex items-center gap-1">
								<Tooltip
									v-if="!pending && panels.activeIssues.value.length"
									:text="formatMessage(reviewTabMessages.clearIssues)"
								>
									<Button
										type="quiet"
										size="sm"
										icon-only
										:aria-label="formatMessage(reviewTabMessages.clearIssues)"
										@click="clearIssuesModal?.show()"
									>
										<RotateCounterClockwiseIcon aria-hidden="true" />
									</Button>
								</Tooltip>
								<IssuePicker />
							</div>
						</div>
						<IssueList v-show="activeReviewTab === 'issues'" />
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
			<template #history><History :key="projectId" /></template>
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
import { computed, nextTick, ref, watch } from 'vue'

import { useModerationKeybinds } from '~/composables/moderation'
import { isStaff } from '~/helpers/users.js'
import { injectProjectReviewPageContext } from '~/providers/project-review'
import {
	createReviewMessages,
	provideReviewMessages,
} from '~/providers/project-review/review-messages'
import { createReviewPanels, provideReviewPanels } from '~/providers/project-review/review-panels'
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
import History from './history/index.vue'
import IssueList from './issue-list/index.vue'
import IssuePicker from './issue-list/issue-picker.vue'
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
		})),
	),
)
const messages = provideReviewMessages(createReviewMessages(project, projectV2, panels))
provideReviewContext(createReviewContext(reviewProjectId, (target) => !!panels.resolve(target)))
const { pending, loadingAction } = provideReviewSubmission(
	createReviewSubmission(messages, panels, session),
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
const activeReviewTab = ref('thread')
const clearIssuesModal = ref<InstanceType<typeof ConfirmModal>>()
const messageThread = ref<InstanceType<typeof MessageThread>>()
const auth = useAuthState()
const keybinds = useModerationKeybinds()
const reviewTabMessages = defineMessages({
	issues: { id: 'project-review.right-panel.issues', defaultMessage: 'Issues ({count})' },
	thread: { id: 'project-review.right-panel.thread', defaultMessage: 'Thread' },
	clearIssues: { id: 'project-review.issues.clear', defaultMessage: 'Clear issues' },
	clearAllIssues: {
		id: 'project-review.issues.clear-all',
		defaultMessage: 'Clear all issues',
	},
	clearAllIssuesDescription: {
		id: 'project-review.issues.clear-all-description',
		defaultMessage: 'This will remove all issues you currently have selected in the project.',
	},
})

watch(projectId, () => {
	activeReviewTab.value = 'thread'
	clearIssuesModal.value?.hide()
})

function clearIssues() {
	if (pending.value) return
	for (const { id } of panels.activeIssues.value) {
		panels.removeIssue(id)
		messages.resetIssueMessage(id)
	}
}

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
