<template>
	<NewModal
		ref="modal"
		:header="formatMessage(shouldCancel ? messages.cancelHeader : messages.header, { username })"
		max-width="470px"
	>
		<div class="flex flex-col gap-4">
			<Admonition type="warning">
				{{
					formatMessage(shouldCancel ? messages.cancelWarningBody : messages.warningBody, {
						username,
					})
				}}
			</Admonition>

			<div class="flex min-w-0 items-center gap-2 rounded-[20px] bg-surface-2 p-3">
				<Avatar
					:src="avatarUrl"
					:alt="formatMessage(messages.userAvatarAlt, { username })"
					:tint-by="username"
					size="40px"
					circle
					no-shadow
				/>
				<div class="flex min-w-0 flex-1 flex-col gap-0.5">
					<div class="flex min-w-0 items-center gap-1.5">
						<span class="min-w-0 truncate font-medium text-contrast">{{ username }}</span>
						<span
							v-if="memberStatusLabel"
							class="inline-flex h-6 shrink-0 items-center rounded-full border border-solid px-2 py-1 text-sm font-medium leading-none"
							:class="memberStatusClasses"
						>
							{{ memberStatusLabel }}
						</span>
					</div>
					<span class="truncate text-sm text-secondary">{{ memberSubtitle }}</span>
				</div>
			</div>

			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{
					formatMessage(messages.whatHappensLabel)
				}}</span>
				<ul class="m-0 list-disc pl-6 text-primary">
					<li
						v-for="effect in effectMessages"
						:key="effect.id"
						class="leading-6 marker:text-secondary"
					>
						{{ formatMessage(effect) }}
					</li>
				</ul>
			</div>

			<div class="flex justify-end gap-2 pt-1">
				<ButtonStyled type="outlined">
					<button class="!border !border-surface-5" @click="hide">
						<XIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="orange">
					<button @click="confirm">
						<XIcon v-if="shouldCancel" aria-hidden="true" />
						<UserXIcon v-else aria-hidden="true" />
						{{ formatMessage(shouldCancel ? messages.cancelButton : messages.removeButton) }}
					</button>
				</ButtonStyled>
			</div>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import { UserXIcon, XIcon } from '@modrinth/assets'
import { computed, ref } from 'vue'

import { useRelativeTime } from '../../../composables'
import { defineMessages, useVIntl } from '../../../composables/i18n'
import { commonMessages } from '../../../utils/common-messages'
import Admonition from '../../base/Admonition.vue'
import Avatar from '../../base/Avatar.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'
import NewModal from '../../modal/NewModal.vue'
import type { ServerAccessRole } from './types'

const props = withDefaults(
	defineProps<{
		username: string
		avatarUrl?: string
		role?: ServerAccessRole
		joinedAt?: string | null
		pending?: boolean
		shouldCancel?: boolean
	}>(),
	{
		avatarUrl: undefined,
		role: undefined,
		joinedAt: null,
		pending: false,
		shouldCancel: false,
	},
)

const emit = defineEmits<{
	remove: []
}>()

const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()
const modal = ref<InstanceType<typeof NewModal>>()

const messages = defineMessages({
	header: {
		id: 'servers.remove-access-modal.header',
		defaultMessage: 'Remove user',
	},
	cancelHeader: {
		id: 'servers.remove-access-modal.cancel-header',
		defaultMessage: 'Cancel invite',
	},
	warningBody: {
		id: 'servers.remove-access-modal.warning-body',
		defaultMessage:
			"If you remove a user from your server, you'll need to re-invite them to restore access.",
	},
	cancelWarningBody: {
		id: 'servers.remove-access-modal.cancel-warning-body',
		defaultMessage:
			'If you cancel this invite, {username} will need a new invitation before they can join this server.',
	},
	removeButton: {
		id: 'servers.remove-access-modal.remove-button',
		defaultMessage: 'Remove user',
	},
	cancelButton: {
		id: 'servers.remove-access-modal.cancel-button',
		defaultMessage: 'Cancel invite',
	},
	userAvatarAlt: {
		id: 'servers.remove-access-modal.user-avatar-alt',
		defaultMessage: "{username}'s avatar",
	},
	whatHappensLabel: {
		id: 'servers.remove-access-modal.what-happens-label',
		defaultMessage: 'What happens?',
	},
	removeEffectAccess: {
		id: 'servers.remove-access-modal.remove-effect-access',
		defaultMessage:
			'They will immediately lose access to the server panel and will no longer be able to edit content',
	},
	removeEffectJoin: {
		id: 'servers.remove-access-modal.remove-effect-join',
		defaultMessage:
			'They will still be able to join and play on the server unless you make separate changes',
	},
	cancelEffectAccess: {
		id: 'servers.remove-access-modal.cancel-effect-access',
		defaultMessage: 'They will not be added to this server',
	},
	cancelEffectInvite: {
		id: 'servers.remove-access-modal.cancel-effect-invite',
		defaultMessage: 'You can send them another invite later',
	},
	addedLabel: {
		id: 'servers.remove-access-modal.added-label',
		defaultMessage: 'Added {time}',
	},
	invitedLabel: {
		id: 'servers.remove-access-modal.invited-label',
		defaultMessage: 'Invited {time}',
	},
	pendingInviteLabel: {
		id: 'servers.remove-access-modal.pending-invite-label',
		defaultMessage: 'Pending invite',
	},
	unknownAddedLabel: {
		id: 'servers.remove-access-modal.unknown-added-label',
		defaultMessage: 'Added date unknown',
	},
	ownerRole: {
		id: 'servers.access-role.owner',
		defaultMessage: 'Owner',
	},
	editorRole: {
		id: 'servers.access-role.editor',
		defaultMessage: 'Editor',
	},
	viewerRole: {
		id: 'servers.access-role.viewer',
		defaultMessage: 'Limited',
	},
})

const memberStatusLabel = computed(() => {
	if (!props.role) return null
	return formatRole(props.role)
})

const memberStatusClasses = computed(() => {
	if (!props.role) return ''
	return roleClasses(props.role)
})

const memberSubtitle = computed(() => {
	if (props.shouldCancel || props.pending) {
		return props.joinedAt
			? formatMessage(messages.invitedLabel, { time: formatRelativeTime(props.joinedAt) })
			: formatMessage(messages.pendingInviteLabel)
	}

	return props.joinedAt
		? formatMessage(messages.addedLabel, { time: formatRelativeTime(props.joinedAt) })
		: formatMessage(messages.unknownAddedLabel)
})

const effectMessages = computed(() =>
	props.shouldCancel
		? [messages.cancelEffectAccess, messages.cancelEffectInvite]
		: [messages.removeEffectAccess, messages.removeEffectJoin],
)

function formatRole(role: ServerAccessRole): string {
	switch (role) {
		case 'owner':
			return formatMessage(messages.ownerRole)
		case 'editor':
			return formatMessage(messages.editorRole)
		case 'viewer':
			return formatMessage(messages.viewerRole)
	}
}

function roleClasses(role: ServerAccessRole): string {
	switch (role) {
		case 'owner':
			return 'border-orange bg-highlight-orange text-orange'
		case 'editor':
			return 'border-green bg-highlight-green text-brand'
		case 'viewer':
			return 'border-blue bg-highlight-blue text-blue'
	}
}

function show() {
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

function confirm() {
	hide()
	emit('remove')
}

defineExpose({ show, hide })
</script>
