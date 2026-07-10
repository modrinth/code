import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, onUnmounted, type MaybeRefOrGetter, toValue } from 'vue'

import { toError } from '@/helpers/errors'
import { friend_listener } from '@/helpers/events.js'
import {
	acceptCachedFriend,
	add_friend,
	createPendingFriend,
	type FriendCacheUser,
	type FriendWithUserData,
	friendsQueryKey,
	getFriendsWithUserData,
	getFriendUserId,
	matchesFriend,
	remove_friend,
	removeCachedFriend,
	upsertCachedFriend,
} from '@/helpers/friends'
import type { ModrinthCredentials } from '@/helpers/mr_auth'

type FriendsMutationContext = {
	queryKey: ReturnType<typeof friendsQueryKey>
	previousFriends?: FriendWithUserData[]
}

type AddFriendMutationVariables = {
	userId: string
	user: FriendCacheUser
	acceptExisting: boolean
}

type RemoveFriendMutationVariables = {
	userId: string
	user: FriendWithUserData
}

export function useFriends(options: {
	currentUserId: MaybeRefOrGetter<string | null | undefined>
	getCredentials: () => ModrinthCredentials | null | Promise<ModrinthCredentials | null>
	enabled?: MaybeRefOrGetter<boolean>
	onError?: (error: Error) => void
}) {
	const queryClient = useQueryClient()
	const queryKey = computed(() => friendsQueryKey(toValue(options.currentUserId)))
	const query = useQuery({
		queryKey,
		queryFn: async () => getFriendsWithUserData(await options.getCredentials()),
		enabled: () => !!toValue(options.currentUserId) && toValue(options.enabled ?? true),
		staleTime: 30_000,
	})
	const friends = computed(() => query.data.value ?? [])

	function restore(context?: FriendsMutationContext) {
		if (!context) return
		if (context.previousFriends === undefined) {
			queryClient.removeQueries({ queryKey: context.queryKey, exact: true })
			return
		}
		queryClient.setQueryData(context.queryKey, context.previousFriends)
	}

	const addMutation = useMutation({
		mutationFn: ({ userId }: AddFriendMutationVariables) => add_friend(userId),
		onMutate: async ({ user, acceptExisting }): Promise<FriendsMutationContext> => {
			const activeQueryKey = queryKey.value
			await queryClient.cancelQueries({ queryKey: activeQueryKey })
			const previousFriends = queryClient.getQueryData<FriendWithUserData[]>(activeQueryKey)
			const currentUserId = toValue(options.currentUserId)
			queryClient.setQueryData<FriendWithUserData[]>(activeQueryKey, (cachedFriends = []) =>
				acceptExisting
					? acceptCachedFriend(cachedFriends, user.id, user.username, currentUserId)
					: upsertCachedFriend(
							cachedFriends,
							createPendingFriend(user, currentUserId),
							currentUserId,
						),
			)
			return { queryKey: activeQueryKey, previousFriends }
		},
		onError: (error, _variables, context) => {
			restore(context)
			options.onError?.(toError(error))
		},
		onSettled: (_data, _error, _variables, context) => {
			void queryClient.invalidateQueries({ queryKey: context?.queryKey ?? queryKey.value })
		},
	})

	const removeMutation = useMutation({
		mutationFn: ({ userId }: RemoveFriendMutationVariables) => remove_friend(userId),
		onMutate: async ({ user, userId }): Promise<FriendsMutationContext> => {
			const activeQueryKey = queryKey.value
			await queryClient.cancelQueries({ queryKey: activeQueryKey })
			const previousFriends = queryClient.getQueryData<FriendWithUserData[]>(activeQueryKey)
			queryClient.setQueryData<FriendWithUserData[]>(activeQueryKey, (cachedFriends = []) =>
				removeCachedFriend(
					cachedFriends,
					userId,
					user.username,
					toValue(options.currentUserId),
				),
			)
			return { queryKey: activeQueryKey, previousFriends }
		},
		onError: (error, _variables, context) => {
			restore(context)
			options.onError?.(toError(error))
		},
		onSettled: (_data, _error, _variables, context) => {
			void queryClient.invalidateQueries({ queryKey: context?.queryKey ?? queryKey.value })
		},
	})

	function requestFriend(user: FriendCacheUser, acceptExisting = false) {
		addMutation.mutate({ userId: user.id, user, acceptExisting })
	}

	function acceptFriend(friend: FriendWithUserData) {
		const userId = getFriendUserId(friend, toValue(options.currentUserId))
		requestFriend(
			{ id: userId, username: friend.username, avatarUrl: friend.avatar },
			true,
		)
	}

	function removeFriend(friend: FriendWithUserData) {
		const userId = getFriendUserId(friend, toValue(options.currentUserId))
		removeMutation.mutate({ userId, user: friend })
	}

	function findFriend(id: string, username: string) {
		return friends.value.find((friend) =>
			matchesFriend(friend, id, username, toValue(options.currentUserId)),
		)
	}

	let unlisten: (() => void) | undefined
	void friend_listener(() => {
		void queryClient.invalidateQueries({ queryKey: queryKey.value })
	}).then((listener) => {
		unlisten = listener
	})
	onUnmounted(() => unlisten?.())

	return {
		query,
		queryKey,
		friends,
		loading: computed(() => !!toValue(options.currentUserId) && query.isLoading.value),
		requestFriend,
		acceptFriend,
		removeFriend,
		findFriend,
	}
}
