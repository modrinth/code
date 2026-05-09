<template>
	<template v-if="canAccess">
		<template v-if="currentMember && (!currentMember.staffOnly || moderatorSeeUserUi)">
			<Admonition
				v-if="project.status === 'draft'"
				type="info"
				class="mb-4"
				:header="formatMessage(messages.admonitionDraftHeader)"
			>
				<p class="m-0 leading-tight">
					{{ formatMessage(messages.admonitionDraftBody) }}
				</p>
				<p class="mb-0 mt-2 leading-tight">
					<IntlFormatted :message-id="messages.admonitionDraftSubmitForReview">
						<template #rules-link="{ children }">
							<nuxt-link to="/legal/rules" class="text-link" target="_blank">
								<component :is="() => normalizeChildren(children)" />
							</nuxt-link>
						</template>
					</IntlFormatted>
				</p>
			</Admonition>
			<Admonition
				v-else-if="isApproved(project)"
				type="success"
				class="mb-4"
				:header="formatMessage(messages.admonitionApprovedHeader)"
			>
				<p class="m-0 leading-tight">
					<IntlFormatted
						:message-id="
							project.status === 'approved' || project.status === 'archived'
								? messages.admonitionApprovedPublishedFull
								: project.status === 'unlisted'
									? messages.admonitionApprovedUnlistedFull
									: project.status === 'private'
										? messages.admonitionApprovedPrivateFull
										: ''
						"
					>
						<template #visibility-settings-link="{ children }">
							<router-link :to="`${getProjectLink(project)}/settings#visibility`" class="text-link">
								<component :is="() => normalizeChildren(children)" />
							</router-link>
						</template>
					</IntlFormatted>
				</p>
			</Admonition>
			<Admonition
				v-else-if="isUnderReview(project)"
				type="moderation"
				class="mb-4"
				:header="formatMessage(messages.admonitionUnderReviewHeader)"
			>
				<p class="m-0 leading-tight">
					<IntlFormatted :message-id="messages.admonitionUnderReviewBody">
						<template #rules-link="{ children }">
							<nuxt-link to="/legal/rules" class="text-link" target="_blank">
								<component :is="() => normalizeChildren(children)" />
							</nuxt-link>
						</template>
						<template #terms-link="{ children }">
							<nuxt-link to="/legal/terms" class="text-link" target="_blank">
								<component :is="() => normalizeChildren(children)" />
							</nuxt-link>
						</template>
					</IntlFormatted>
				</p>
				<p class="mb-0 mt-2 leading-tight">
					{{ formatMessage(messages.admonitionUnderReviewFreeToEdit) }}
				</p>
				<p class="mb-0 mt-2 leading-tight">
					{{ formatMessage(messages.admonitionUnderReviewTiming) }}
				</p>
				<p v-if="showDelayMessage" class="mb-0 mt-2 font-semibold leading-tight">
					{{ formatMessage(messages.admonitionUnderReviewTimingDelay) }}
				</p>
			</Admonition>
			<Admonition
				v-else-if="project.status === 'withheld'"
				type="warning"
				class="mb-4"
				:header="formatMessage(messages.admonitionWithheldHeader)"
			>
				<p class="m-0 leading-tight">
					<IntlFormatted :message-id="messages.admonitionRejectedBody">
						<template #rules-link="{ children }">
							<nuxt-link to="/legal/rules" class="text-link" target="_blank">
								<component :is="() => normalizeChildren(children)" />
							</nuxt-link>
						</template>
						<template #terms-link="{ children }">
							<nuxt-link to="/legal/terms" class="text-link" target="_blank">
								<component :is="() => normalizeChildren(children)" />
							</nuxt-link>
						</template>
					</IntlFormatted>
				</p>
				<p class="m-0 mt-2 leading-tight">
					<IntlFormatted
						:message-id="messages.admonitionWithheldStillAccessible"
						:values="{ requestedWithheld: project.requested_status === 'unlisted' }"
					>
						<template #visibility-settings-link="{ children }">
							<nuxt-link :to="`/project/${project.id}/settings#visibility`" class="text-link">
								<component :is="() => normalizeChildren(children)" />
							</nuxt-link>
						</template>
					</IntlFormatted>
				</p>
				<p class="mb-0 mt-2 leading-tight">
					{{ formatMessage(messages.admonitionRejectedSpamNotice) }}
				</p>
			</Admonition>
			<Admonition
				v-else-if="isRejected(project)"
				type="critical"
				class="mb-4"
				:header="formatMessage(messages.admonitionRejectedHeader)"
			>
				<p class="m-0 leading-tight">
					<IntlFormatted :message-id="messages.admonitionRejectedBody">
						<template #rules-link="{ children }">
							<nuxt-link to="/legal/rules" class="text-link" target="_blank">
								<component :is="() => normalizeChildren(children)" />
							</nuxt-link>
						</template>
						<template #terms-link="{ children }">
							<nuxt-link to="/legal/terms" class="text-link" target="_blank">
								<component :is="() => normalizeChildren(children)" />
							</nuxt-link>
						</template>
					</IntlFormatted>
				</p>
				<p class="mb-0 mt-2 leading-tight">
					{{ formatMessage(messages.admonitionRejectedAddressAllConcerns) }}
				</p>
				<p class="mb-0 mt-2 leading-tight">
					{{ formatMessage(messages.admonitionRejectedSpamNotice) }}
				</p>
			</Admonition>
		</template>
		<div class="card-shadow rounded-2xl border border-solid border-surface-4 bg-surface-3">
			<div class="flex flex-col p-4">
				<div class="flex items-center justify-between">
					<h2 id="messages" class="m-0 text-xl font-semibold text-contrast">
						{{ formatMessage(messages.threadSectionTitle) }}
					</h2>
					<div v-if="currentMember?.staffOnly" class="flex items-center gap-2">
						<Toggle id="moderator-see-user-ui-toggle" v-model="moderatorSeeUserUi" small />
						<label for="moderator-see-user-ui-toggle">See what users see</label>
					</div>
				</div>
				<p v-if="!currentMember.staffOnly || moderatorSeeUserUi" class="m-0 mt-2 leading-tight">
					{{ formatMessage(messages.threadPrivateDescription) }}
				</p>
				<p v-if="!currentMember.staffOnly || moderatorSeeUserUi" class="mb-0 mt-3 leading-tight">
					<IntlFormatted :message-id="messages.threadHelpCenterNote">
						<template #help-center-link="{ children }">
							<a class="text-link" href="https://support.modrinth.com" target="_blank">
								<component :is="() => normalizeChildren(children)" />
							</a>
						</template>
					</IntlFormatted>
				</p>
				<p
					v-if="isApproved(project) && (!currentMember.staffOnly || moderatorSeeUserUi)"
					class="mb-0 mt-3 flex items-center gap-2 font-semibold text-orange"
				>
					<IssuesIcon /> {{ formatMessage(messages.threadApprovedWarning) }}
				</p>
			</div>
			<ConversationThread
				v-if="thread"
				:thread="thread"
				:project="project"
				:set-status="setStatus"
				:current-member="currentMember"
				:auth="auth"
				class="overflow-clip rounded-b-2xl border-0 border-t border-solid border-surface-4 bg-surface-2"
				@update-thread="updateThread"
			/>
		</div>
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

const showDelayMessage = ref(true)

const messages = defineMessages({
	admonitionApprovedHeader: {
		id: 'project.moderation.admonition.approved.header',
		defaultMessage: 'Project approved',
	},
	admonitionApprovedPublishedFull: {
		id: 'project.moderation.admonition.approved.published-full',
		defaultMessage:
			"Your project is published and discoverable on Modrinth. You can change the visibility of your project in your project's <visibility-settings-link>visibility settings</visibility-settings-link>.",
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
			'This is a draft project that cannot be seen by others until submitted for review and approved by our content moderation team.',
	},
	admonitionDraftSubmitForReview: {
		id: 'project.moderation.admonition.draft.submit-for-review',
		defaultMessage: `Once you have completed all required steps and ensured your project complies with <rules-link>Modrinth's Content Rules</rules-link> you can submit your project for review in the publishing checklist above.`,
	},
	admonitionUnderReviewHeader: {
		id: 'project.moderation.admonition.under-review.header',
		defaultMessage: 'Project under review',
	},
	admonitionUnderReviewBody: {
		id: 'project.moderation.admonition.under-review.body',
		defaultMessage:
			"Your project is in queue to be reviewed by our content moderation team. While you wait, please ensure your project is compliant with <rules-link>Modrinth's Content Rules</rules-link> and <terms-link>Terms of Use</terms-link>.",
	},
	admonitionUnderReviewFreeToEdit: {
		id: 'project.moderation.admonition.under-review.free-to-edit',
		defaultMessage: `You may freely modify your project as needed while under review. It won't affect your position in the queue.`,
	},
	admonitionUnderReviewTiming: {
		id: 'project.moderation.admonition.under-review.timing',
		defaultMessage: 'We aim to review projects within 24-48 hours of submission.',
	},
	admonitionUnderReviewTimingDelay: {
		id: 'project.moderation.admonition.under-review.timing-delay',
		defaultMessage: `Due to an increase in submissions, some project reviews may be delayed, this does not reflect an issue with your submission and there is no cause for alarm.
We appreciate your patience during this time while our content moderation team works hard to keep Modrinth safe.`,
	},
	admonitionRejectedHeader: {
		id: 'project.moderation.admonition.rejected.header',
		defaultMessage: 'Changes requested',
	},
	admonitionWithheldHeader: {
		id: 'project.moderation.admonition.withheld.header',
		defaultMessage: 'Unlisted by staff',
	},
	admonitionRejectedBody: {
		id: 'project.moderation.admonition.rejected.body',
		defaultMessage:
			"Our content moderation team found issues with this project that prevent it from being published on Modrinth, this may include violations of <rules-link>Modrinth's Content Rules</rules-link> or <terms-link>Terms of Use</terms-link>.",
	},
	admonitionWithheldStillAccessible: {
		id: 'project.moderation.admonition.withheld.still-accessible',
		defaultMessage:
			'Your project can still be accessed via a direct link, but will not appear publicly.{requestedWithheld, select, true { This matches your selected <visibility-settings-link>visibility settings</visibility-settings-link>, so no action is necessary.} other { Please address all moderation concerns, including any issues listed in messages below before resubmitting this project.}}',
	},
	admonitionRejectedAddressAllConcerns: {
		id: 'project.moderation.admonition.rejected.address-all-concerns',
		defaultMessage:
			'Please address all moderation concerns, including any issues listed in messages below before resubmitting this project.',
	},
	admonitionRejectedSpamNotice: {
		id: 'project.moderation.admonition.rejected.spam-notice',
		defaultMessage:
			'Spam, or repeatedly resubmitting your project without addressing all moderation concerns first, may result in account suspension.',
	},
	admonitionResubmitNote: {
		id: 'project.moderation.admonition.resubmit-note',
		defaultMessage:
			"Read the messages from the moderators below and address their comments before resubmitting. <warning-strong>Repeated submissions without addressing the moderators' comments may result in an account suspension.</warning-strong>",
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
	threadHelpCenterNote: {
		id: 'project.moderation.thread.help-center-note',
		defaultMessage:
			'This thread is only checked when you submit your project for review. For additional inquiries, please go to the <help-center-link>Modrinth Help Center</help-center-link> and click the green bubble to contact support.',
	},
	threadApprovedWarning: {
		id: 'project.moderation.thread.approved-warning',
		defaultMessage:
			'This thread is not actively monitored, but may be reviewed for information about your project as needed.',
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
