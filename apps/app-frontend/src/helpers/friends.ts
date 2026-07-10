import type { User } from '@modrinth/utils'
import { invoke } from '@tauri-apps/api/core'
import type { Dayjs } from 'dayjs'
import dayjs from 'dayjs'

import { get_user_many } from '@/helpers/cache'
import type { ModrinthCredentials } from '@/helpers/mr_auth'

export const friendsQueryKey = (userId?: string | null) => ['friends', userId ?? null] as const

export type UserStatus = {
	user_id: string
	instance_name: string | null
	last_update: string
}

export type UserFriend = {
	id: string
	friend_id: string
	accepted: boolean
	created: string
}

export async function friends(): Promise<UserFriend[]> {
	return await invoke('plugin:friends|friends')
}

export async function friend_statuses(): Promise<UserStatus[]> {
	return await invoke('plugin:friends|friend_statuses')
}

export async function add_friend(userId: string): Promise<void> {
	return await invoke('plugin:friends|add_friend', { userId })
}

export async function remove_friend(userId: string): Promise<void> {
	return await invoke('plugin:friends|remove_friend', { userId })
}

export type FriendWithUserData = {
	id: string
	friend_id: string | null
	status: string | null
	last_updated: Dayjs | null
	created: Dayjs
	username: string
	accepted: boolean
	online: boolean
	avatar: string
}

export type FriendCacheUser = {
	id: string
	username: string
	avatarUrl?: string | null
}

export async function getFriendsWithUserData(
	credentials: ModrinthCredentials | null,
): Promise<FriendWithUserData[]> {
	if (!credentials) return []

	const friendsList = await friends()
	return await transformFriends(friendsList, credentials)
}

export function createPendingFriend(
	user: FriendCacheUser,
	currentUserId?: string | null,
): FriendWithUserData {
	return {
		id: user.id,
		friend_id: currentUserId ?? null,
		status: null,
		last_updated: null,
		created: dayjs(),
		username: user.username,
		accepted: false,
		online: false,
		avatar: user.avatarUrl ?? '',
	}
}

export function getFriendUserId(
	friend: Pick<FriendWithUserData, 'id' | 'friend_id'>,
	currentUserId?: string | null,
) {
	return friend.id === currentUserId && friend.friend_id ? friend.friend_id : friend.id
}

export function matchesFriend(
	friend: FriendWithUserData,
	id: string,
	username: string,
	currentUserId?: string | null,
) {
	const friendId = getFriendUserId(friend, currentUserId)
	return (
		normalizeFriendKey(friendId) === normalizeFriendKey(id) ||
		normalizeFriendKey(friend.username) === normalizeFriendKey(username)
	)
}

export function upsertCachedFriend(
	friends: FriendWithUserData[],
	friend: FriendWithUserData,
	currentUserId?: string | null,
) {
	const existingFriend = friends.find((cachedFriend) =>
		matchesFriend(cachedFriend, friend.id, friend.username, currentUserId),
	)

	if (!existingFriend) return [friend, ...friends]

	return friends.map((cachedFriend) =>
		matchesFriend(cachedFriend, friend.id, friend.username, currentUserId)
			? {
					...cachedFriend,
					...friend,
					id: cachedFriend.id,
					friend_id: cachedFriend.friend_id,
				}
			: cachedFriend,
	)
}

export function acceptCachedFriend(
	friends: FriendWithUserData[],
	id: string,
	username: string,
	currentUserId?: string | null,
) {
	return friends.map((friend) =>
		matchesFriend(friend, id, username, currentUserId)
			? {
					...friend,
					accepted: true,
				}
			: friend,
	)
}

export function removeCachedFriend(
	friends: FriendWithUserData[],
	id: string,
	username: string,
	currentUserId?: string | null,
) {
	return friends.filter((friend) => !matchesFriend(friend, id, username, currentUserId))
}

export function normalizeFriendKey(value: string) {
	return value.trim().toLowerCase()
}

export async function transformFriends(
	friends: UserFriend[],
	credentials: ModrinthCredentials | null,
): Promise<FriendWithUserData[]> {
	if (friends.length === 0 || !credentials) {
		return []
	}

	const friendStatuses = await friend_statuses()
	const users = await get_user_many(
		friends.map((x) => (x.id === credentials.user_id ? x.friend_id : x.id)),
	)

	return friends.map((friend) => {
		const user = users.find((x: User) => x.id === friend.id || x.id === friend.friend_id)
		const status = friendStatuses.find(
			(x) => x.user_id === friend.id || x.user_id === friend.friend_id,
		)
		return {
			id: friend.id,
			friend_id: friend.friend_id,
			status: status?.profile_name ?? null,
			last_updated: status && status.last_update ? dayjs(status.last_update) : null,
			created: dayjs(friend.created),
			avatar: user?.avatar_url ?? '',
			username: user?.username ?? '',
			online: !!status,
			accepted: friend.accepted,
		}
	})
}
