<template>
	<NewModal
		ref="modal"
		:header="header"
		width="min(34rem, calc(100vw - 2rem))"
		max-width="34rem"
		no-padding
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
					<button
						type="button"
						class="flex h-10 w-full cursor-pointer items-center justify-between gap-3 rounded-[14px] border-none bg-surface-2 px-4 text-left transition-all hover:brightness-110 active:scale-[0.98]"
						@click="copyInviteLink"
					>
						<span class="min-w-0 truncate text-base font-semibold text-primary">
							{{ link }}
						</span>
						<ClipboardCopyIcon class="size-5 shrink-0 text-secondary" aria-hidden="true" />
					</button>
				</div>
			</div>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import { ClipboardCopyIcon, PlusIcon } from '@modrinth/assets'
import { useDebounceFn } from '@vueuse/core'
import { computed, ref, watch } from 'vue'

import { defineMessages, useVIntl } from '../../../composables/i18n'
import { injectNotificationManager } from '../../../providers'
import Avatar from '../../base/Avatar.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'
import Combobox, { type ComboboxOption } from '../../base/Combobox.vue'
import NewModal from '../../modal/NewModal.vue'
import InvitePlayersModalUserRow from './invite-players-modal-user-row.vue'
import type {
	InvitePlayersInvitePayload,
	InvitePlayersSearchUser,
	InvitePlayersUser,
	InvitePlayersUserProfileLink,
	InvitePlayersUserStatus,
} from './types'

const props = withDefaults(
	defineProps<{
		header?: string
		friends?: InvitePlayersUser[]
		suggestions?: InvitePlayersSearchUser[]
		searchUsers?: (query: string) => Promise<InvitePlayersSearchUser[]>
		link?: string
		linkDescription?: string
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
const searchTarget = ref('')
const searchInputKey = ref(0)
const selectedSearchUser = ref<InvitePlayersSearchUser | null>(null)
const remoteSearchUsers = ref<InvitePlayersSearchUser[]>([])
const searchLookupStatus = ref<'idle' | 'loading' | 'loaded'>('idle')
const searchLookupRequestId = ref(0)
const friendOrder = ref(new Map<string, number>())
const searchMinimumLength = 1
const passwordManagerIgnoreAttrs = {
	'data-1p-ignore': 'true',
	'data-bwignore': 'true',
	'data-form-type': 'other',
	'data-lpignore': 'true',
	'data-protonpass-ignore': 'true',
}

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
	inviteLinkDescription: {
		id: 'sharing.invite-players-modal.invite-link-description',
		defaultMessage:
			'This link invites your friends as players and automatically shares all required files!',
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

const normalizedSearchTarget = computed(() => searchTarget.value.trim())
const usesRemoteSearch = computed(() => !!props.searchUsers)
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
const inviteLinkDescription = computed(
	() => props.linkDescription ?? formatMessage(messages.inviteLinkDescription),
)
const inviteDisabledMessage = computed(
	() => props.inviteDisabledMessage ?? formatMessage(messages.alreadyInvited),
)
const searchLookupMessage = computed(() =>
	usesRemoteSearch.value && searchLookupStatus.value !== 'loaded'
		? formatMessage(messages.searching)
		: formatMessage(messages.noSearchResults),
)
const searchableUsers = computed(() => {
	const users = new Map<string, InvitePlayersSearchUser>()

	for (const user of [...remoteSearchUsers.value, ...props.suggestions]) {
		users.set(user.id.toLowerCase(), user)
		users.set(user.username.toLowerCase(), user)
		if (user.email) users.set(user.email.toLowerCase(), user)
	}

	return [...new Set(users.values())]
})
const searchOptions = computed<ComboboxOption<string>[]>(() =>
	searchableUsers.value.map((user) => ({
		value: user.username,
		label: user.username,
		searchTerms: [user.username, user.id, user.email].filter(Boolean) as string[],
	})),
)
const sortedFriends = computed(() =>
	props.friends
		.map((friend, index) => ({
			friend,
			order: friendOrder.value.get(friend.id) ?? friendOrder.value.size + index,
		}))
		.sort((a, b) => a.order - b.order)
		.map(({ friend }) => friend),
)
const matchedSearchUser = computed(() => {
	if (
		selectedSearchUser.value &&
		normalizeInviteKey(selectedSearchUser.value.username) ===
			normalizeInviteKey(normalizedSearchTarget.value)
	) {
		return selectedSearchUser.value
	}

	return findSearchUser(normalizedSearchTarget.value)
})
const invitedUserKeys = computed(() => {
	const keys = new Set<string>()

	for (const friend of props.friends) {
		if (friendStatus(friend) === 'available') continue
		keys.add(normalizeInviteKey(friend.id))
		keys.add(normalizeInviteKey(friend.username))
	}

	return keys
})
const searchTargetAlreadyInvited = computed(() => {
	const user = matchedSearchUser.value
	if (!user) return false

	return (
		invitedUserKeys.value.has(normalizeInviteKey(user.id)) ||
		invitedUserKeys.value.has(normalizeInviteKey(user.username))
	)
})
const canInviteSearchTarget = computed(
	() =>
		props.canInvite &&
		normalizedSearchTarget.value.length >= searchMinimumLength &&
		!!matchedSearchUser.value &&
		!searchTargetAlreadyInvited.value &&
		(!usesRemoteSearch.value ||
			searchLookupStatus.value === 'loaded' ||
			!!selectedSearchUser.value),
)
const searchInviteTooltip = computed(() => {
	if (!props.canInvite) return inviteDisabledMessage.value
	if (searchTargetAlreadyInvited.value) return formatMessage(messages.alreadyInvited)
	return undefined
})

const searchTargetUsers = useDebounceFn(async (query: string, requestId: number) => {
	const searchUsers = props.searchUsers
	if (!searchUsers) return

	try {
		const users = await searchUsers(query)
		if (requestId !== searchLookupRequestId.value || query !== normalizedSearchTarget.value) return

		remoteSearchUsers.value = users
	} catch {
		if (requestId !== searchLookupRequestId.value || query !== normalizedSearchTarget.value) return

		remoteSearchUsers.value = []
	} finally {
		if (requestId === searchLookupRequestId.value && query === normalizedSearchTarget.value) {
			searchLookupStatus.value = 'loaded'
		}
	}
}, 250)

function friendStatus(friend: InvitePlayersUser): InvitePlayersUserStatus {
	return friend.status ?? 'available'
}

function friendStatusSort(friend: InvitePlayersUser) {
	switch (friendStatus(friend)) {
		case 'available':
			return 0
		case 'requested':
			return 1
		case 'pending':
			return 2
		case 'added':
			return 3
	}
}

function syncFriendOrder(friends: InvitePlayersUser[]) {
	const nextOrder = new Map(friendOrder.value)
	let orderChanged = false
	let nextIndex = nextOrder.size

	const unorderedFriends = friends.filter((friend) => !nextOrder.has(friend.id))
	const orderedFriends = unorderedFriends
		.map((friend, index) => ({ friend, index }))
		.sort((a, b) => {
			const statusSort = friendStatusSort(a.friend) - friendStatusSort(b.friend)
			return statusSort || a.index - b.index
		})

	for (const { friend } of orderedFriends) {
		nextOrder.set(friend.id, nextIndex)
		nextIndex += 1
		orderChanged = true
	}

	if (orderChanged) {
		friendOrder.value = nextOrder
	}
}

function normalizeInviteKey(value: string) {
	return value.trim().toLowerCase()
}

function findSearchUser(value: string) {
	const normalizedValue = normalizeInviteKey(value)
	return searchableUsers.value.find(
		(user) =>
			normalizeInviteKey(user.username) === normalizedValue ||
			normalizeInviteKey(user.id) === normalizedValue ||
			(!!user.email && normalizeInviteKey(user.email) === normalizedValue),
	)
}

function handleSearchInput(value: string) {
	searchTarget.value = value
	selectedSearchUser.value = null
	remoteSearchUsers.value = []
	searchLookupRequestId.value += 1

	if (normalizedSearchTarget.value.length < searchMinimumLength) {
		searchLookupStatus.value = 'idle'
		return
	}

	if (!usesRemoteSearch.value) {
		searchLookupStatus.value = 'loaded'
		return
	}

	searchLookupStatus.value = 'loading'
	void searchTargetUsers(normalizedSearchTarget.value, searchLookupRequestId.value)
}

function handleSearchSelect(option: ComboboxOption<string>) {
	searchTarget.value = option.value
	selectedSearchUser.value = findSearchUser(option.value) ?? null
	searchLookupStatus.value = 'loaded'
}

function resetSearch() {
	searchTarget.value = ''
	searchInputKey.value += 1
	selectedSearchUser.value = null
	remoteSearchUsers.value = []
	searchLookupStatus.value = 'idle'
	searchLookupRequestId.value += 1
}

function inviteSearchTarget() {
	if (!canInviteSearchTarget.value) return
	const user = matchedSearchUser.value
	if (!user) return

	emit('invite', {
		user: {
			id: user.id,
			username: user.username,
			avatarUrl: user.avatarUrl,
		},
		source: 'search',
	})
	resetSearch()
}

function inviteFriend(friend: InvitePlayersUser) {
	emit('invite', {
		user: friend,
		source: 'friend',
	})
}

function cancelInvite(friend: InvitePlayersUser) {
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

watch(() => props.friends, syncFriendOrder, { immediate: true })

defineExpose({ show, hide })
</script>
