<template>
	<template v-if="canAccess">
		<div v-if="currentMember?.staffOnly" class="mb-6">
			<Toggle v-model="moderatorSeeUserUi" />
			{{ formatMessage(messages.seeUserUiLabel) }}
		</div>
		<template v-if="currentMember && (!currentMember.staffOnly || moderatorSeeUserUi)">
			<Admonition
				v-if="project.status === 'draft'"
				type="info"
				:header="formatMessage(messages.admonitionDraftHeader)"
			>
				<p class="m-0 leading-tight">
					{{ formatMessage(messages.admonitionDraftBody) }}
				</p>
			</Admonition>
			<Admonition
				v-else-if="isApproved(project)"
				type="success"
				:header="formatMessage(messages.admonitionApprovedHeader)"
			>
				<p class="m-0 leading-tight">
					<IntlFormatted
						v-if="project.status === 'approved' || project.status === 'archived'"
						:message-id="messages.admonitionApprovedPublishedFull"
					>
						<template #settings-link="{ children }">
							<router-link :to="`${getProjectLink(project)}/settings`" class="text-link">
								<component :is="() => children" />
							</router-link>
						</template>
					</IntlFormatted>
					<IntlFormatted
						v-else-if="project.status === 'unlisted'"
						:message-id="messages.admonitionApprovedUnlistedFull"
					>
						<template #settings-link="{ children }">
							<router-link :to="`${getProjectLink(project)}/settings`" class="text-link">
								<component :is="() => children" />
							</router-link>
						</template>
					</IntlFormatted>
					<IntlFormatted
						v-else-if="project.status === 'private'"
						:message-id="messages.admonitionApprovedPrivateFull"
					>
						<template #settings-link="{ children }">
							<router-link :to="`${getProjectLink(project)}/settings`" class="text-link">
								<component :is="() => children" />
							</router-link>
						</template>
					</IntlFormatted>
				</p>
			</Admonition>
			<Admonition
				v-else-if="isUnderReview(project)"
				type="moderation"
				:header="formatMessage(messages.admonitionUnderReviewHeader)"
			>
				<p class="m-0">
					{{ formatMessage(messages.admonitionUnderReviewBody) }}
				</p>
				<p class="mb-0 mt-2 font-semibold">
					{{ formatMessage(messages.admonitionUnderReviewTiming) }}
				</p>
			</Admonition>
			<Admonition
				v-else-if="project.status === 'withheld'"
				type="warning"
				:header="formatMessage(messages.admonitionChangesRequestedHeader)"
			>
				<p class="m-0">
					<IntlFormatted :message-id="messages.admonitionWithheldBody">
						<template #rules-link="{ children }">
							<nuxt-link to="/legal/rules" class="text-link" target="_blank">
								<component :is="() => children" />
							</nuxt-link>
						</template>
					</IntlFormatted>
				</p>
				<p class="mb-0 mt-2">
					<IntlFormatted :message-id="messages.admonitionResubmitNote">
						<template #warning-strong="{ children }">
							<span class="font-semibold">
								<component :is="() => normalizeChildren(children)" />
							</span>
						</template>
					</IntlFormatted>
				</p>
			</Admonition>
			<Admonition
				v-else-if="isRejected(project)"
				type="critical"
				:header="formatMessage(messages.admonitionChangesRequestedHeader)"
			>
				<p class="m-0">
					<IntlFormatted :message-id="messages.admonitionRejectedBody">
						<template #rules-link="{ children }">
							<nuxt-link to="/legal/rules" class="text-link" target="_blank">
								<component :is="() => children" />
							</nuxt-link>
						</template>
					</IntlFormatted>
				</p>
				<p class="mb-0 mt-2">
					<IntlFormatted :message-id="messages.admonitionResubmitNote">
						<template #warning-strong="{ children }">
							<span class="font-semibold">
								<component :is="() => normalizeChildren(children)" />
							</span>
						</template>
					</IntlFormatted>
				</p>
			</Admonition>
			<div class="mb-4 flex flex-col">
				<h2 id="messages" class="mb-2 mt-4 text-xl font-semibold text-contrast">
					{{ formatMessage(messages.threadSectionTitle) }}
				</h2>
				<p class="m-0 leading-tight">
					{{ formatMessage(messages.threadPrivateDescription) }}
				</p>
				<p class="mb-0 mt-3 leading-tight">
					<IntlFormatted :message-id="messages.threadHelpCenterNote">
						<template #help-center-link="{ children }">
							<a class="text-link" href="https://support.modrinth.com" target="_blank">
								<component :is="() => children" />
							</a>
						</template>
					</IntlFormatted>
				</p>
				<p
					v-if="isApproved(project)"
					class="mb-0 mt-3 flex items-center gap-2 font-semibold text-orange"
				>
					<IssuesIcon /> {{ formatMessage(messages.threadApprovedWarning) }}
				</p>
			</div>
		</template>
		<ConversationThread
			v-if="thread"
			:thread="thread"
			:project="project"
			:set-status="setStatus"
			:current-member="currentMember"
			:auth="auth"
			@update-thread="updateThread"
		/>
	</template>
</template>
<script setup>
import { IssuesIcon } from '@modrinth/assets'
import {
	Admonition,
	commonMessages,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	IntlFormatted,
	normalizeChildren,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, watch } from 'vue'

import ConversationThread from '~/components/ui/thread/ConversationThread.vue'
import { getProjectLink, isApproved, isRejected, isUnderReview } from '~/helpers/projects.js'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	seeUserUiLabel: {
		id: 'project.moderation.see-user-ui',
		defaultMessage: 'See what users see',
	},
	admonitionApprovedHeader: {
		id: 'project.moderation.admonition.approved.header',
		defaultMessage: 'Project approved',
	},
	admonitionApprovedPublishedFull: {
		id: 'project.moderation.admonition.approved.published-full',
		defaultMessage:
			"Your project is published and discoverable on Modrinth. You can change the visibility of your project in <settings-link>your project's settings</settings-link>.",
	},
	admonitionApprovedUnlistedFull: {
		id: 'project.moderation.admonition.approved.unlisted-full',
		defaultMessage:
			"Your project is unlisted, meaning it can only be accessed with a direct link and is not discoverable on Modrinth. You can change the visibility of your project in <settings-link>your project's settings</settings-link>.",
	},
	admonitionApprovedPrivateFull: {
		id: 'project.moderation.admonition.approved.private-full',
		defaultMessage:
			"Your project is private, meaning it can only be accessed by you and people you invite. You can change the visibility of your project in <settings-link>your project's settings</settings-link>.",
	},
	admonitionDraftHeader: {
		id: 'project.moderation.admonition.draft.header',
		defaultMessage: 'Draft project',
	},
	admonitionDraftBody: {
		id: 'project.moderation.admonition.draft.body',
		defaultMessage:
			'This is a draft project that needs to be submitted for review before it can be accessed by others. To submit, check out the publishing checklist at the top.',
	},
	admonitionUnderReviewHeader: {
		id: 'project.moderation.admonition.under-review.header',
		defaultMessage: 'Project under review',
	},
	admonitionUnderReviewBody: {
		id: 'project.moderation.admonition.under-review.body',
		defaultMessage:
			"Your project won't be available for people to use until it's manually reviewed by our moderation team. Please be patient and ensure your project follows all our rules!",
	},
	admonitionUnderReviewTiming: {
		id: 'project.moderation.admonition.under-review.timing',
		defaultMessage:
			'Our goal is always to review projects in 24–48 hours, however due to an increase in submissions, do not be alarmed if your project takes longer to be reviewed.',
	},
	admonitionChangesRequestedHeader: {
		id: 'project.moderation.admonition.changes-requested.header',
		defaultMessage: 'Changes requested',
	},
	admonitionWithheldBody: {
		id: 'project.moderation.admonition.withheld.body',
		defaultMessage:
			"Your project does not currently meet Modrinth's <rules-link>content rules</rules-link> in order to be publicly listed on Modrinth. The moderators have requested you make changes before it can be fully approved, but it can currently be accessed with a direct link.",
	},
	admonitionRejectedBody: {
		id: 'project.moderation.admonition.rejected.body',
		defaultMessage:
			"Your project does not currently meet Modrinth's <rules-link>content rules</rules-link> and the moderators have requested you make changes before it can be approved.",
	},
	admonitionResubmitNote: {
		id: 'project.moderation.admonition.resubmit-note',
		defaultMessage:
			"Read the messages from the moderators below and address their comments before resubmitting. <warning-strong>Repeated submissions without addressing the moderators' comments may result in an account suspension.</warning-strong>",
	},
	threadSectionTitle: {
		id: 'project.moderation.thread.title',
		defaultMessage: 'Message the moderators',
	},
	threadPrivateDescription: {
		id: 'project.moderation.thread.private-description',
		defaultMessage:
			'This is a private conversation thread with the Modrinth moderators. They may message you with issues concerning this project.',
	},
	threadHelpCenterNote: {
		id: 'project.moderation.thread.help-center-note',
		defaultMessage:
			'This thread is only checked when you submit your project for review. For additional inquiries, please go to the <help-center-link>Modrinth Help Center</help-center-link> and click the green bubble to contact support.',
	},
	threadApprovedWarning: {
		id: 'project.moderation.thread.approved-warning',
		defaultMessage:
			'Moderators are not actively monitoring this chat because your project has already been approved.',
	},
	unauthorizedStatus: {
		id: 'project.moderation.error.unauthorized',
		defaultMessage: 'Unauthorized',
	},
})

const { addNotification } = injectNotificationManager()
const { projectV2: project, currentMember, invalidate, allMembers } = injectProjectPageContext()

const canAccess = computed(() => !!currentMember.value)

const moderatorSeeUserUiCookie = useCookie('moderation-see-user-ui', {
	default: () => false,
	maxAge: 60 * 60 * 24 * 365,
	sameSite: 'lax',
	path: '/',
})

const moderatorSeeUserUi = computed({
	get() {
		return moderatorSeeUserUiCookie.value
	},
	set(value) {
		moderatorSeeUserUiCookie.value = value
	},
})

watch(
	[currentMember, allMembers],
	() => {
		if (allMembers.value.length > 0 && !canAccess.value) {
			showError({
				fatal: true,
				statusCode: 401,
				statusMessage: formatMessage(messages.unauthorizedStatus),
			})
		}
	},
	{ flush: 'sync', immediate: true },
)

const auth = await useAuth()
const client = injectModrinthClient()
const queryClient = useQueryClient()

const { data: thread } = useQuery({
	queryKey: computed(() => ['thread', project.value?.thread_id]),
	queryFn: () => client.labrinth.threads_v3.getThread(project.value.thread_id),
	enabled: computed(() => !!project.value?.thread_id),
})

function updateThread(newThread) {
	const threadId = newThread?.id ?? project.value?.thread_id
	if (!threadId) return

	queryClient.setQueryData(['thread', threadId], newThread)
}

async function setStatus(status) {
	startLoading()

	try {
		await client.labrinth.projects_v2.edit(project.value.id, { status })

		project.value.status = status
		await invalidate()
		await queryClient.invalidateQueries({ queryKey: ['thread', project.value?.thread_id] })
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}

	stopLoading()
}
</script>
