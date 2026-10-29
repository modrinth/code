import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type Ref } from 'vue'

import type {
	InviteLinkSettings,
	InvitePlayersUser,
} from '#ui/components/sharing/invite-players-modal/types'
import type { InvitedPlayerRow } from '#ui/components/sharing/invited-players-table/types'
import { sharedInstanceKeys } from '#ui/composables/server-queries'
import {
	ensureServerInviteLink,
	findActiveInviteLink,
	INVITE_LINK_MAX_AGE_SECONDS,
	type ServerPlayerMembers,
	sharedInstanceInvitesQueryOptions,
	useServerPlayerMembers,
} from '#ui/composables/server-sharing'
import { injectAuth, injectModrinthClient } from '#ui/providers'

export function useServerPlayers(instanceId: Ref<string | null>, canManage: Ref<boolean>) {
	const client = injectModrinthClient()
	const auth = injectAuth()
	const queryClient = useQueryClient()
	const userId = computed(() => auth.user.value?.id)
	const memberKey = (id: string) => sharedInstanceKeys.players(id, userId.value)
	const members = useServerPlayerMembers(instanceId)
	const rows = computed(() => members.data.value?.rows ?? [])
	const remaining = computed(() => members.data.value?.remaining ?? 0)
	const friends = useQuery({
		queryKey: computed(() => sharedInstanceKeys.friends(userId.value)),
		enabled: computed(() => !!userId.value && canManage.value),
		queryFn: async () => {
			const currentUserId = userId.value
			const relationships = await client.labrinth.friends_v3.list()
			const ids = relationships
				.filter((friend) => friend.accepted)
				.map((friend) => (friend.id === currentUserId ? friend.friend_id : friend.id))
			return ids.length ? client.labrinth.users_v2.getMultiple(ids) : []
		},
	})
	const candidates = computed<InvitePlayersUser[]>(() => {
		const candidates = new Map<string, InvitePlayersUser>()
		for (const friend of friends.data.value ?? []) {
			candidates.set(friend.id, {
				id: friend.id,
				username: friend.username,
				avatarUrl: friend.avatar_url,
				status: 'available',
			})
		}
		for (const row of rows.value) {
			candidates.set(row.id, { ...row, status: row.pending ? 'pending' : 'added' })
		}
		return [...candidates.values()]
	})
	const links = useQuery(
		computed(() => ({
			...sharedInstanceInvitesQueryOptions(client, instanceId.value ?? '', userId.value),
			enabled: !!instanceId.value && !!userId.value && canManage.value,
		})),
	)
	const link = computed(() => findActiveInviteLink(links.data.value))

	const membershipMutation = useMutation({
		mutationFn: async ({
			id,
			userId,
			remove,
		}: {
			id: string
			userId: string
			remove: boolean
			user?: InvitePlayersUser
		}) => {
			if (!canManage.value) throw new Error('You do not have permission to manage players.')
			if (remove) return client.sharedinstances.instances_v1.removeUsers(id, [userId])
			const result = await client.sharedinstances.instances_v1.inviteUsers(id, [userId])
			if (result.failed.includes(userId))
				throw new Error('This player’s privacy settings do not allow this invitation.')
		},
		onMutate: async ({ id, userId, remove, user }) => {
			const queryKey = memberKey(id)
			await queryClient.cancelQueries({ queryKey, exact: true })
			const previous = queryClient.getQueryData<ServerPlayerMembers>(queryKey)
			if (previous) {
				const exists = previous.rows.some((row) => row.id === userId)
				const addedRow: InvitedPlayerRow = {
					id: userId,
					username: user?.username ?? userId,
					avatarUrl: user?.avatarUrl ?? undefined,
					joinedAt: null,
					lastPlayedAt: null,
					pending: true,
					method: 'direct',
				}
				queryClient.setQueryData<ServerPlayerMembers>(queryKey, {
					rows: remove
						? previous.rows.filter((row) => row.id !== userId)
						: exists
							? previous.rows
							: [...previous.rows, addedRow],
					remaining: Math.max(0, previous.remaining + (remove ? Number(exists) : -Number(!exists))),
				})
			}
			return { queryKey, previous }
		},
		onError: (_error, _variables, context) => {
			if (context?.previous) queryClient.setQueryData(context.queryKey, context.previous)
		},
		onSettled: (_data, _error, { id }) =>
			Promise.all([
				queryClient.invalidateQueries({ queryKey: sharedInstanceKeys.players(id) }),
				queryClient.invalidateQueries({ queryKey: sharedInstanceKeys.instance(id) }),
			]),
	})
	const linkMutation = useMutation({
		mutationFn: async ({
			id,
			settings,
			replaceId,
		}: {
			id: string
			settings: InviteLinkSettings
			replaceId?: string
		}) => {
			if (!canManage.value) throw new Error('You do not have permission to manage invite links.')
			const requestedAge = Math.floor((settings.expiresAt.getTime() - Date.now()) / 1000)
			const created = await client.sharedinstances.invites_v1.create(id, {
				max_age: Math.max(1, Math.min(INVITE_LINK_MAX_AGE_SECONDS, requestedAge)),
				max_uses: Math.max(1, Math.min(settings.maxUses, remaining.value)),
			})
			if (replaceId) {
				try {
					await client.sharedinstances.invites_v1.delete(id, replaceId)
				} catch (error) {
					await client.sharedinstances.invites_v1.delete(id, created.id)
					throw error
				}
			}
		},
		onSettled: (_data, _error, { id }) =>
			queryClient.invalidateQueries({ queryKey: sharedInstanceKeys.invites(id) }),
	})

	async function ensureLink(id: string, remainingSlots?: number) {
		await ensureServerInviteLink(client, id, remainingSlots)
		await queryClient.invalidateQueries({ queryKey: sharedInstanceKeys.invites(id) })
	}

	async function search(query: string) {
		const users = await queryClient.fetchQuery({
			queryKey: ['users', 'search', query],
			queryFn: () => client.labrinth.users_v3.search(query),
			staleTime: 30_000,
		})
		return users
			.filter(
				(user) =>
					user.id !== userId.value &&
					!candidates.value.some((candidate) => candidate.id === user.id),
			)
			.map((user) => ({ id: user.id, username: user.username, avatarUrl: user.avatar_url }))
	}

	return {
		members,
		rows,
		remaining,
		candidates,
		link,
		links,
		friends,
		membershipMutation,
		linkMutation,
		ensureLink,
		search,
	}
}
