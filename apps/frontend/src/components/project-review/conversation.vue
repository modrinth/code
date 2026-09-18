<template>
	<div class="flex h-full min-h-0 min-w-0 flex-col gap-2.5 overflow-hidden">
		<ProjectActions class="border-0 border-b border-solid border-divider" />
		<div v-if="project" class="min-h-0 min-w-0 flex-1">
			<ConversationThread
				v-if="thread"
				:thread="thread"
				:project="project"
				:auth="auth"
				:set-status="setStatus"
				scroll-messages
				class="rounded-none border-none bg-transparent p-0"
				@update-thread="updateThread"
			/>
			<div v-else-if="isError" class="flex flex-col gap-3 p-4">
				<p class="m-0 text-red" role="alert">{{ formatMessage(messages.loadError) }}</p>
				<Button class="w-fit" @click="() => refetch()">
					{{ formatMessage(messages.retry) }}
				</Button>
			</div>
			<p v-else class="m-0 p-4 text-secondary" role="status">
				{{ formatMessage(messages.loading) }}
			</p>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	Button,
	commonMessages,
	injectModrinthClient,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQueryClient } from '@tanstack/vue-query'

import ConversationThread from '~/components/ui/thread/ConversationThread.vue'
import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages as messages } from './messages'
import ProjectActions from './project-actions.vue'

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const auth = useAuthState()
const { project, threadQuery } = injectProjectReviewPageContext()
const { data: thread, isError, refetch } = threadQuery

function updateThread(updatedThread: Labrinth.Threads.v3.Thread | null | undefined) {
	if (!updatedThread) return
	queryClient.setQueryData(['thread', updatedThread.id], updatedThread)
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
