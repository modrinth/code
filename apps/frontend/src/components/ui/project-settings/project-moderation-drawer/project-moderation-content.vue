<template>
	<template v-if="canAccess">
		<div v-show="view === 'thread'" class="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
			<ModerationThread
				v-if="prefixedThread"
				compact
				:thread="prefixedThread"
				:project-owner-id="projectOwner?.id"
				:project="project"
				:moderation-actions="moderationActions"
				:current-member="currentMember ?? undefined"
				:auth="auth"
				class="min-h-0 flex-1"
				@update-thread="updateThread"
			>
				<template #thread-help>
					<IntlFormatted :message-id="messages.threadDescription">
						<template #help-center-link="{ children }">
							<a href="https://support.modrinth.com" target="_blank" class="text-link">
								<component :is="() => children" />
								<span class="sr-only"> {{ formatMessage(messages.opensInNewTab) }}</span>
							</a>
						</template>
					</IntlFormatted>
				</template>
			</ModerationThread>
			<div
				v-else
				class="flex items-center justify-center gap-2 rounded-b-2xl border-0 border-t border-solid border-surface-4 bg-surface-2 py-12"
			>
				<template v-if="pending">
					<SpinnerIcon class="size-5 animate-spin" /> {{ formatMessage(messages.loadingMessages) }}
				</template>
				<template v-else>
					<p class="m-0 text-red">{{ formatMessage(messages.loadingMessagesFailed) }}</p>
				</template>
			</div>
		</div>
	</template>
</template>
<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { CheckIcon, EyeOffIcon, ScaleIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import {
	type ButtonMenuAction,
	commonMessages,
	defineMessage,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	IntlFormatted,
	useVIntl,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import dayjs from 'dayjs'
import { computed, watch } from 'vue'

import ModerationThread from '~/components/ui/thread/ModerationThread.vue'
import { isApproved } from '~/helpers/projects.js'
import { verifyThreadIssuesForApproval } from '~/helpers/thread-issues'
import { injectProjectPublishing } from '~/providers/project-publishing'

withDefaults(defineProps<{ view?: 'moderation' | 'thread' }>(), { view: 'thread' })

const { formatMessage } = useVIntl()

const messages = defineMessages({
	opensInNewTab: {
		id: 'project.moderation.thread.opens-in-new-tab',
		defaultMessage: '(opens in a new tab)',
	},
	loadingMessages: {
		id: 'project.moderation.thread.loading-messages',
		defaultMessage: 'Loading messages',
	},
	loadingMessagesFailed: {
		id: 'project.moderation.thread.loading-messages-failed',
		defaultMessage: 'Failed to load messages',
	},
	approve: {
		id: 'project.moderation.thread.approve',
		defaultMessage: 'Approve',
	},
	reject: {
		id: 'project.moderation.thread.reject',
		defaultMessage: 'Reject',
	},
	withhold: {
		id: 'project.moderation.thread.withhold',
		defaultMessage: 'Withhold',
	},
	sendToReview: {
		id: 'project.moderation.thread.send-to-review',
		defaultMessage: 'Send to review',
	},
	approveWithReply: {
		id: 'conversation-thread.action.approve-with-reply',
		defaultMessage: 'Approve with reply',
	},
	rejectWithReply: {
		id: 'conversation-thread.action.reject-with-reply',
		defaultMessage: 'Reject with reply',
	},
	withholdWithReply: {
		id: 'conversation-thread.action.withhold-with-reply',
		defaultMessage: 'Withhold with reply',
	},
	projectChanged: {
		id: 'project.moderation.project-changed',
		defaultMessage: 'The selected project changed. Review it again.',
	},
	threadSectionTitle: {
		id: 'project.moderation.thread.title',
		defaultMessage: 'Moderation messages',
	},
	threadDescription: {
		id: 'project.moderation.thread.description',
		defaultMessage:
			'This is a private thread with Modrinth moderators about your project. Messages here won’t notify staff. For support, visit the <help-center-link>Help Center</help-center-link>.',
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

const {
	projectV2: project,
	setProcessing,
	currentMember,
	invalidate,
	allMembers,
	organization,
	thread,
} = injectProjectPageContext()

const { canSubmit } = injectProjectPublishing()

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
		const legacyMessage = {
			id: '69',
			author_id: null,
			body: {
				type: 'legacy_project_message' as const,
			},
			created: THREADS_RELEASE_DATE,
			hide_identity: false,
		}
		return { ...ownerThread, messages: [legacyMessage, ...ownerThread.messages] }
	}
	return ownerThread
})

const canAccess = computed(() => !!currentMember.value)

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
const { addNotification } = injectNotificationManager()
const queryClient = useQueryClient()
const pending = computed(() => thread.value === undefined)

type ModerationAction = Omit<ButtonMenuAction, 'action'> & {
	labelWithReply?: string
	action: (reviewReason?: string) => Promise<unknown>
}

const moderationActions = computed<ModerationAction[]>(() => [
	{
		id: 'approve',
		label: formatMessage(messages.approve),
		labelWithReply: formatMessage(messages.approveWithReply),
		icon: CheckIcon,
		tone: 'green',
		disabled: isApproved(project.value),
		action: (reviewReason) => {
			const requested = project.value.requested_status
			return setStatus(
				requested === 'unlisted' || requested === 'private' ? requested : 'approved',
				reviewReason,
			)
		},
	},
	{
		id: 'reject',
		label: formatMessage(messages.reject),
		labelWithReply: formatMessage(messages.rejectWithReply),
		icon: XIcon,
		tone: 'red',
		disabled: project.value.status === 'rejected',
		action: (reviewReason) => setStatus('rejected', reviewReason),
	},
	{
		id: 'withhold',
		label: formatMessage(messages.withhold),
		labelWithReply: formatMessage(messages.withholdWithReply),
		icon: EyeOffIcon,
		tone: 'orange',
		disabled: project.value.status === 'withheld',
		action: (reviewReason) => setStatus('withheld', reviewReason),
	},
	{
		id: 'send-to-review',
		label: formatMessage(messages.sendToReview),
		icon: ScaleIcon,
		tone: 'orange',
		disabled: project.value.status === 'processing' || !canSubmit.value,
		action: setProcessing,
	},
])

function updateThread(newThread: Labrinth.Threads.v3.Thread | null | undefined) {
	const threadId = newThread?.id ?? project.value?.thread_id
	if (!threadId) return

	queryClient.setQueryData<Labrinth.Threads.v3.Thread | null | undefined>(
		['thread', threadId],
		newThread,
	)
}

async function setStatus(status: Labrinth.Projects.v2.ProjectStatus, reviewReason?: string) {
	const { id: projectId, thread_id: threadId } = project.value
	startLoading()

	try {
		if (status === 'approved' || status === 'unlisted' || status === 'private') {
			await verifyThreadIssuesForApproval(threadId, client, () => {
				if (project.value.id !== projectId) throw new Error(formatMessage(messages.projectChanged))
			})
		}
		await client.labrinth.projects_v3.edit(projectId, { status, review_reason: reviewReason })
		await invalidate()
		return true
	} catch (error) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: getErrorDescription(error),
			type: 'error',
		})
		return false
	} finally {
		await queryClient.invalidateQueries({ queryKey: ['thread', threadId] })
		stopLoading()
	}
}

function getErrorDescription(error: unknown): string {
	if (typeof error === 'object' && error !== null && 'data' in error) {
		const data = (error as { data?: { description?: string } }).data
		if (data?.description) return data.description
	}

	return error instanceof Error ? error.message : String(error)
}
</script>
