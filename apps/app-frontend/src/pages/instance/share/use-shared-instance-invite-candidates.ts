import {
	injectNotificationManager,
	type InvitePlayersSearchUser,
	type InvitePlayersUser,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, onUnmounted, type Ref } from 'vue'

import { friend_listener } from '@/helpers/events.js'
import {
	add_friend,
	friendsQueryKey,
	getFriendsWithUserData,
	getFriendUserId,
} from '@/helpers/friends.ts'
import { get as getCredentials } from '@/helpers/mr_auth.ts'
import { search_user } from '@/helpers/users.ts'

import { normalizeInviteKey, type ShareRow, toError } from './shared-instance-share-types'

export function useSharedInstanceInviteCandidates(options: {
	rows: Ref<ShareRow[]>
	currentUserId: Ref<string | null>
	isSignedIn: Ref<boolean>
	actionsLocked: Ref<boolean>
}) {
	const queryClient = useQueryClient()
	const { handleError } = injectNotificationManager()
	const queryKey = computed(() => friendsQueryKey(options.currentUserId.value))
	const friendsQuery = useQuery({
		queryKey,
		queryFn: async () => getFriendsWithUserData(await getCredentials()),
		enabled: () =>
			options.isSignedIn.value && !!options.currentUserId.value && !options.actionsLocked.value,
		staleTime: 30_000,
	})
	const friends = computed(() => friendsQuery.data.value ?? [])
	const invitedRows = computed(() => {
		const invited = new Map<string, ShareRow>()
		for (const row of options.rows.value) {
			invited.set(normalizeInviteKey(row.id), row)
			invited.set(normalizeInviteKey(row.username), row)
		}
		return invited
	})
	const inviteFriends = computed<InvitePlayersUser[]>(() =>
		friends.value
			.filter((friend) => friend.username && friend.accepted)
			.sort((a, b) => Number(b.online) - Number(a.online))
			.map((friend) => {
				const id = getFriendUserId(friend, options.currentUserId.value)
				const invited =
					invitedRows.value.get(normalizeInviteKey(id)) ??
					invitedRows.value.get(normalizeInviteKey(friend.username))
				return {
					id,
					username: friend.username,
					avatarUrl: friend.avatar,
					online: friend.online,
					status: invited ? (invited.pending ? 'pending' : 'added') : 'available',
				}
			}),
	)
	const candidateKeys = computed(() => {
		const keys = new Set<string>()
		for (const friend of inviteFriends.value) {
			keys.add(normalizeInviteKey(friend.id))
			keys.add(normalizeInviteKey(friend.username))
		}
		return keys
	})

	const friendRequestMutation = useMutation({
		mutationFn: (user: InvitePlayersUser) => {
			if (!options.actionsLocked.value) return add_friend(user.id)
		},
		onError: (error) => handleError(toError(error)),
	})

	async function search(query: string): Promise<InvitePlayersSearchUser[]> {
		if (options.actionsLocked.value) return []
		const credentials = await getCredentials()
		const ownUserId = options.currentUserId.value ?? credentials?.user_id ?? null
		return (await search_user(query))
			.filter((user) => user.id !== ownUserId)
			.filter((user) => {
				const id = normalizeInviteKey(user.id)
				const username = normalizeInviteKey(user.username)
				return (
					!candidateKeys.value.has(id) &&
					!candidateKeys.value.has(username) &&
					!invitedRows.value.has(id) &&
					!invitedRows.value.has(username)
				)
			})
			.map((user) => ({
				id: user.id,
				username: user.username,
				avatarUrl: user.avatar_url || undefined,
			}))
	}

	async function requestFriend(user: InvitePlayersUser) {
		if (options.actionsLocked.value) return
		const credentials = await getCredentials()
		const ownUserId = options.currentUserId.value ?? credentials?.user_id ?? null
		if (ownUserId && normalizeInviteKey(user.id) === normalizeInviteKey(ownUserId)) return
		const existing = friends.value.find((friend) => {
			const friendId = getFriendUserId(friend, options.currentUserId.value)
			return (
				normalizeInviteKey(friendId) === normalizeInviteKey(user.id) ||
				normalizeInviteKey(friend.username) === normalizeInviteKey(user.username)
			)
		})
		if (!existing) friendRequestMutation.mutate(user)
	}

	const unlisten = await friend_listener(() => {
		void queryClient.invalidateQueries({ queryKey: queryKey.value })
	})
	onUnmounted(unlisten)

	return { inviteFriends, search, requestFriend }
}
