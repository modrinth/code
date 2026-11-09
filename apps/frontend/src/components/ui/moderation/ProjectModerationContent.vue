<template>
	<template v-if="canAccess">
		<div v-show="view === 'thread'" class="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
			<div class="shrink-0 border-0 border-b border-solid border-divider px-4 py-3">
				<p class="m-0 text-xs leading-relaxed text-secondary">
					{{ formatMessage(messages.threadPrivateDescription) }}
				</p>
				<p class="m-0 mt-2 text-xs leading-relaxed text-secondary">
					{{ formatMessage(messages.threadHelpCenterNote1) }}
					<a href="https://support.modrinth.com" target="_blank" class="text-link"
						>Help Center <span class="sr-only">(opens in a new tab)</span></a
					>
				</p>
			</div>
			<ConversationThread
				v-if="prefixedThread"
				compact
				:thread="prefixedThread"
				:project-owner-id="projectOwner?.id"
				:project="project"
				:set-status="setStatus"
				:review-submission-disabled="reviewSubmissionDisabled"
				:current-member="currentMember ?? undefined"
				:auth="auth"
				class="min-h-0 flex-1"
				@update-thread="updateThread"
			/>
			<div
				v-else
				class="flex items-center justify-center gap-2 rounded-b-2xl border-0 border-t border-solid border-surface-4 bg-surface-2 py-12"
			>
				<template v-if="pending">
					<SpinnerIcon class="size-5 animate-spin" /> Loading messages
				</template>
				<template v-else>
					<p class="m-0 text-red">Failed to load messages</p>
				</template>
			</div>
		</div>
	</template>
</template>
<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { SpinnerIcon } from '@modrinth/assets'
import {
	commonMessages,
	defineMessage,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	useVIntl,
} from '@modrinth/ui'
import { isStaff } from '@modrinth/utils'
import { useQueryClient } from '@tanstack/vue-query'
import dayjs from 'dayjs'
import { computed, watch } from 'vue'

import ConversationThread from '~/components/ui/thread/ConversationThread.vue'
import {
	canResubmitProjectForReview,
	canSubmitProjectForReview,
} from '~/composables/link-network-validation/submission'
import { isRejected } from '~/helpers/projects.js'
import { verifyThreadIssuesForApproval } from '~/helpers/thread-issues'

withDefaults(defineProps<{ view?: 'moderation' | 'thread' }>(), { view: 'thread' })

const { formatMessage } = useVIntl()

const messages = defineMessages({
	projectChanged: {
		id: 'project.moderation.project-changed',
		defaultMessage: 'The selected project changed. Review it again.',
	},

	threadSectionTitle: {
		id: 'project.moderation.thread.title',
		defaultMessage: 'Moderation messages',
	},
	threadPrivateDescription: {
		id: 'project.moderation.thread.private-description',
		defaultMessage:
			'This is a private conversation thread with the Modrinth moderators. They may message you with issues concerning this project.',
	},
	threadHelpCenterNote1: {
		id: 'project.moderation.thread.help-center-note.1',
		defaultMessage:
			'Content moderators cannot provide support for most issues and messages to this thread do not notify staff.',
	},
	threadHelpCenterNote2: {
		id: 'project.moderation.thread.help-center-note.2',
		defaultMessage:
			'If you need assistance or have additional inquiries, please visit the <help-center-link>Modrinth Help Center</help-center-link> and click the blue bubble to contact support.',
	},
	threadApprovedWarning: {
		id: 'project.moderation.thread.approved-warning',
		defaultMessage:
			'This thread is not actively monitored, but may be reviewed for information about your project if needed.',
	},
})

const { addNotification } = injectNotificationManager()
const {
	projectV2: project,
	projectValidation,
	projectValidationLoading,
	projectLinksNetworkValidationLoading,
	setProcessing,
	currentMember,
	invalidate,
	allMembers,
	organization,
	thread,
} = injectProjectPageContext()

const reviewSubmissionDisabled = computed(
	() =>
		!canSubmitProjectForReview(
			projectValidation.value,
			projectValidationLoading.value || projectLinksNetworkValidationLoading.value,
		) ||
		(isRejected(project.value) && !canResubmitProjectForReview(thread.value)),
)

const THREADS_RELEASE_DATE = '2023-08-05T12:00:00-07:00'

const projectOwner = computed(
	() =>
		allMembers.value.find((member) => member.is_owner)?.user ??
		organization.value?.members.find((member) => member.is_owner)?.user,
)

const prefixedThread = computed(() => {
	if (!thread.value) return thread.value
	const owner = projectOwner.value
	const ownerThread = {
		...thread.value,
		members: owner
			? [...thread.value.members.filter((member) => member.id !== owner.id), owner]
			: thread.value.members,
	}
	const projectDate = project.value?.queued ?? project.value?.approved ?? project.value?.published
	if (projectDate && dayjs(projectDate).isBefore(dayjs(THREADS_RELEASE_DATE))) {
		const newThread = { ...ownerThread, messages: [...ownerThread.messages] }
		newThread.messages.unshift({
			id: '69',
			author_id: null,
			body: {
				type: 'legacy_project_message',
			},
			created: THREADS_RELEASE_DATE,
			hide_identity: false,
		})
		return newThread
	}
	return ownerThread
})

const canAccess = computed(() => !!currentMember.value)
const staff = computed(() => isStaff(currentMember.value?.user))

watch(
	[currentMember, allMembers],
	() => {
		if (allMembers.value.length > 0 && !canAccess.value) {
			showError({
				fatal: true,
				statusCode: 401,
				statusMessage: formatMessage(
					defineMessage({
						id: 'project.moderation.error.unauthorized',
						defaultMessage: 'Unauthorized',
					}),
				),
			})
		}
	},
	{ flush: 'sync', immediate: true },
)

const auth = await useAuth()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const pending = computed(() => thread.value === undefined)

function updateThread(newThread: Labrinth.Threads.v3.Thread | null | undefined) {
	const threadId = newThread?.id ?? project.value?.thread_id
	if (!threadId) return

	queryClient.setQueryData<Labrinth.Threads.v3.Thread | null | undefined>(
		['thread', threadId],
		newThread,
	)
}

async function setStatus(status: Labrinth.Projects.v2.ProjectStatus) {
	if (status === 'processing') {
		return await setProcessing()
	}
	startLoading()
	const projectId = project.value.id
	const threadId = project.value.thread_id

	try {
		if (['approved', 'unlisted', 'private'].includes(status))
			await verifyThreadIssuesForApproval(threadId, client, () => {
				if (project.value.id !== projectId) throw new Error(formatMessage(messages.projectChanged))
			})
		await client.labrinth.projects_v3.edit(projectId, { status })
		await invalidate()
		await queryClient.invalidateQueries({ queryKey: ['thread', threadId] })
		return true
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: getErrorDescription(err),
			type: 'error',
		})
		return false
	} finally {
		await queryClient.invalidateQueries({ queryKey: ['thread', threadId] })
		stopLoading()
	}
}

function getErrorDescription(err: unknown): string {
	if (typeof err === 'object' && err !== null && 'data' in err) {
		const data = (err as { data?: { description?: string } }).data
		if (data?.description) return data.description
	}

	return err instanceof Error ? err.message : String(err)
}
</script>
