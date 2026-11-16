import { useQuery } from '@tanstack/vue-query'
import { computed, type Ref } from 'vue'

import type { InvitedPlayerRow } from '#ui/components/sharing/invited-players-table/types'
import { sharedInstanceKeys } from '#ui/composables/server-queries'
import { injectAuth, injectModrinthClient } from '#ui/providers'

import { remainingPlayerSlots } from './invite-link'

export type ServerPlayerMembers = { rows: InvitedPlayerRow[]; remaining: number }

export function useServerPlayerMembers(instanceId: Ref<string | null>) {
	const client = injectModrinthClient()
	const auth = injectAuth()
	const userId = computed(() => auth.user.value?.id)
	return useQuery({
		queryKey: computed(() => sharedInstanceKeys.players(instanceId.value ?? '', userId.value)),
		enabled: computed(() => !!instanceId.value && !!userId.value),
		queryFn: async (): Promise<ServerPlayerMembers> => {
			const response = await client.sharedinstances.instances_v1.getUsers(instanceId.value!)
			const users = response.users.length
				? await client.labrinth.users_v2.getMultiple(response.users.map((user) => user.id))
				: []
			const rows = response.users
				.filter((user) => user.join_type !== 'owner')
				.map((member): InvitedPlayerRow => {
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
			return { rows, remaining: remainingPlayerSlots(response.users.length) }
		},
		refetchInterval: 30_000,
	})
}

export function useSharedInstanceInstalledOnce(instanceId: Ref<string | null>) {
	const client = injectModrinthClient()
	const auth = injectAuth()
	const userId = computed(() => auth.user.value?.id)
	const instance = useQuery({
		queryKey: computed(() => sharedInstanceKeys.instance(instanceId.value ?? '', userId.value)),
		enabled: computed(() => !!instanceId.value && !!userId.value),
		queryFn: () => client.sharedinstances.instances_v1.get(instanceId.value!),
		refetchInterval: 30_000,
	})
	return computed(() => instance.data.value?.installed_once ?? false)
}

export function sharedInstanceMrpackFilename(name: string) {
	return `${name.replace(/[\\/:*?"<>|]/g, '_')}.mrpack`
}
