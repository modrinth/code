import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type Ref } from 'vue'

import type { InviteLinkSettings, InvitePlayersUser } from '#ui/components/sharing'
import { injectAuth, injectModrinthClient } from '#ui/providers'

import type { ServerPlayerRow } from './types'

export function useServerPlayers(instanceId: Ref<string | null>, canManage: Ref<boolean>) {
	const client = injectModrinthClient()
	const auth = injectAuth()
	const queryClient = useQueryClient()
	const userId = computed(() => auth.user.value?.id)
	const memberKey = (id: string) => ['shared-instances', id, 'players', userId.value] as const
	const inviteKey = (id: string) => ['shared-instances', id, 'invites', userId.value] as const
	const members = useQuery({
		queryKey: computed(() => memberKey(instanceId.value ?? '')),
		enabled: computed(() => !!instanceId.value && !!userId.value),
		queryFn: async () => {
			const response = await client.sharedinstances.instances_v1.getUsers(instanceId.value!)
			const users = response.users.length
				? await client.labrinth.users_v2.getMultiple(response.users.map((user) => user.id))
				: []
			const rows: ServerPlayerRow[] = response.users.filter((user) => user.join_type !== 'owner').map((member) => {
				const user = users.find((user) => user.id === member.id)
				return {
					id: member.id,
					username: user?.username ?? member.id,
					avatarUrl: user?.avatar_url ?? undefined,
					joinedAt: member.joined_at ? new Date(member.joined_at) : null,
					lastPlayedAt: member.last_played ? new Date(member.last_played) : null,
					pending: !member.joined_at,
					method: member.join_type === 'link' ? 'link' : 'direct',
				}
			})
			return { rows, remaining: Math.max(0, 50 - response.users.length - response.tokens) }
		},
		refetchInterval: 30_000,
	})
	const rows = computed(() => members.data.value?.rows ?? [])
	const remaining = computed(() => members.data.value?.remaining ?? 0)
	const friends = useQuery({
		queryKey: computed(() => ['shared-instances', 'friends', userId.value]),
		enabled: computed(() => !!userId.value && canManage.value),
		queryFn: async () => {
			const currentUserId = userId.value
			const relationships = await client.labrinth.friends_v3.list()
			const ids = relationships.filter((friend) => friend.accepted)
				.map((friend) => friend.id === currentUserId ? friend.friend_id : friend.id)
			return ids.length ? client.labrinth.users_v2.getMultiple(ids) : []
		},
	})
	const candidates = computed<InvitePlayersUser[]>(() => {
		const candidates = new Map<string, InvitePlayersUser>()
		for (const friend of friends.data.value ?? []) {
			candidates.set(friend.id, { id: friend.id, username: friend.username, avatarUrl: friend.avatar_url, status: 'available' })
		}
		for (const row of rows.value) {
			candidates.set(row.id, { ...row, status: row.pending ? 'pending' : 'added' })
		}
		return [...candidates.values()]
	})
	const links = useQuery({
		queryKey: computed(() => inviteKey(instanceId.value ?? '')),
		enabled: computed(() => !!instanceId.value && canManage.value),
		queryFn: () => client.sharedinstances.invites_v1.list(instanceId.value!),
	})
	const link = computed(() => links.data.value?.find((link) => new Date(link.expiration).getTime() > Date.now() && link.uses < link.max_uses))

	const membershipMutation = useMutation({
		mutationFn: async ({ id, userId, remove }: { id: string; userId: string; remove: boolean }) => {
			if (!canManage.value) throw new Error('You do not have permission to manage players.')
			if (remove) return client.sharedinstances.instances_v1.removeUsers(id, [userId])
			const result = await client.sharedinstances.instances_v1.inviteUsers(id, [userId])
			if (result.failed.includes(userId)) throw new Error('This player’s privacy settings do not allow this invitation.')
		},
		onSettled: (_data, _error, { id }) => queryClient.invalidateQueries({ queryKey: ['shared-instances', id, 'players'] }),
	})
	const linkMutation = useMutation({
		mutationFn: async ({ id, settings, replaceId }: { id: string; settings: InviteLinkSettings; replaceId?: string }) => {
			if (!canManage.value) throw new Error('You do not have permission to manage invite links.')
			const created = await client.sharedinstances.invites_v1.create(id, {
				max_age: Math.max(1, Math.min(604800, Math.floor((settings.expiresAt.getTime() - Date.now()) / 1000))),
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
		onSettled: (_data, _error, { id }) => queryClient.invalidateQueries({ queryKey: ['shared-instances', id, 'invites'] }),
	})

	async function ensureLink(id: string) {
		const available = await queryClient.fetchQuery({ queryKey: inviteKey(id), queryFn: () => client.sharedinstances.invites_v1.list(id), staleTime: 0 })
		if (available.some((link) => new Date(link.expiration).getTime() > Date.now() && link.uses < link.max_uses)) return
		if (remaining.value <= 0) return
		await linkMutation.mutateAsync({ id, settings: { maxUses: Math.min(10, remaining.value), expiresAt: new Date(Date.now() + 86400_000) } })
	}

	async function search(query: string) {
		const users = await queryClient.fetchQuery({
			queryKey: ['users', 'search', query],
			queryFn: () => client.labrinth.users_v3.search(query),
			staleTime: 30_000,
		})
		return users.filter((user) => user.id !== userId.value && !candidates.value.some((candidate) => candidate.id === user.id))
			.map((user) => ({ id: user.id, username: user.username, avatarUrl: user.avatar_url }))
	}

	return { members, rows, remaining, candidates, link, links, friends, membershipMutation, linkMutation, ensureLink, search }
}
