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
			v-if="members[message.author_id]"
			class="message__icon flex min-w-0 max-w-full items-center no-underline"
			:to="noLinks ? '' : `/user/${members[message.author_id].username}`"
			tabindex="-1"
			aria-hidden="true"
		>
			<Avatar size="2rem" :src="members[message.author_id].avatar_url" circle :raised="raised" />
		</AutoLink>
		<div
			v-else
			class="message__icon inline-flex size-8 shrink-0 items-center justify-center rounded-full text-orange [&>svg]:h-3/5 [&>svg]:w-3/5"
			:class="raised ? 'bg-surface-3' : 'bg-surface-2'"
		>
			<InfoIcon v-if="message.body.type === 'legacy_project_message'" class="text-blue" />
			<ThreadRoleBadge
				v-else
				:role="message.body.type === 'auto_approval' ? 'system' : 'moderator'"
				avatar
			/>
		</div>
		<div class="message__content min-w-0">
			<template v-if="members[message.author_id]">
				<span
					class="message__author min-w-0 max-w-full font-bold"
					:class="[
						authorClasses,
						members[message.author_id].role === 'admin'
							? 'text-green'
							: members[message.author_id].role === 'moderator'
								? 'text-orange'
								: '',
					]"
				>
					<AutoLink
						:to="noLinks ? '' : `/user/${members[message.author_id].username}`"
						class="inline min-w-0 max-w-full items-center no-underline hover:underline hover:[filter:var(--hover-filter)] focus-visible:underline focus-visible:[filter:var(--hover-filter)] active:[filter:var(--active-filter)]"
					>
						{{ members[message.author_id].username }}
					</AutoLink>
					<ThreadRoleBadge :role="members[message.author_id].role" />
					<EyeOffIcon
						v-if="isPrivateMessage"
						v-tooltip="'Only visible to moderators'"
						class="mb-px ml-1 text-orange"
					/>
					<MicrophoneIcon
						v-if="report && message.author_id === report.reporter_user?.id"
						v-tooltip="'Reporter'"
						class="text-purple"
					/>
					<span
						v-if="message.preview"
						class="border-blue/60 rounded-full border border-solid bg-highlight-blue px-2 py-0.5 text-xs font-semibold text-blue"
					>
						Preview
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
					class="message__author min-w-0 max-w-full font-bold text-orange"
					:class="authorClasses"
				>
					{{
						formatMessage(
							message.body.type === 'auto_approval'
								? imageMessages.system
								: imageMessages.moderator,
						)
					}}
					<ThreadRoleBadge :role="message.body.type === 'auto_approval' ? 'system' : 'moderator'" />
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
					This project was published on Modrinth before moderation threads existed and may be
					missing moderation history.
				</span>
				<span v-if="message.body.type === 'deleted'">
					posted a message that has been deleted.
				</span>
				<IntlFormatted
					v-else-if="message.body.type === 'auto_approval'"
					:message-id="imageMessages.autoApproval"
				>
					<template #status><Badge :type="message.body.new_status" /></template>
				</IntlFormatted>
				<template v-else-if="message.body.type === 'status_change'">
					<span v-if="message.body.new_status === 'processing'">
						submitted the project for review.
					</span>
					<span v-else-if="message.body.old_status === 'processing'" class="-ml-[3px]">
						reviewed the project and set its status to
						<span class="whitespace-nowrap"><Badge :type="message.body.new_status" />.</span>
					</span>
					<span v-else-if="message.body.new_status === 'draft'">
						reverted this project back to a
						<span class="whitespace-nowrap"><Badge :type="message.body.new_status" />.</span>
					</span>
					<span v-else>
						changed the project's status from <Badge :type="message.body.old_status" /> to
						<span class="whitespace-nowrap"><Badge :type="message.body.new_status" />.</span>
					</span>
				</template>
				<span v-else-if="message.body.type === 'thread_closure'">closed the thread.</span>
				<span v-else-if="message.body.type === 'thread_reopen'">reopened the thread.</span>
				<span v-else-if="message.body.type === 'tech_review'">
					completed technical review and marked project as
					<span class="whitespace-nowrap"><Badge :type="message.body.verdict" />.</span>
				</span>
				<span v-else-if="message.body.type === 'tech_review_entered'">
					The project has entered the technical review queue.
				</span>
				<span v-else-if="message.body.type === 'tech_review_exited'">
					The project has left the technical review queue as all pending traces have been resolved.
				</span>
				<span v-else-if="message.body.type === 'tech_review_exit_file_deleted'">
					The project has left the technical review queue as all files pending review were deleted
					by the user.
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
						label="More options"
						class="btn-dropdown-animation !size-6 !min-h-0 !p-1"
						:options="[
							{
								id: 'delete',
								label: 'Delete',
								action: () => deleteMessage(),
								tone: 'red',
								hoverFilled: true,
							},
						]"
					>
						<MoreHorizontalIcon />
						<template #delete> <TrashIcon /> Delete </template>
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
const bodyClasses = computed(() => (hasBody.value ? 'mt-1' : 'inline'))

const emit = defineEmits(['update-thread', 'open-image'])
const settings = useModerationSettings()
const client = injectModrinthClient()
const { formatMessage } = useVIntl()
const imageMessages = defineMessages({
	system: { id: 'thread.message.system', defaultMessage: 'Modrinth' },
	moderator: { id: 'thread.message.moderator', defaultMessage: 'Moderator' },
	autoApproval: {
		id: 'thread.message.auto-approval',
		defaultMessage:
			'All moderation issues have been resolved and your project is automatically approved with status <status>approved</status>',
	},
	openImage: {
		id: 'thread.message.open-image',
		defaultMessage: 'Open image',
	},
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
			return 'sent an image.'
		} else {
			return 'sent a message.'
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
