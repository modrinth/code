<template>
	<template v-if="canAccess">
		<Admonition
			v-if="userFacingUiVisible && moderationAdmonition"
			:type="moderationAdmonition.type"
			class="mb-4"
			:header="formatMessage(moderationAdmonition.header)"
		>
			<template
				v-for="(message, index) in moderationAdmonition.body"
				:key="`moderation-admonition.${project.status}+${project.requested_status ?? 'none'}.body.${index}`"
			>
				<p v-if="message" class="preserve-lines mb-0 mt-2 leading-tight first:mt-0">
					<IntlFormatted
						:message-id="message"
						:values="{
							requestedStatus: project.requested_status ?? 'none',
						}"
					>
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
						<template #visibility-settings-link="{ children }">
							<router-link :to="`${getProjectLink(project)}/settings#visibility`" class="text-link">
								<component :is="() => normalizeChildren(children)" />
							</router-link>
						</template>
						<template #emphasis="{ children }">
							<span class="font-semibold">
								<component :is="() => normalizeChildren(children)" />
							</span>
						</template>
					</IntlFormatted>
				</p>
			</template>
		</Admonition>
		<div class="card-shadow mb-6 rounded-2xl border border-solid border-surface-4 bg-surface-3">
			<div class="flex flex-col p-4">
				<div class="flex items-center justify-between">
					<h2 id="messages" class="m-0 text-xl font-semibold text-contrast">
						{{ formatMessage(messages.threadSectionTitle) }}
					</h2>
					<div v-if="currentMember?.staffOnly" class="flex items-center gap-2">
						<Toggle id="moderator-see-user-ui-toggle" v-model="moderatorSeeUserUi" small />
						<label for="moderator-see-user-ui-toggle">
							{{ formatMessage(messages.moderatorSeeUserUiToggle) }}
						</label>
					</div>
				</div>
				<template v-if="userFacingUiVisible">
					<p class="m-0 mt-2 leading-tight">
						{{ formatMessage(messages.threadPrivateDescription) }}
					</p>
					<p class="mb-0 mt-3 leading-tight">
						<IntlFormatted :message-id="messages.threadHelpCenterNote1">
							<template #help-center-link="{ children }">
								<a class="text-link" href="https://support.modrinth.com" target="_blank">
									<component :is="() => normalizeChildren(children)" />
								</a>
							</template>
						</IntlFormatted>
					</p>
					<p class="mb-0 mt-2 leading-tight">
						<IntlFormatted :message-id="messages.threadHelpCenterNote2">
							<template #help-center-link="{ children }">
								<a class="text-link" href="https://support.modrinth.com" target="_blank">
									<component :is="() => normalizeChildren(children)" />
								</a>
							</template>
						</IntlFormatted>
					</p>
					<p
						v-if="isApproved(project)"
						class="mb-0 mt-3 flex items-center gap-2 font-semibold text-orange"
					>
						<IssuesIcon />
						{{ formatMessage(messages.threadApprovedWarning) }}
					</p>
				</template>
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
<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { IssuesIcon } from '@modrinth/assets'
import {
	Admonition,
	commonMessages,
	defineMessage,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	IntlFormatted,
	type MessageDescriptor,
	normalizeChildren,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type Ref, ref, watch } from 'vue'

import ConversationThread from '~/components/ui/thread/ConversationThread.vue'
import { getProjectLink, isApproved, isRejected, isUnderReview } from '~/helpers/projects.js'

const { formatMessage } = useVIntl()

const showDelayMessage = ref(true)

type ProjectPageMember = Labrinth.Projects.v3.TeamMember & { staffOnly?: boolean }

const messages = defineMessages({
	admonitionRejectedBody: {
		id: 'project.moderation.admonition.rejected.body',
		defaultMessage:
			"Modrinth's moderation team found issues with this project that prevent it from being published on Modrinth, this may include violations of <rules-link>Modrinth's Content Rules</rules-link> or <terms-link>Terms of Use</terms-link>.",
	},
	admonitionRejectedSpamNotice: {
		id: 'project.moderation.admonition.rejected.spam-notice',
		defaultMessage:
			'Repeatedly submitting your project without addressing all moderation concerns first may result in account suspension.',
	},
	threadSectionTitle: {
		id: 'project.moderation.thread.title',
		defaultMessage: 'Moderation messages',
	},
	moderatorSeeUserUiToggle: {
		id: 'project.moderation.thread.moderator-see-user-ui-toggle',
		defaultMessage: 'See what users see',
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
	approvedProjectVisibilityMessage: {
		id: 'project.moderation.admonition.approved.body.visibility-message',
		defaultMessage:
			"You can change the visibility of your project in your project's <visibility-settings-link>visibility settings</visibility-settings-link>.",
	},
})

const { addNotification } = injectNotificationManager()
const {
	projectV2: project,
	currentMember: currentMemberRaw,
	invalidate,
	allMembers,
} = injectProjectPageContext()
const currentMember = currentMemberRaw as Ref<ProjectPageMember | null>

const canAccess = computed(() => !!currentMember.value)
const userFacingUiVisible = computed(
	() => !!currentMember.value && (!currentMember.value.staffOnly || moderatorSeeUserUi.value),
)

const approvedAdmonitionMessage = computed<MessageDescriptor | null>(() => {
	switch (project.value?.status) {
		case 'approved':
		case 'archived':
			return defineMessage({
				id: 'project.moderation.admonition.approved.body.public',
				defaultMessage: 'Your project is published and discoverable on Modrinth.',
			})
		case 'unlisted':
			return defineMessage({
				id: 'project.moderation.admonition.approved.body.unlisted',
				defaultMessage:
					'Your project is unlisted, meaning it can only be accessed with a direct link and is not discoverable on Modrinth.',
			})

		case 'private':
			return defineMessage({
				id: 'project.moderation.admonition.approved.body.private',
				defaultMessage:
					'Your project is private, meaning it can only be accessed by you and people you invite',
			})
		default:
			return null
	}
})

const moderationAdmonition = computed<{
	type: InstanceType<typeof Admonition>['type']
	header: MessageDescriptor
	body: (MessageDescriptor | null)[]
} | null>(() => {
	const currentProject = project.value

	if (currentProject.status === 'draft') {
		return {
			type: 'info',
			header: defineMessage({
				id: 'project.moderation.admonition.draft.header',
				defaultMessage: 'Draft project',
			}),
			body: [
				defineMessage({
					id: 'project.moderation.admonition.draft.body',
					defaultMessage:
						"This is a draft project that cannot be seen by others until submitted for review and approved by Modrinth's moderation team.",
				}),
				defineMessage({
					id: 'project.moderation.admonition.draft.submit-for-review',
					defaultMessage: `Once you have completed all required steps and ensured your project complies with <rules-link>Modrinth's Content Rules</rules-link> you can submit your project for review.`,
				}),
			],
		}
	}

	if (isApproved(currentProject) && approvedAdmonitionMessage.value) {
		return {
			type: 'success',
			header: defineMessage({
				id: 'project.moderation.admonition.approved.header',
				defaultMessage: 'Project approved',
			}),
			body: [approvedAdmonitionMessage.value, messages.approvedProjectVisibilityMessage],
		}
	}

	if (isUnderReview(currentProject)) {
		return {
			type: 'moderation',
			header: defineMessage({
				id: 'project.moderation.admonition.under-review.header',
				defaultMessage: 'Project under review',
			}),
			body: [
				defineMessage({
					id: 'project.moderation.admonition.under-review.body.1',
					defaultMessage: `Your project is in queue to be reviewed by Modrinth's moderation team.`,
				}),
				defineMessage({
					id: 'project.moderation.admonition.under-review.body.2',
					defaultMessage: `While you wait, please ensure your project is compliant with <rules-link>Modrinth's Content Rules</rules-link> and <terms-link>Terms of Use</terms-link>.`,
				}),
				defineMessage({
					id: 'project.moderation.admonition.under-review.body.3',
					defaultMessage: `You may freely modify or update your project while under review. It won't affect your position in the queue.`,
				}),
				defineMessage({
					id: 'project.moderation.admonition.under-review.body.4',
					defaultMessage: `We aim to review projects within 24-48 hours of submission.`,
				}),
				showDelayMessage.value
					? defineMessage({
							id: 'project.moderation.admonition.under-review.body.delay.1',
							defaultMessage: `<emphasis>Due to delays, some reviews may take longer than expected, this does not reflect an issue with your submission and is not a cause for alarm.</emphasis>`,
						})
					: null,
				showDelayMessage.value
					? defineMessage({
							id: 'project.moderation.admonition.under-review.body.delay.2',
							defaultMessage: `<emphasis>We appreciate your patience while our content moderators work hard to keep Modrinth safe.</emphasis>`,
						})
					: null,
			],
		}
	}

	if (currentProject.status === 'withheld') {
		return {
			type: 'warning',
			header: defineMessage({
				id: 'project.moderation.admonition.withheld.header',
				defaultMessage: 'Unlisted by staff',
			}),
			body: [
				messages.admonitionRejectedBody,
				defineMessage({
					id: 'project.moderation.admonition.withheld.still-accessible',
					defaultMessage:
						'Your project can still be accessed via a direct link, but will not appear publicly.{requestedStatus, select, unlisted { Based on your selected <visibility-settings-link>visibility settings</visibility-settings-link>, most likely no action is necessary.} other { Please address all moderation concerns, including any issues listed in messages below before resubmitting this project.}}',
				}),
				messages.admonitionRejectedSpamNotice,
			],
		}
	}

	if (isRejected(currentProject)) {
		return {
			type: 'critical',
			header: defineMessage({
				id: 'project.moderation.admonition.rejected.header',
				defaultMessage: 'Changes requested',
			}),
			body: [
				messages.admonitionRejectedBody,
				defineMessage({
					id: 'project.moderation.admonition.rejected.address-all-concerns',
					defaultMessage:
						'Please address all moderation concerns, including any issues listed in messages below before resubmitting this project.',
				}),
				messages.admonitionRejectedSpamNotice,
			],
		}
	}

	return null
})

const moderatorSeeUserUiCookie = useCookie('moderation-see-user-ui', {
	default: () => false,
	maxAge: 60 * 60 * 24 * 365,
	sameSite: 'lax',
	path: '/',
})

const moderatorSeeUserUi = computed<boolean>({
	get() {
		return moderatorSeeUserUiCookie.value ?? false
	},
	set(value: boolean) {
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

const { data: thread } = useQuery({
	queryKey: computed(() => ['thread', project.value?.thread_id]),
	queryFn: () => client.labrinth.threads_v3.getThread(project.value.thread_id),
	enabled: computed(() => !!project.value?.thread_id),
})

function updateThread(newThread: Labrinth.Threads.v3.Thread | null | undefined) {
	const threadId = newThread?.id ?? project.value?.thread_id
	if (!threadId) return

	queryClient.setQueryData<Labrinth.Threads.v3.Thread | null | undefined>(
		['thread', threadId],
		newThread,
	)
}

async function setStatus(status: Labrinth.Projects.v2.ProjectStatus) {
	startLoading()

	try {
		await client.labrinth.projects_v2.edit(project.value.id, { status })

		project.value.status = status
		await invalidate()
		await queryClient.invalidateQueries({ queryKey: ['thread', project.value?.thread_id] })
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: getErrorDescription(err),
			type: 'error',
		})
	}

	stopLoading()
}

function getErrorDescription(err: unknown): string {
	if (typeof err === 'object' && err !== null && 'data' in err) {
		const data = (err as { data?: { description?: string } }).data
		if (data?.description) return data.description
	}

	return err instanceof Error ? err.message : String(err)
}
</script>
