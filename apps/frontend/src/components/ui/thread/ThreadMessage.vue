<template>
	<div
		class="message group/thread relative grid min-w-0 grid-cols-[2rem_minmax(0,1fr)] items-start gap-x-2 break-words px-4 py-3 before:pointer-events-none before:absolute before:inset-0 before:opacity-5 before:content-[''] [&>.message__icon:focus-visible+.message__content_.message__author_a]:underline [&>.message__icon:hover+.message__content_.message__author_a]:underline"
		:class="[
			noLinks ? '!p-0' : 'focus-within:bg-surface-2.5 hover:bg-surface-2.5',
			isPrivateMessage ? 'text-[var(--color-icon)]' : '',
			message.body.type === 'legacy_project_message'
				? 'mb-2 !py-6 before:bg-blue'
				: isPrivateMessage && settings.get(moderationSettings.General.PrivateMessageHighlight)
					? 'before:bg-orange'
					: '',
		]"
	>
		<AutoLink
			v-if="author"
			class="message__icon flex min-w-0 max-w-full items-center no-underline"
			:to="noLinks ? '' : `/user/${author.username}`"
			tabindex="-1"
			aria-hidden="true"
		>
			<Avatar size="2rem" :src="author.avatar_url" circle :raised="raised" />
		</AutoLink>
		<div
			v-else
			class="message__icon inline-flex size-8 shrink-0 items-center justify-center rounded-full text-orange [&>svg]:h-3/5 [&>svg]:w-3/5"
			:class="raised ? 'bg-surface-3' : 'bg-surface-2'"
		>
			<InfoIcon v-if="message.body.type === 'legacy_project_message'" class="text-blue" />
			<Avatar v-else-if="isAutoApproval" size="2rem" circle :raised="raised" />
			<ThreadRoleBadge v-else role="moderator" avatar />
		</div>
		<div class="message__content min-w-0">
			<template v-if="author">
				<span
					class="message__author min-w-0 max-w-full font-bold"
					:class="[
						authorClasses,
						author.role === 'admin'
							? 'text-green'
							: author.role === 'moderator'
								? 'text-orange'
								: '',
					]"
				>
					<AutoLink
						:to="noLinks ? '' : `/user/${author.username}`"
						class="inline min-w-0 max-w-full items-center no-underline hover:underline hover:[filter:var(--hover-filter)] focus-visible:underline focus-visible:[filter:var(--hover-filter)] active:[filter:var(--active-filter)]"
					>
						{{ author.username }}
					</AutoLink>
					<ThreadRoleBadge :role="author.role" />
					<EyeOffIcon
						v-if="isPrivateMessage"
						v-tooltip="formatMessage(imageMessages.privateMessage)"
						class="mb-px ml-1 text-orange"
					/>
					<MicrophoneIcon
						v-if="report && message.author_id === report.reporter_user?.id"
						v-tooltip="formatMessage(imageMessages.reporter)"
						class="text-purple"
					/>
					<span
						v-if="message.preview"
						class="border-blue/60 rounded-full border border-solid bg-highlight-blue px-2 py-0.5 text-xs font-semibold text-blue"
					>
						{{ formatMessage(imageMessages.preview) }}
					</span>
				</span>
			</template>
			<template v-else>
				<span
					v-if="
						![
							'legacy_project_message',
							'tech_review_entered',
							'tech_review_exited',
							'tech_review_exit_file_deleted',
						].includes(message.body.type)
					"
					class="message__author min-w-0 max-w-full font-bold"
					:class="[authorClasses, isAutoApproval ? '' : 'text-orange']"
				>
					{{ formatMessage(isAutoApproval ? imageMessages.projectOwner : imageMessages.moderator) }}
					<ThreadRoleBadge v-if="!isAutoApproval" role="moderator" />
				</span>
			</template>
			<div
				v-if="message.body.type === 'text'"
				v-image-previews="imagePreviews ? formatMessage(imageMessages.openImage) : null"
				class="message__body markdown-body min-w-0 max-w-full leading-5 [&>:first-child]:mt-0 [&>:last-child]:mb-0"
				:class="[
					bodyClasses,
					imagePreviews
						? '[&_img[role=button]:focus-visible]:outline [&_img[role=button]:focus-visible]:outline-2 [&_img[role=button]:focus-visible]:outline-offset-2 [&_img[role=button]:focus-visible]:outline-brand [&_img[role=button]]:cursor-zoom-in'
						: '',
				]"
				@click="openImage"
				@keydown="handleImageKeydown"
				v-html="formattedMessage"
			/>
			<div
				v-else
				class="message__body min-w-0 max-w-full leading-5 [&_.version-badge]:relative [&_.version-badge]:top-0.5 [&_.version-badge]:!inline-flex [&_.version-badge]:align-baseline"
				:class="bodyClasses"
			>
				<span v-if="message.body.type === 'legacy_project_message'">
					{{ formatMessage(imageMessages.legacyProjectMessage) }}
				</span>
				<span v-if="message.body.type === 'deleted'">
					{{ formatMessage(imageMessages.deletedMessage) }}
				</span>
				<span v-else-if="isAutoApproval">
					<IntlFormatted :message-id="imageMessages.autoApproval">
						<template #status><Badge :type="message.body.new_status" /></template>
					</IntlFormatted>
				</span>
				<template v-else-if="message.body.type === 'status_change'">
					<span v-if="message.body.new_status === 'processing'">
						{{ formatMessage(imageMessages.submittedForReview) }}
					</span>
					<span v-else-if="message.body.old_status === 'processing'">
						<IntlFormatted
							:message-id="
								reviewIssueCount > 0 ? imageMessages.reviewWithIssues : imageMessages.review
							"
							:values="{ count: reviewIssueCount }"
						>
							<template #status><Badge :type="message.body.new_status" /></template>
						</IntlFormatted>
					</span>
					<span v-else-if="message.body.new_status === 'draft'">
						<IntlFormatted :message-id="imageMessages.revertedToDraft">
							<template #status="{ children }">
								<span class="whitespace-nowrap"
									><Badge :type="message.body.new_status" /><component :is="() => children"
								/></span>
							</template>
						</IntlFormatted>
					</span>
					<span v-else>
						<IntlFormatted :message-id="imageMessages.statusChanged">
							<template #old-status><Badge :type="message.body.old_status" /></template>
							<template #status="{ children }">
								<span class="whitespace-nowrap"
									><Badge :type="message.body.new_status" /><component :is="() => children"
								/></span>
							</template>
						</IntlFormatted>
					</span>
				</template>
				<span v-else-if="message.body.type === 'thread_closure'">
					{{ formatMessage(imageMessages.threadClosed) }}
				</span>
				<span v-else-if="message.body.type === 'thread_reopen'">
					{{ formatMessage(imageMessages.threadReopened) }}
				</span>
				<span v-else-if="message.body.type === 'tech_review'">
					<IntlFormatted :message-id="imageMessages.techReviewCompleted">
						<template #status="{ children }">
							<span class="whitespace-nowrap"
								><Badge :type="message.body.verdict" /><component :is="() => children"
							/></span>
						</template>
					</IntlFormatted>
				</span>
				<span v-else-if="message.body.type === 'tech_review_entered'">
					{{ formatMessage(imageMessages.techReviewEntered) }}
				</span>
				<span v-else-if="message.body.type === 'tech_review_exited'">
					{{ formatMessage(imageMessages.techReviewExited) }}
				</span>
				<span v-else-if="message.body.type === 'tech_review_exit_file_deleted'">
					{{ formatMessage(imageMessages.techReviewExitFileDeleted) }}
				</span>
			</div>
			<div class="mt-1 flex items-center gap-2">
				<span class="message__date block text-xs text-secondary">
					<span v-tooltip="formatDateTime(message.created)">
						{{ timeSincePosted }}
					</span>
				</span>
				<div
					v-if="isStaff(auth.user) && message.author_id === auth.user.id"
					class="message__actions ml-auto group-focus-within/thread:opacity-100 group-hover/thread:opacity-100 [@media(hover:hover)]:opacity-0"
					:class="{ hidden: noLinks }"
				>
					<TeleportOverflowMenu
						type="quiet"
						:label="formatMessage(imageMessages.moreOptions)"
						class="btn-dropdown-animation !size-6 !min-h-0 !p-1"
						:options="[
							{
								id: 'delete',
								label: formatMessage(imageMessages.delete),
								action: () => deleteMessage(),
								tone: 'red',
								hoverFilled: true,
							},
						]"
					>
						<MoreHorizontalIcon />
						<template #delete> <TrashIcon /> {{ formatMessage(imageMessages.delete) }} </template>
					</TeleportOverflowMenu>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup>
import {
	EyeOffIcon,
	InfoIcon,
	MicrophoneIcon,
	MoreHorizontalIcon,
	TrashIcon,
} from '@modrinth/assets'
import { moderationSettings } from '@modrinth/moderation'
import {
	AutoLink,
	Avatar,
	Badge,
	defineMessages,
	injectModrinthClient,
	IntlFormatted,
	TeleportOverflowMenu,
	useFormatDateTime,
	useRelativeTime,
	useVIntl,
} from '@modrinth/ui'
import { renderString } from '@modrinth/utils'

import { isStaff } from '~/helpers/users.js'

import ThreadRoleBadge from './ThreadRoleBadge.vue'

const props = defineProps({
	message: {
		type: Object,
		required: true,
	},
	issues: {
		type: /** @type {import('vue').PropType<readonly import('@modrinth/api-client').Labrinth.Threads.v3.ThreadIssue[]>} */ (
			Array
		),
		default: () => [],
	},
	projectOwnerId: {
		type: String,
		default: null,
	},
	report: {
		type: Object,
		default: null,
	},
	members: {
		type: Object,
		default: () => {},
	},
	forceCompact: {
		type: Boolean,
		default: false,
	},
	noLinks: {
		type: Boolean,
		default: false,
	},
	raised: {
		type: Boolean,
		default: false,
	},
	imagePreviews: {
		type: Boolean,
		default: false,
	},
	auth: {
		type: Object,
		required: true,
	},
})

const hasBody = computed(() => props.message.body.type === 'text' && !props.forceCompact)
const authorClasses = 'inline [&>svg]:ms-1 [&>svg]:me-1 [&>svg]:inline-block [&>svg]:align-middle'
const bodyClasses = computed(() => (hasBody.value ? 'mt-1' : 'ms-1 inline'))

const emit = defineEmits(['update-thread', 'open-image'])
const settings = useModerationSettings()
const client = injectModrinthClient()
const isAutoApproval = computed(() => props.message.body.type === 'auto_approval')
const reviewIssueCount = computed(() => {
	const eventTime = new Date(props.message.created).getTime()
	const reviewWindow = 10 * 60 * 1000
	return props.issues.filter(
		(issue) => Math.abs(new Date(issue.created_at).getTime() - eventTime) <= reviewWindow,
	).length
})
const author = computed(
	() => props.members[isAutoApproval.value ? props.projectOwnerId : props.message.author_id],
)

const { formatMessage } = useVIntl()

const imageMessages = defineMessages({
	projectOwner: {
		id: 'thread.message.project-owner',
		defaultMessage: 'Project owner',
	},
	moderator: {
		id: 'thread.message.moderator',
		defaultMessage: 'Moderator',
	},
	autoApproval: {
		id: 'thread.message.auto-approval',
		defaultMessage:
			'resolved all issues and the project status has been automatically set to <status>approved</status>.',
	},
	review: {
		id: 'thread.message.review',
		defaultMessage: 'reviewed the project and set its status to <status>status</status>.',
	},
	reviewWithIssues: {
		id: 'thread.message.review-with-issues',
		defaultMessage:
			'reviewed the project and set its status to <status>status</status> with {count, plural, one {# issue} other {# issues}}.',
	},
	openImage: {
		id: 'thread.message.open-image',
		defaultMessage: 'Open image',
	},
	privateMessage: {
		id: 'thread.message.private-message',
		defaultMessage: 'Only visible to moderators',
	},
	reporter: { id: 'thread.message.reporter', defaultMessage: 'Reporter' },
	preview: { id: 'thread.message.preview', defaultMessage: 'Preview' },
	legacyProjectMessage: {
		id: 'thread.message.legacy-project-message',
		defaultMessage:
			'This project was published on Modrinth before moderation threads existed and may be missing moderation history.',
	},
	deletedMessage: {
		id: 'thread.message.deleted-message',
		defaultMessage: 'posted a message that has been deleted.',
	},
	submittedForReview: {
		id: 'thread.message.submitted-for-review',
		defaultMessage: 'submitted the project for review.',
	},
	revertedToDraft: {
		id: 'thread.message.reverted-to-draft',
		defaultMessage: 'reverted this project back to a <status>.</status>',
	},
	statusChanged: {
		id: 'thread.message.status-changed',
		defaultMessage:
			"changed the project's status from <old-status>status</old-status> to <status>.</status>",
	},
	threadClosed: {
		id: 'thread.message.thread-closed',
		defaultMessage: 'closed the thread.',
	},
	threadReopened: {
		id: 'thread.message.thread-reopened',
		defaultMessage: 'reopened the thread.',
	},
	techReviewCompleted: {
		id: 'thread.message.tech-review-completed',
		defaultMessage: 'completed technical review and marked project as <status>.</status>',
	},
	techReviewEntered: {
		id: 'thread.message.tech-review-entered',
		defaultMessage: 'The project has entered the technical review queue.',
	},
	techReviewExited: {
		id: 'thread.message.tech-review-exited',
		defaultMessage:
			'The project has left the technical review queue as all pending traces have been resolved.',
	},
	techReviewExitFileDeleted: {
		id: 'thread.message.tech-review-exit-file-deleted',
		defaultMessage:
			'The project has left the technical review queue as all files pending review were deleted by the user.',
	},
	moreOptions: { id: 'thread.message.more-options', defaultMessage: 'More options' },
	delete: { id: 'thread.message.delete', defaultMessage: 'Delete' },
	sentImage: { id: 'thread.message.sent-image', defaultMessage: 'sent an image.' },
	sentMessage: { id: 'thread.message.sent-message', defaultMessage: 'sent a message.' },
})

function prepareImagePreviews(element, binding) {
	if (!binding.value && !binding.oldValue) return
	for (const image of element.querySelectorAll('img')) {
		if (binding.value && image.getAttribute('src')) {
			image.setAttribute('role', 'button')
			image.setAttribute('tabindex', '0')
			image.setAttribute('aria-label', image.alt || binding.value)
		} else {
			image.removeAttribute('role')
			image.removeAttribute('tabindex')
			image.removeAttribute('aria-label')
		}
	}
}

const vImagePreviews = {
	mounted: prepareImagePreviews,
	updated: prepareImagePreviews,
}

function openImage(event) {
	if (!props.imagePreviews || !(event.target instanceof HTMLImageElement)) return
	const image = event.target
	if (!image.getAttribute('src')) return
	event.preventDefault()
	event.stopPropagation()
	emit('open-image', { src: image.currentSrc || image.src, alt: image.alt, element: image })
}

function handleImageKeydown(event) {
	if (event.key === 'Enter' || event.key === ' ') openImage(event)
}

const formattedMessage = computed(() => {
	const body = renderString(props.message.body.body)
	if (props.forceCompact) {
		const hasImage = body.includes('<img')
		const noHtml = body.replace(/<\/?[^>]+(>|$)/g, '')
		if (noHtml.trim()) {
			return noHtml
		} else if (hasImage) {
			return formatMessage(imageMessages.sentImage)
		} else {
			return formatMessage(imageMessages.sentMessage)
		}
	}
	return body
})

const formatRelativeTime = useRelativeTime()
const formatDateTime = useFormatDateTime({
	timeStyle: 'short',
	dateStyle: 'long',
})

const timeSincePosted = ref(formatRelativeTime(props.message.created))

const isPrivateMessage = computed(() => {
	return (
		props.message.body.private ||
		[
			'tech_review',
			'tech_review_entered',
			'tech_review_exited',
			'tech_review_exit_file_deleted',
		].includes(props.message.body.type)
	)
})

async function deleteMessage() {
	await client.labrinth.threads_v3.deleteMessage(props.message.id)
	emit('update-thread')
}
</script>
