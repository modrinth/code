<template>
	<ClientOnly>
		<ProjectReviewLayout :tabs="visibleTabs">
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
									{ value: 'issues', label: formatMessage(reviewTabMessages.issues) },
									{ value: 'thread', label: formatMessage(reviewTabMessages.thread) },
								]"
							/>
							<IssuePicker v-if="activeReviewTab === 'issues'" />
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
import { defineMessages, Tabs, useVIntl } from '@modrinth/ui'
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
import { projectReviewTabs } from './layout/types'
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
const { projectId, project, projectV2, wasReviewed, permissions } = injectProjectReviewPageContext()
const visibleTabs = computed(() =>
	projectReviewTabs.filter(
		(tab) => tab !== 'permissions' || project.value?.project_types.includes('modpack'),
	),
)
const reviewProjectId = computed(() => project.value?.id)
const session = provideReviewSession(createReviewSession())
const panels = provideReviewPanels(
	createReviewPanels(
		project,
		session,
		computed(() => ({
			wasReviewed: wasReviewed.value,
			permissions: permissions.value,
		})),
	),
)
const messages = provideReviewMessages(createReviewMessages(project, projectV2, panels))
provideReviewContext(createReviewContext(reviewProjectId, (target) => !!panels.resolve(target)))
const { pending } = provideReviewSubmission(createReviewSubmission(messages, panels))
const activeReviewTab = ref('issues')
const messageThread = ref<InstanceType<typeof MessageThread>>()
const auth = useAuthState()
const keybinds = useModerationKeybinds()
const reviewTabMessages = defineMessages({
	issues: { id: 'project-review.right-panel.issues', defaultMessage: 'Issues' },
	thread: { id: 'project-review.right-panel.thread', defaultMessage: 'Thread' },
})

watch(projectId, () => {
	activeReviewTab.value = 'issues'
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
