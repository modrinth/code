<template>
	<div
		class="message px-4 py-3"
		:class="{
			'has-body': message.body.type === 'text' && !forceCompact,
			'no-actions': noLinks,
			private: isPrivateMessage,
			'show-private-bg': flags.showModeratorPrivateMessageHighlight,
		}"
	>
		<template v-if="members[message.author_id]">
			<AutoLink
				class="message__icon"
				:to="noLinks ? '' : `/user/${members[message.author_id].username}`"
				tabindex="-1"
				aria-hidden="true"
			>
				<Avatar
					class="message__icon"
					:src="members[message.author_id].avatar_url"
					circle
					:raised="raised"
				/>
			</AutoLink>
			<span :class="`message__author role-${members[message.author_id].role}`">
				<AutoLink :to="noLinks ? '' : `/user/${members[message.author_id].username}`">
					{{ members[message.author_id].username }}
				</AutoLink>
				<ScaleIcon v-if="members[message.author_id].role === 'moderator'" v-tooltip="'Moderator'" />
				<ModrinthIcon
					v-else-if="members[message.author_id].role === 'admin'"
					v-tooltip="'Modrinth Team'"
				/>
				<EyeOffIcon
					v-if="isPrivateMessage"
					v-tooltip="'Only visible to moderators'"
					class="ml-1 text-orange"
				/>
				<MicrophoneIcon
					v-if="report && message.author_id === report.reporter_user?.id"
					v-tooltip="'Reporter'"
					class="reporter-icon"
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
			<div
				class="message__icon backed-svg circle moderation-color"
				:class="{
					raised: raised,
					'system-message-icon': ['tech_review_entered', 'tech_review_exit_file_deleted'].includes(
						message.body.type,
					),
				}"
			>
				<ScaleIcon />
			</div>
			<span
				v-if="!['tech_review_entered', 'tech_review_exit_file_deleted'].includes(message.body.type)"
				class="message__author moderation-color"
			>
				Moderator
				<ScaleIcon v-tooltip="'Moderator'" />
			</span>
		</template>
		<div
			v-if="message.body.type === 'text'"
			class="message__body markdown-body"
			v-html="formattedMessage"
		/>
		<div v-else class="message__body status-message">
			<span v-if="message.body.type === 'deleted'"> posted a message that has been deleted. </span>
			<template v-else-if="message.body.type === 'status_change'">
				<span v-if="message.body.new_status === 'processing'">
					submitted the project for review.
				</span>
				<span v-else-if="message.body.old_status === 'processing'">
					reviewed the project and set its status to <Badge :type="message.body.new_status" />.
				</span>
				<span v-else-if="message.body.new_status === 'draft'">
					reverted this project back to a <Badge :type="message.body.new_status" />.
				</span>
				<span v-else>
					changed the project's status from <Badge :type="message.body.old_status" /> to
					<Badge :type="message.body.new_status" />.
				</span>
			</template>
			<span v-else-if="message.body.type === 'thread_closure'">closed the thread.</span>
			<span v-else-if="message.body.type === 'thread_reopen'">reopened the thread.</span>
			<span v-else-if="message.body.type === 'tech_review'">
				completed technical review and marked project as
				<Badge :type="message.body.verdict" />.
			</span>
			<span v-else-if="message.body.type === 'tech_review_entered'">
				The project has entered the technical review queue.
			</span>
			<span v-else-if="message.body.type === 'tech_review_exit_file_deleted'">
				The project has left the technical review queue as all files pending review were deleted by
				the user.
			</span>
		</div>
		<span class="message__date">
			<span v-tooltip="formatDateTime(message.created)">
				{{ timeSincePosted }}
			</span>
		</span>
		<div v-if="isStaff(auth.user) && message.author_id === auth.user.id" class="message__actions">
			<ButtonStyled circular type="transparent">
				<OverflowMenu
					class="btn-dropdown-animation"
					:options="[
						{
							id: 'delete',
							action: () => deleteMessage(),
							color: 'red',
							hoverFilled: true,
						},
					]"
				>
					<MoreHorizontalIcon />
					<template #delete> <TrashIcon /> Delete </template>
				</OverflowMenu>
			</ButtonStyled>
		</div>
	</div>
</template>

<script setup>
import {
	EyeOffIcon,
	MicrophoneIcon,
	ModrinthIcon,
	MoreHorizontalIcon,
	ScaleIcon,
	TrashIcon,
} from '@modrinth/assets'
import {
	AutoLink,
	Avatar,
	Badge,
	ButtonStyled,
	OverflowMenu,
	useFormatDateTime,
	useRelativeTime,
} from '@modrinth/ui'
import { renderString } from '@modrinth/utils'

import { isStaff } from '~/helpers/users.js'

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
	auth: {
		type: Object,
		required: true,
	},
})

const emit = defineEmits(['update-thread'])
const flags = useFeatureFlags()

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
		['tech_review', 'tech_review_entered', 'tech_review_exit_file_deleted'].includes(
			props.message.body.type,
		)
	)
})

async function deleteMessage() {
	await useBaseFetch(`message/${props.message.id}`, {
		method: 'DELETE',
	})
	emit('update-thread')
}
</script>

<style lang="scss" scoped>
.message {
	display: flex;
	flex-direction: row;
	gap: var(--spacing-card-sm);
	flex-wrap: wrap;
	align-items: center;
	word-break: break-word;
	position: relative;

	.avatar,
	.backed-svg {
		--size: 1.5rem;
	}

	&.has-body {
		display: grid;
		grid-template:
			'icon author actions'
			'icon body actions'
			'date date date';
		grid-template-columns: min-content auto 1fr;
		row-gap: var(--spacing-card-xs);

		.message__icon {
			margin-bottom: auto;
		}

		.avatar,
		.backed-svg {
			--size: 3rem;
		}
	}

	&:not(.no-actions):hover,
	&:not(.no-actions):focus-within {
		background-color: var(--surface-2-5);

		.message__actions {
			opacity: 1;
		}
	}

	&.private.show-private-bg::before {
		content: '';
		inset: 0;
		position: absolute;
		background-color: var(--color-orange);
		opacity: 0.05;
		pointer-events: none;
	}

	&.no-actions {
		padding: 0;

		.message__actions {
			display: none;
		}
	}
}

.message__icon {
	grid-area: icon;
}

.message__author {
	grid-area: author;
	font-weight: bold;
	display: flex;
	gap: var(--spacing-card-xs);
	flex-wrap: wrap;
	flex-shrink: 0;
}

.message__date {
	grid-area: date;
	font-size: var(--font-size-xs);
	color: var(--color-text-secondary);
}

.message__actions {
	grid-area: actions;
	margin-left: auto;

	@media (hover: hover) {
		opacity: 0;
	}
}

.message__body {
	grid-area: body;
}

.status-message > span {
	display: flex;
	align-items: center;
	gap: var(--spacing-card-xs);
	flex-wrap: wrap;
}
a {
	display: flex;
	align-items: center;
	text-decoration: none;
}

a:focus-visible + .message__author a,
a:hover + .message__author a,
.message__author a:focus-visible,
.message__author a:hover {
	text-decoration: underline;
	filter: var(--hover-filter);
}

a:active + .message__author a,
.message__author a:active {
	filter: var(--active-filter);
}

.moderation-color,
.role-moderator {
	color: var(--color-orange);
}

.role-admin {
	color: var(--color-green);
}

.reporter-icon {
	color: var(--color-purple);
}

@media screen and (min-width: 600px) {
	.message {
		//grid-template:
		//  'icon author body'
		//  'date date date';
		//grid-template-columns: min-content auto 1fr;

		&.has-body {
			grid-template:
				'icon author actions'
				'icon body actions'
				'date date date';
			grid-template-columns: min-content auto 1fr;
			grid-template-rows: min-content 1fr auto;
		}
	}
}

@media screen and (min-width: 1024px) {
	.message {
		//grid-template: 'icon author body date';
		//grid-template-columns: min-content auto 1fr auto;

		&.has-body {
			grid-template:
				'icon author date actions'
				'icon body body actions';
			grid-template-columns: min-content auto 1fr;
			grid-template-rows: min-content 1fr;
		}
	}
}

.private {
	color: var(--color-icon);
}

.system-message-icon {
	--size: 2rem !important;
}
</style>
