import { useDebounceFn } from '@vueuse/core'
import { computed, type MaybeRefOrGetter, ref, toValue, watch } from 'vue'

import type { ComboboxOption } from '../../base/Combobox.vue'
import {
	type InvitePlayersInvitePayload,
	type InvitePlayersSearchUser,
	type InvitePlayersUser,
	type InvitePlayersUserStatus,
	normalizeInviteKey,
} from './types'

export function useInvitePlayersSearch(options: {
	friends: MaybeRefOrGetter<InvitePlayersUser[]>
	suggestions: MaybeRefOrGetter<InvitePlayersSearchUser[]>
	searchUsers: MaybeRefOrGetter<((query: string) => Promise<InvitePlayersSearchUser[]>) | undefined>
	canInvite: MaybeRefOrGetter<boolean>
	inviteDisabledMessage: MaybeRefOrGetter<string>
	alreadyInvitedMessage: MaybeRefOrGetter<string>
	searchingMessage: MaybeRefOrGetter<string>
	noResultsMessage: MaybeRefOrGetter<string>
	onInvite: (payload: InvitePlayersInvitePayload) => void
}) {
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
	const normalizedSearchTarget = computed(() => searchTarget.value.trim())
	const usesRemoteSearch = computed(() => !!toValue(options.searchUsers))
	const searchLookupMessage = computed(() =>
		usesRemoteSearch.value && searchLookupStatus.value !== 'loaded'
			? toValue(options.searchingMessage)
			: toValue(options.noResultsMessage),
	)
	const searchableUsers = computed(() => {
		const users = new Map<string, InvitePlayersSearchUser>()
		for (const user of [...remoteSearchUsers.value, ...toValue(options.suggestions)]) {
			users.set(normalizeInviteKey(user.id), user)
			users.set(normalizeInviteKey(user.username), user)
			if (user.email) users.set(normalizeInviteKey(user.email), user)
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
		toValue(options.friends)
			.map((friend, index) => ({
				friend,
				order: friendOrder.value.get(friend.id) ?? friendOrder.value.size + index,
			}))
			.sort((a, b) => a.order - b.order)
			.map(({ friend }) => friend),
	)

	function findSearchUser(value: string) {
		const normalizedValue = normalizeInviteKey(value)
		return searchableUsers.value.find(
			(user) =>
				normalizeInviteKey(user.username) === normalizedValue ||
				normalizeInviteKey(user.id) === normalizedValue ||
				(!!user.email && normalizeInviteKey(user.email) === normalizedValue),
		)
	}

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
		for (const friend of toValue(options.friends)) {
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
			toValue(options.canInvite) &&
			normalizedSearchTarget.value.length >= searchMinimumLength &&
			!!matchedSearchUser.value &&
			!searchTargetAlreadyInvited.value &&
			(!usesRemoteSearch.value ||
				searchLookupStatus.value === 'loaded' ||
				!!selectedSearchUser.value),
	)
	const searchInviteTooltip = computed(() => {
		if (!toValue(options.canInvite)) return toValue(options.inviteDisabledMessage)
		if (searchTargetAlreadyInvited.value) return toValue(options.alreadyInvitedMessage)
		return undefined
	})

	const searchTargetUsers = useDebounceFn(async (query: string, requestId: number) => {
		const searchUsers = toValue(options.searchUsers)
		if (!searchUsers) return
		try {
			const users = await searchUsers(query)
			if (requestId !== searchLookupRequestId.value || query !== normalizedSearchTarget.value)
				return
			remoteSearchUsers.value = users
		} catch {
			if (requestId !== searchLookupRequestId.value || query !== normalizedSearchTarget.value)
				return
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

	function syncFriendOrder(friends: InvitePlayersUser[]) {
		const nextOrder = new Map(friendOrder.value)
		let nextIndex = nextOrder.size
		const unorderedFriends = friends.filter((friend) => !nextOrder.has(friend.id))
		if (unorderedFriends.length === 0) return
		for (const friend of unorderedFriends) {
			nextOrder.set(friend.id, nextIndex)
			nextIndex += 1
		}
		friendOrder.value = nextOrder
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
		options.onInvite({
			user: {
				id: user.id,
				username: user.username,
				avatarUrl: user.avatarUrl,
			},
			source: 'search',
		})
		resetSearch()
	}

	watch(() => toValue(options.friends), syncFriendOrder, { immediate: true })

	return {
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
	}
}
