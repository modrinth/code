<template>
	<NewModal
		ref="modal"
		:header="header"
		width="min(34rem, calc(100vw - 2rem))"
		max-width="34rem"
		no-padding
		noblur
	>
		<div class="flex max-h-[calc(100vh-8rem)] min-h-0 flex-col">
			<div class="border-0 border-b border-solid border-surface-5 p-6">
				<div class="flex items-start gap-2">
					<Combobox
						:key="searchInputKey"
						:model-value="undefined"
						:options="searchOptions"
						:search-value="searchTarget"
						:search-placeholder="searchPlaceholderLabel"
						:placeholder="searchPlaceholderLabel"
						:no-options-message="searchLookupMessage"
						:min-search-length-to-open="searchMinimumLength"
						:disable-search-filter="usesRemoteSearch"
						class="min-w-0 flex-1"
						searchable
						show-search-icon
						:show-chevron="false"
						search-type="search"
						search-name="modrinth-player-invite-search"
						search-inputmode="search"
						search-autocomplete="new-password"
						search-autocorrect="off"
						search-autocapitalize="none"
						:search-spellcheck="false"
						:search-input-attrs="passwordManagerIgnoreAttrs"
						@search-input="handleSearchInput"
						@select="handleSearchSelect"
					>
						<template #option="{ item, isSelected }">
							<div class="flex min-w-0 items-center gap-2">
								<Avatar
									:src="findSearchUser(item.value)?.avatarUrl"
									:alt="formatMessage(messages.avatarAlt, { username: item.label })"
									:tint-by="item.label"
									size="1.5rem"
									circle
									no-shadow
								/>
								<span
									class="min-w-0 truncate font-semibold"
									:class="isSelected ? 'text-contrast' : 'text-primary'"
								>
									{{ item.label }}
								</span>
							</div>
						</template>
					</Combobox>
					<ButtonStyled color="brand">
						<button
							v-tooltip="searchInviteTooltip"
							class="shrink-0"
							:disabled="!canInviteSearchTarget"
							@click="inviteSearchTarget"
						>
							<PlusIcon aria-hidden="true" />
							{{ addButtonLabel }}
						</button>
					</ButtonStyled>
				</div>
			</div>

			<div class="min-h-[11rem] overflow-y-auto bg-surface-2 px-6 py-4">
				<div class="mb-2 text-base font-semibold text-primary">
					{{ friendsHeading }}
				</div>
				<div
					v-if="friends.length === 0"
					class="flex min-h-32 items-center justify-center text-secondary"
				>
					{{ emptyFriendsLabel }}
				</div>
				<div v-else class="-mx-6 flex flex-col">
					<InvitePlayersModalUserRow
						v-for="friend in sortedFriends"
						:key="friend.id"
						:user="friend"
						:avatar-alt="formatMessage(messages.avatarAlt, { username: friend.username })"
						:added-label="addedButtonLabel"
						:cancel-label="cancelButtonLabel"
						:invite-label="inviteButtonLabel"
						:requested-label="requestedButtonLabel"
						:requested-tooltip="requestedTooltip(friend.username)"
						:user-profile-link="userProfileLink"
						:disabled="!canInvite"
						@invite="inviteFriend"
						@cancel="cancelInvite"
					/>
				</div>
			</div>

			<div v-if="link" class="border-0 border-t border-solid border-surface-5 p-6">
				<div class="flex flex-col gap-2">
					<div class="text-base font-semibold text-contrast">
						{{ inviteLinkHeading }}
					</div>
					<ButtonStyled>
						<button
							type="button"
							class="!h-10 w-full !justify-between !px-4 text-left !shadow-none"
							@click="copyInviteLink"
						>
							<span class="min-w-0 truncate text-base font-semibold text-primary">
								{{ link }}
							</span>
							<ClipboardCopyIcon class="size-5 shrink-0 text-secondary" aria-hidden="true" />
						</button>
					</ButtonStyled>
					<p v-if="link && linkExpiryDescription" class="m-0 text-base text-primary">
						{{ linkExpiryDescription }}
						<button
							v-if="updateInviteLink"
							type="button"
							class="cursor-pointer border-0 bg-transparent p-0 text-base font-medium text-blue hover:underline"
							@click="inviteLinkEditor?.show()"
						>
							{{ formatMessage(messages.editInviteLink) }}
						</button>
					</p>
				</div>
			</div>
		</div>
	</NewModal>

	<InvitePlayersModalInviteLinkEditor
		v-if="updateInviteLink"
		ref="inviteLinkEditor"
		:link-expires-at="linkExpiresAt"
		:link-max-uses="linkMaxUses"
		:update-invite-link="updateInviteLink"
	/>
</template>

<script setup lang="ts">
import { ClipboardCopyIcon, PlusIcon } from '@modrinth/assets'
import { computed, ref } from 'vue'

import { defineMessages, useVIntl } from '../../../composables/i18n'
import { injectNotificationManager } from '../../../providers'
import Avatar from '../../base/Avatar.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'
import Combobox from '../../base/Combobox.vue'
import NewModal from '../../modal/NewModal.vue'
import InvitePlayersModalInviteLinkEditor from './invite-players-modal-invite-link-editor.vue'
import InvitePlayersModalUserRow from './invite-players-modal-user-row.vue'
import type {
	InviteLinkSettings,
	InvitePlayersInvitePayload,
	InvitePlayersSearchUser,
	InvitePlayersUser,
	InvitePlayersUserProfileLink,
} from './types'
import { useInvitePlayersSearch } from './use-invite-players-search'

const props = withDefaults(
	defineProps<{
		header?: string
		friends?: InvitePlayersUser[]
		suggestions?: InvitePlayersSearchUser[]
		searchUsers?: (query: string) => Promise<InvitePlayersSearchUser[]>
		link?: string
		linkExpiresAt?: string | Date | null
		linkMaxUses?: number
		updateInviteLink?: (settings: InviteLinkSettings) => Promise<void>
		friendsLabel?: string
		searchPlaceholder?: string
		addLabel?: string
		inviteLabel?: string
		addedLabel?: string
		cancelLabel?: string
		requestedLabel?: string
		emptyFriendsLabel?: string
		canInvite?: boolean
		inviteDisabledMessage?: string
		userProfileLink?: (username: string) => InvitePlayersUserProfileLink
	}>(),
	{
		header: 'Share instance',
		friends: () => [],
		suggestions: () => [],
		canInvite: true,
		linkMaxUses: 10,
	},
)

const emit = defineEmits<{
	invite: [payload: InvitePlayersInvitePayload]
	cancel: [user: InvitePlayersUser]
	'copy-link': [link: string]
}>()

const { formatMessage } = useVIntl()
const notificationManager = injectNotificationManager(null)
const modal = ref<InstanceType<typeof NewModal> | null>(null)
const inviteLinkEditor = ref<InstanceType<typeof InvitePlayersModalInviteLinkEditor> | null>(null)

const messages = defineMessages({
	friendsHeading: {
		id: 'sharing.invite-players-modal.friends-heading',
		defaultMessage: 'Your friends - {count}',
	},
	searchPlaceholder: {
		id: 'sharing.invite-players-modal.search-placeholder',
		defaultMessage: 'Enter Modrinth username',
	},
	addButton: {
		id: 'sharing.invite-players-modal.add',
		defaultMessage: 'Add',
	},
	inviteButton: {
		id: 'sharing.invite-players-modal.invite',
		defaultMessage: 'Invite',
	},
	addedButton: {
		id: 'sharing.invite-players-modal.added',
		defaultMessage: 'Added',
	},
	cancelButton: {
		id: 'sharing.invite-players-modal.cancel',
		defaultMessage: 'Cancel',
	},
	requestedButton: {
		id: 'sharing.invite-players-modal.requested',
		defaultMessage: 'Request sent',
	},
	requestedTooltip: {
		id: 'sharing.invite-players-modal.requested-tooltip',
		defaultMessage: '{username} needs to accept your friend request first',
	},
	noFriends: {
		id: 'sharing.invite-players-modal.no-friends',
		defaultMessage: 'No friends found.',
	},
	noSearchResults: {
		id: 'sharing.invite-players-modal.no-search-results',
		defaultMessage: 'No matching users found.',
	},
	searching: {
		id: 'sharing.invite-players-modal.searching',
		defaultMessage: 'Searching...',
	},
	alreadyInvited: {
		id: 'sharing.invite-players-modal.already-invited',
		defaultMessage: 'This user has already been invited.',
	},
	inviteLinkHeading: {
		id: 'sharing.invite-players-modal.invite-link-heading',
		defaultMessage: 'Or use an invite link',
	},
	inviteExpiryDescription: {
		id: 'sharing.invite-players-modal.invite-expiry-description',
		defaultMessage: 'Your invite link expires in {duration}.',
	},
	editInviteLink: {
		id: 'sharing.invite-players-modal.edit-invite-link',
		defaultMessage: 'Edit invite link.',
	},
	linkCopiedTitle: {
		id: 'sharing.invite-players-modal.link-copied-title',
		defaultMessage: 'Link copied',
	},
	linkCopiedText: {
		id: 'sharing.invite-players-modal.link-copied-text',
		defaultMessage: 'The invite link has been copied to your clipboard.',
	},
	linkCopyFailedTitle: {
		id: 'sharing.invite-players-modal.link-copy-failed-title',
		defaultMessage: 'Failed to copy link',
	},
	avatarAlt: {
		id: 'sharing.invite-players-modal.avatar-alt',
		defaultMessage: "{username}'s avatar",
	},
})

const friendsHeading = computed(
	() =>
		props.friendsLabel ??
		formatMessage(messages.friendsHeading, {
			count: props.friends.length,
		}),
)
const searchPlaceholderLabel = computed(
	() => props.searchPlaceholder ?? formatMessage(messages.searchPlaceholder),
)
const addButtonLabel = computed(() => props.addLabel ?? formatMessage(messages.addButton))
const inviteButtonLabel = computed(() => props.inviteLabel ?? formatMessage(messages.inviteButton))
const addedButtonLabel = computed(() => props.addedLabel ?? formatMessage(messages.addedButton))
const cancelButtonLabel = computed(() => props.cancelLabel ?? formatMessage(messages.cancelButton))
const requestedButtonLabel = computed(
	() => props.requestedLabel ?? formatMessage(messages.requestedButton),
)
const requestedTooltip = (username: string) =>
	formatMessage(messages.requestedTooltip, {
		username,
	})
const emptyFriendsLabel = computed(
	() => props.emptyFriendsLabel ?? formatMessage(messages.noFriends),
)
const inviteLinkHeading = computed(() => formatMessage(messages.inviteLinkHeading))
const linkExpiryDescription = computed(() => {
	if (!props.linkExpiresAt) return ''

	const expiresAt = new Date(props.linkExpiresAt)
	if (Number.isNaN(expiresAt.getTime())) return ''
	const hours = Math.max(1, Math.ceil((expiresAt.getTime() - Date.now()) / 3_600_000))
	const duration =
		hours < 48 ? `${hours} ${hours === 1 ? 'hour' : 'hours'}` : `${Math.ceil(hours / 24)} days`

	return formatMessage(messages.inviteExpiryDescription, { duration })
})
const inviteDisabledMessage = computed(
	() => props.inviteDisabledMessage ?? formatMessage(messages.alreadyInvited),
)
const {
	searchTarget,
	searchInputKey,
	searchOptions,
	searchLookupMessage,
	searchMinimumLength,
	usesRemoteSearch,
	passwordManagerIgnoreAttrs,
	sortedFriends,
	canInviteSearchTarget,
	searchInviteTooltip,
	findSearchUser,
	handleSearchInput,
	handleSearchSelect,
	inviteSearchTarget,
	resetSearch,
} = useInvitePlayersSearch({
	friends: () => props.friends,
	suggestions: () => props.suggestions,
	searchUsers: () => props.searchUsers,
	canInvite: () => props.canInvite,
	inviteDisabledMessage,
	alreadyInvitedMessage: () => formatMessage(messages.alreadyInvited),
	searchingMessage: () => formatMessage(messages.searching),
	noResultsMessage: () => formatMessage(messages.noSearchResults),
	onInvite: (payload) => emit('invite', payload),
})

function inviteFriend(friend: InvitePlayersUser) {
	if (!props.canInvite) return
	emit('invite', {
		user: friend,
		source: 'friend',
	})
}

function cancelInvite(friend: InvitePlayersUser) {
	if (!props.canInvite) return
	emit('cancel', friend)
}

async function copyInviteLink() {
	if (!props.link) return

	emit('copy-link', props.link)

	try {
		await navigator.clipboard.writeText(props.link)
		notificationManager?.addNotification({
			type: 'success',
			title: formatMessage(messages.linkCopiedTitle),
			text: formatMessage(messages.linkCopiedText),
		})
	} catch (error) {
		const message = error instanceof Error ? error.message : String(error)
		notificationManager?.addNotification({
			type: 'error',
			title: formatMessage(messages.linkCopyFailedTitle),
			text: message,
		})
	}
}

function show(event?: MouseEvent) {
	resetSearch()
	modal.value?.show(event)
}

function hide() {
	modal.value?.hide()
}

defineExpose({ show, hide })
</script>
