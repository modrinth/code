<template>
	<div class="flex h-full min-h-0 min-w-0 flex-col gap-2.5 overflow-hidden">
		<ProjectActions />
		<div
			v-if="project"
			ref="scrollContainer"
			class="flex min-h-0 min-w-0 flex-1 flex-col overflow-y-auto overscroll-contain"
			@scroll="updateScrollPosition"
		>
			<div ref="conversationContent" class="mt-auto shrink-0">
				<ConversationThread
					v-if="thread"
					v-model:reply-body="draft"
					:thread="thread"
					:generating-message="generating"
					:project="project"
					:auth="auth"
					:set-status="setStatus"
					:before-send-reply="beforeSendReply"
					initial-preview
					class="rounded-none border-none bg-transparent p-0 text-xs"
					@update-thread="updateThread"
				/>
				<div v-else-if="isError" class="flex flex-col gap-3 p-4">
					<p class="m-0 text-red" role="alert">
						{{ formatMessage(messages.loadError) }}
					</p>
					<Button class="w-fit" @click="() => refetch()">
						{{ formatMessage(messages.retry) }}
					</Button>
				</div>
				<p v-else class="m-0 p-4 text-secondary" role="status">
					{{ formatMessage(messages.loading) }}
				</p>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	Button,
	commonMessages,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { useResizeObserver } from '@vueuse/core'
import { ref, watch } from 'vue'

import ConversationThread from '~/components/ui/thread/ConversationThread.vue'
import { injectProjectReviewPageContext } from '~/providers/project-review'
import { injectReviewMessages } from '~/providers/project-review/review-messages'
import { injectReviewPanels } from '~/providers/project-review/review-panels'

import { projectReviewMessages as messages } from './messages'
import ProjectActions from './project-actions.vue'

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const auth = useAuthState()
const { draft, generating } = injectReviewMessages()
const panels = injectReviewPanels()
const { project, threadQuery, disclosures } = injectProjectReviewPageContext()
const { data: thread, isError, refetch } = threadQuery

const scrollContainer = ref<HTMLElement>()
const conversationContent = ref<HTMLElement>()
const isAtBottom = ref(true)

function updateScrollPosition() {
	const container = scrollContainer.value
	if (!container) return
	isAtBottom.value = container.scrollHeight - container.scrollTop - container.clientHeight <= 1
}

function scrollToBottom() {
	const container = scrollContainer.value
	if (container) container.scrollTop = container.scrollHeight
}

useResizeObserver([scrollContainer, conversationContent], () => {
	if (isAtBottom.value) scrollToBottom()
})

watch(
	[scrollContainer, () => thread.value?.id],
	() => {
		isAtBottom.value = true
		scrollToBottom()
	},
	{ flush: 'post' },
)

function updateThread(updatedThread: Labrinth.Threads.v3.Thread | null | undefined) {
	if (!updatedThread) return
	queryClient.setQueryData(['thread', updatedThread.id], updatedThread)
}

const correctionMessages = defineMessages({
	unsavedDisclosures: {
		id: 'project-review.disclosures.unsaved',
		defaultMessage:
			'Save or reset your disclosure changes before sending a reply or changing the project status.',
	},
	missing: {
		id: 'project-review.corrections.missing-fields',
		defaultMessage: 'Complete the required review fields before sending your reply.',
	},
	conflict: {
		id: 'project-review.corrections.conflicts',
		defaultMessage: 'Resolve conflicting corrections before sending your reply.',
	},
	projectChanged: {
		id: 'project-review.corrections.project-changed',
		defaultMessage: 'The selected project changed. Review the corrections again.',
	},
})

const correctionMutation = useMutation({
	mutationFn: async ({ id, plan }: { id: string; plan: typeof panels.corrections.value }) => {
		if (Object.keys(plan.project).length) await client.labrinth.projects_v3.edit(id, plan.project)
		for (const [versionId, patch] of Object.entries(plan.versions)) {
			if (Object.keys(patch).length)
				await client.labrinth.versions_v3.modifyVersion(versionId, patch)
		}
	},
	onSettled: async (_, __, { id, plan }) => {
		await Promise.all([
			queryClient.invalidateQueries({ queryKey: ['project', 'v3', id] }),
			queryClient.invalidateQueries({ queryKey: ['project', 'v2', id] }),
			queryClient.invalidateQueries({ queryKey: ['project', id] }),
			...Object.keys(plan.versions).map((id) =>
				queryClient.invalidateQueries({ queryKey: ['version', id] }),
			),
		])
	},
})

async function beforeSendReply({ privateMessage }: { privateMessage: boolean }) {
	if (privateMessage) return
	if (disclosures.hasChanges.value || disclosures.saving.value)
		throw new Error(formatMessage(correctionMessages.unsavedDisclosures))
	if (panels.validationErrors.value.length)
		throw new Error(formatMessage(correctionMessages.missing))
	if (!panels.correctionsRequested.value) return
	const current = project.value
	if (!current) throw new Error(formatMessage(correctionMessages.projectChanged))
	const plan = panels.corrections.value
	if (plan.conflicts.length) throw new Error(formatMessage(correctionMessages.conflict))
	if (Object.keys(plan.versions).some((id) => !current.versions.includes(id)))
		throw new Error(formatMessage(correctionMessages.projectChanged))
	if (!plan.issueIds.length) return
	const reply = draft.value
	try {
		await correctionMutation.mutateAsync({ id: current.id, plan })
	} finally {
		if (project.value?.id === current.id) draft.value = reply
	}
	if (project.value?.id !== current.id)
		throw new Error(formatMessage(correctionMessages.projectChanged))
}

const statusMutation = useMutation({
	mutationFn: ({
		id,
		status,
	}: {
		id: string
		threadId: string
		status: Labrinth.Projects.v2.ProjectStatus
	}) => client.labrinth.projects_v3.edit(id, { status }),
	onSuccess: async (_, { id, threadId }) => {
		await Promise.all([
			queryClient.invalidateQueries({ queryKey: ['project', 'v3', id] }),
			queryClient.invalidateQueries({ queryKey: ['project', 'v2', id] }),
			queryClient.invalidateQueries({ queryKey: ['project', id] }),
			queryClient.invalidateQueries({ queryKey: ['thread', threadId] }),
		])
	},
})

async function setStatus(status: Labrinth.Projects.v2.ProjectStatus) {
	if (!project.value) return
	try {
		if (disclosures.hasChanges.value || disclosures.saving.value)
			throw new Error(formatMessage(correctionMessages.unsavedDisclosures))
		await statusMutation.mutateAsync({
			id: project.value.id,
			threadId: project.value.thread_id,
			status,
		})
	} catch (error) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: error instanceof Error ? error.message : String(error),
			type: 'error',
		})
	}
}
</script>
