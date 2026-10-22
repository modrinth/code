<template>
	<ClientOnly>
		<NewModal
			ref="resetIssuesModal"
			header="Reset all issues"
			max-width="800px"
			danger
			:initial-focus="() => resetCancelButton?.element ?? null"
		>
			<div class="flex max-w-[35rem] flex-col gap-4">
				<p class="m-0">
					This will discard your issue changes and move all previous issues back to Re-rev. issues.
				</p>
				<p class="m-0">
					Revert all customizations restores default issue messages while keeping custom issue
					messages and all issue toggles and selections.
				</p>
			</div>
			<template #actions>
				<div class="flex flex-wrap justify-end gap-2">
					<Button ref="resetCancelButton" @click="resetIssuesModal?.hide()">
						<XIcon />
						Cancel
					</Button>
					<Button
						type="colored"
						color="orange"
						:disabled="pending || resetting"
						@click="revertCustomizations"
					>
						Revert all customizations
					</Button>
					<Button
						type="colored"
						color="red"
						:disabled="pending || resetting"
						@click="resetAllIssues"
					>
						<TrashIcon />
						Reset issues
					</Button>
				</div>
			</template>
		</NewModal>
		<ProjectReviewLayout
			ref="layout"
			:tabs="visibleTabs"
			:side-panels="sidePanels"
			:counts="panelCounts"
			:reset-key="selection"
			:ready="layoutReady"
		>
			<template #project><ProjectInfo /></template>
			<template #rulings>
				<div :key="projectId" class="flex h-full min-h-0 min-w-0 flex-col">
					<ProjectWideChecks :inert="pending" />
				</div>
			</template>
			<template #issues>
				<ConversationPanel
					:key="projectId"
					:hosts-outcome="outcomeHost === 'issues'"
					class="[&_.cm-editor]:text-xs"
					@shown="(shown) => setConversationShown('issues', shown)"
				>
					<div class="flex shrink-0 items-center justify-end gap-1 pb-2">
						<Tooltip v-if="!pending && project" text="Reset issues">
							<Button
								type="quiet"
								size="sm"
								icon-only
								:disabled="resetting"
								aria-label="Reset issues"
								@click="resetIssuesModal?.show()"
							>
								<RotateCounterClockwiseIcon aria-hidden="true" />
							</Button>
						</Tooltip>
						<IssuePicker ref="issuePicker" @selected="reveal('issues')" />
					</div>
					<IssueList />
				</ConversationPanel>
			</template>
			<template #re-review>
				<ConversationPanel
					:key="projectId"
					:hosts-outcome="outcomeHost === 're-review'"
					class="[&_.cm-editor]:text-xs"
					@shown="(shown) => setConversationShown('re-review', shown)"
				>
					<IssueList re-review />
				</ConversationPanel>
			</template>
			<template #thread>
				<ConversationPanel
					:key="projectId"
					:hosts-outcome="outcomeHost === 'thread'"
					@shown="(shown) => setConversationShown('thread', shown)"
				>
					<MessageThread />
				</ConversationPanel>
			</template>
			<template #footer><QueueBar /></template>
			<template #settings="{ floating }"><ReviewSettings :floating="floating" /></template>
			<template #description><Description :key="projectId" /></template>
			<template #gallery><Gallery :key="projectId" /></template>
			<template #disclosures><Disclosures :key="projectId" /></template>
			<template #permissions><Permissions :key="projectId" /></template>
			<template #versions><Versions :key="projectId" /></template>
			<template #tech-review><TechReview :key="projectId" /></template>
			<template #archive-source="{ projectShown }">
				<ArchiveProjectDetails
					v-if="project"
					:project="project"
					:versions="versions"
					:version="archive.version.value ?? null"
					:file="archive.file.value ?? null"
					:hide-project="projectShown"
					@select-version="archive.selectVersion"
					@select-file="archive.selectFile"
				/>
			</template>
		</ProjectReviewLayout>
		<template #fallback>
			<p class="m-0 p-4 text-secondary" role="status">Loading...</p>
		</template>
	</ClientOnly>
</template>

<script setup lang="ts">
import { RotateCounterClockwiseIcon, TrashIcon, XIcon } from '@modrinth/assets'
import { moderationSettings } from '@modrinth/moderation'
import { Button, injectLoadingState, NewModal, Tooltip } from '@modrinth/ui'
import { computed, onScopeDispose, ref, shallowReactive, watch } from 'vue'

import ArchiveProjectDetails from '~/components/ui/archive-viewer/ArchiveProjectDetails.vue'
import {
	notifyCopied,
	useModerationKeybinds,
	useModerationSettings,
} from '~/composables/moderation'
import { isStaff } from '~/helpers/users.js'
import { injectProjectReviewPageContext } from '~/providers/project-review'
import { previousReviewLinkUrls } from '~/providers/project-review/project-links'
import {
	createReviewArchive,
	provideReviewArchive,
} from '~/providers/project-review/review-archive'
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

import ConversationPanel from './conversation-panel.vue'
import Description from './description/index.vue'
import Disclosures from './disclosures/index.vue'
import Gallery from './gallery/index.vue'
import IssueList from './issue-list/index.vue'
import IssuePicker from './issue-list/issue-picker.vue'
import { useReReviewIssues } from './issue-list/re-review-issues'
import ProjectReviewLayout from './layout/index.client.vue'
import {
	type ProjectReviewSidePanel,
	type ProjectReviewSlot,
	type ProjectReviewTab,
	projectReviewTabs,
} from './layout/types'
import MessageThread from './message-thread/index.vue'
import Permissions from './permissions/index.vue'
import ProjectInfo from './project-info/index.vue'
import ProjectWideChecks from './project-wide-checks.vue'
import QueueBar from './queue-bar.vue'
import { createReviewContext, provideReviewContext } from './review-panel/context'
import ReviewSettings from './review-settings.vue'
import { createReviewShortcuts, provideReviewShortcuts, reviewShortcutBlocked } from './shortcuts'
import TechReview from './tech-review/index.vue'
import Versions from './versions/index.vue'

const shortcuts = provideReviewShortcuts(createReviewShortcuts())
function useShortcut(...args: Parameters<typeof shortcuts.register>) {
	onScopeDispose(shortcuts.register(...args))
}

const {
	projectId,
	project,
	projectV2,
	versions,
	versionsV2,
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
const layoutReady = computed(() => !!project.value && !isLoading.value && !navigation.busy.value)
const archive = provideReviewArchive(createReviewArchive(projectId, versions))
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
const settings = useModerationSettings()
const panels = provideReviewPanels(
	createReviewPanels(
		project,
		session,
		computed(() => ({
			projectV2: projectV2.value,
			versionsV3: versions.value,
			versionsV2: versionsV2.value,
			disclosures: disclosures.disclosuresQuery.data.value?.disclosures ?? [],
			members: members.value,
			organization: organization.value,
			organizationMembers: organizationMembers.value,
			wasReviewed: wasReviewed.value,
			hasPreviousStructuredIssues:
				wasReviewed.value &&
				project.value?.thread_id === threadQuery.data.value?.id &&
				(threadQuery.data.value?.issues?.length ?? 0) > 0,
			sendIssuesAsThreadMessage: settings.value.get(
				moderationSettings.General.SendIssuesAsThreadMessage,
			),
			showAllIssueControls: settings.value.get(moderationSettings.General.ShowAllIssueControls),
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
	createReviewContext(reviewProjectId, (target) => !!panels.resolve(target), shortcuts),
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
const hasPreviousStructuredIssues = computed(() => previousIssues.issues.value.length > 0)
const layout = ref<InstanceType<typeof ProjectReviewLayout>>()

/** The conversation's panels, of which the re-review only shows once there are previous issues. */
const conversationPanels = computed<ProjectReviewSidePanel[]>(() => [
	...(hasPreviousStructuredIssues.value ? (['re-review'] as const) : []),
	'thread',
	'issues',
])
const sidePanels = computed<ProjectReviewSidePanel[]>(() => [
	'project',
	'rulings',
	...conversationPanels.value,
	'settings',
])
const panelCounts = computed<Partial<Record<ProjectReviewSlot, number>>>(() => ({
	issues: issueCount.value,
	...(hasPreviousStructuredIssues.value
		? { 're-review': previousIssues.remainingReReviewIssues.value.length }
		: {}),
}))

function reveal(panel: ProjectReviewSlot) {
	layout.value?.reveal(panel)
}

/** The conversation panels currently shown, the first of which hosts the outcome buttons. */
const shownConversation = shallowReactive(new Set<ProjectReviewSidePanel>())
function setConversationShown(panel: ProjectReviewSidePanel, shown: boolean) {
	if (shown) shownConversation.add(panel)
	else shownConversation.delete(panel)
}

/**
 * Where the outcome buttons go: below the first conversation panel shown, else the first one
 * (out of sight), so their shortcuts keep working while the conversation is hidden.
 */
const outcomeHost = computed(
	() =>
		conversationPanels.value.find((panel) => shownConversation.has(panel)) ??
		conversationPanels.value[0],
)

onScopeDispose(
	reviewContext.registerRoute('re-review', () => {
		if (!hasPreviousStructuredIssues.value) {
			void reviewContext.revealPanel('re-review-misc')
		} else {
			reveal('re-review')
		}
	}),
)
const resetIssuesModal = ref<InstanceType<typeof NewModal>>()
const resetCancelButton = ref<InstanceType<typeof Button>>()
const issuePicker = ref<InstanceType<typeof IssuePicker>>()
const auth = useAuthState()
const keybinds = useModerationKeybinds()

function resetAllIssues() {
	resetIssuesModal.value?.hide()
	void resetIssues()
}

function revertCustomizations() {
	if (!project.value || pending.value || resetting.value) return
	messages.resetAllIssueMessages()
	resetIssuesModal.value?.hide()
}

watch(
	[projectId, hasPreviousStructuredIssues],
	([, hasIssues]) => reveal(hasIssues ? 're-review' : 'thread'),
	{ flush: 'post' },
)

watch(projectId, () => {
	resetIssuesModal.value?.hide()
})

useShortcut('cycle-conversation', () => {
	const panels = conversationPanels.value
	const shown = panels.findIndex((panel) => shownConversation.has(panel))
	reveal(panels[(shown + 1) % panels.length])
})
useShortcut(
	'new-issue',
	() => {
		void issuePicker.value?.openPicker()
	},
	() => !!project.value && !isLoading.value && !pending.value,
)
useShortcut(
	'reset',
	() => resetIssuesModal.value?.show(),
	() => !!project.value && !pending.value && !resetting.value,
)
useShortcut(
	're-review',
	() => {
		void reviewContext.revealPanel('re-review')
	},
	() =>
		hasPreviousStructuredIssues.value || !!panels.resolve({ kind: 'panel', key: 're-review-misc' }),
)

shortcuts.listen('keydown', (event) => {
	if (!isStaff(auth.value.user) || reviewShortcutBlocked(event)) return
	if (keybinds.value.handle(event, { scope: 'review-actions', ...shortcuts })) return
	if (!layoutReady.value || !projectV2.value) return
	keybinds.value.handle(event, {
		scope: 'project',
		project: projectV2.value,
		notifyCopied,
	})
})
</script>
