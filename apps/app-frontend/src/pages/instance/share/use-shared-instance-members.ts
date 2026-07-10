import type { InvitePlayersUser } from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type Ref, ref, watch } from 'vue'

import { get_user_many } from '@/helpers/cache.js'
import {
	get_shared_instance_users,
	invite_shared_instance_users,
	remove_shared_instance_users,
	type SharedInstanceUser,
	type SharedInstanceUsers,
} from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'

import { normalizeInviteKey, type ShareRow, toError } from './shared-instance-share-types'

type MutationContext = {
	queryKey: readonly ['sharedInstanceUsers', string]
	previousRows?: ShareRow[]
	previousPendingRows: Record<string, ShareRow>
}

export function useSharedInstanceMembers(options: {
	instance: Ref<GameInstance>
	currentUserId: Ref<string | null>
	isSignedIn: Ref<boolean>
	actionsLocked: Ref<boolean>
	onError: (error: unknown) => void
	onHydrationError: (error: Error) => void
}) {
	const queryClient = useQueryClient()
	const queryKey = computed(
		() => ['sharedInstanceUsers', options.instance.value.id] as const,
	)
	const pendingRows = ref<Record<string, ShareRow>>(loadPendingRows(options.instance.value.id))

	async function usersToRows(users: SharedInstanceUsers): Promise<ShareRow[]> {
		const excludedIds = new Set(
			[options.instance.value.shared_instance?.manager_id, options.currentUserId.value].filter(
				(id): id is string => !!id,
			),
		)
		const usersToDisplay = userEntries(users).filter((user) => !excludedIds.has(user.id))
		if (usersToDisplay.length === 0) return []

		const profiles = (await get_user_many(usersToDisplay.map((user) => user.id))) as Array<{
			id: string
			username?: string
			avatar_url?: string | null
		}>

		return usersToDisplay.map((user) => {
			const profile = profiles.find((candidate) => candidate.id === user.id)
			const joinedAt = parseDate(user.joined_at)
			return {
				id: user.id,
				username: profile?.username ?? user.id,
				avatarUrl: profile?.avatar_url ?? undefined,
				lastPlayedAt: parseDate(user.last_played),
				joinedAt,
				method: user.join_type === 'link' ? 'link' : 'direct',
				pending: !joinedAt,
			}
		})
	}

	async function loadRows() {
		if (options.actionsLocked.value) return []
		const loadedRows = await usersToRows(
			await get_shared_instance_users(options.instance.value.id),
		)
		removePendingRows(loadedRows.map((row) => row.id))
		return loadedRows
	}

	const query = useQuery({
		queryKey,
		queryFn: loadRows,
		enabled: () =>
			options.isSignedIn.value && !!options.instance.value.id && !options.actionsLocked.value,
		staleTime: Infinity,
	})
	const remoteRows = computed(
		() => query.data.value ?? queryClient.getQueryData<ShareRow[]>(queryKey.value) ?? [],
	)
	const rows = computed(() => {
		if (options.actionsLocked.value) return remoteRows.value
		const remoteIds = new Set(remoteRows.value.map((row) => normalizeInviteKey(row.id)))
		return [
			...Object.values(pendingRows.value).filter(
				(row) => !remoteIds.has(normalizeInviteKey(row.id)),
			),
			...remoteRows.value,
		]
	})

	const inviteMutation = useMutation({
		mutationFn: async (user: InvitePlayersUser) => {
			if (options.actionsLocked.value) return
			return await invite_shared_instance_users(options.instance.value.id, [user.id])
		},
		onMutate: async (user): Promise<MutationContext> => {
			const currentKey = queryKey.value
			await queryClient.cancelQueries({ queryKey: currentKey })
			const context = {
				queryKey: currentKey,
				previousRows: queryClient.getQueryData<ShareRow[]>(currentKey),
				previousPendingRows: pendingRows.value,
			}
			setPendingRow(user)
			return context
		},
		onError: (error, user, context) => {
			removePendingRow(user.id)
			restore(context)
			options.onError(error)
		},
		onSuccess: async (users, user) => {
			try {
				if (users) {
					queryClient.setQueryData<ShareRow[]>(queryKey.value, await usersToRows(users))
				} else {
					upsert(inviteUserToRow(user))
				}
			} catch (error) {
				options.onHydrationError(toError(error))
				upsert(inviteUserToRow(user))
				await queryClient.invalidateQueries({ queryKey: queryKey.value })
			}
		},
	})

	const removeMutation = useMutation({
		mutationFn: async (id: string) => {
			if (options.actionsLocked.value) {
				return { user_ids: [], users: [], tokens: 0 } satisfies SharedInstanceUsers
			}
			return await remove_shared_instance_users(options.instance.value.id, [id])
		},
		onMutate: async (id): Promise<MutationContext> => {
			const currentKey = queryKey.value
			await queryClient.cancelQueries({ queryKey: currentKey })
			const context = {
				queryKey: currentKey,
				previousRows: queryClient.getQueryData<ShareRow[]>(currentKey),
				previousPendingRows: pendingRows.value,
			}
			queryClient.setQueryData<ShareRow[]>(currentKey, (currentRows = []) =>
				currentRows.filter((row) => normalizeInviteKey(row.id) !== normalizeInviteKey(id)),
			)
			removePendingRow(id)
			return context
		},
		onError: (error, _id, context) => {
			restore(context)
			options.onError(error)
		},
		onSuccess: async (users) => {
			try {
				queryClient.setQueryData<ShareRow[]>(queryKey.value, await usersToRows(users))
			} catch (error) {
				options.onHydrationError(toError(error))
				await queryClient.invalidateQueries({ queryKey: queryKey.value })
			}
		},
	})

	function find(id: string, username: string) {
		const normalizedId = normalizeInviteKey(id)
		const normalizedUsername = normalizeInviteKey(username)
		return rows.value.find(
			(row) =>
				normalizeInviteKey(row.id) === normalizedId ||
				normalizeInviteKey(row.username) === normalizedUsername,
		)
	}

	function invite(user: InvitePlayersUser) {
		if (!options.actionsLocked.value && !find(user.id, user.username)) inviteMutation.mutate(user)
	}

	function remove(id: string) {
		if (!options.actionsLocked.value) removeMutation.mutate(id)
	}

	function setPendingRow(user: InvitePlayersUser) {
		pendingRows.value = { ...pendingRows.value, [user.id]: inviteUserToRow(user) }
		savePendingRows()
	}

	function removePendingRow(id: string) {
		const normalizedId = normalizeInviteKey(id)
		pendingRows.value = Object.fromEntries(
			Object.entries(pendingRows.value).filter(
				([pendingId]) => normalizeInviteKey(pendingId) !== normalizedId,
			),
		)
		savePendingRows()
	}

	function removePendingRows(ids: string[]) {
		if (ids.length === 0) return
		const normalizedIds = new Set(ids.map(normalizeInviteKey))
		pendingRows.value = Object.fromEntries(
			Object.entries(pendingRows.value).filter(
				([id]) => !normalizedIds.has(normalizeInviteKey(id)),
			),
		)
		savePendingRows()
	}

	function upsert(row: ShareRow) {
		queryClient.setQueryData<ShareRow[]>(queryKey.value, (currentRows = []) => [
			row,
			...currentRows.filter(
				(existing) => normalizeInviteKey(existing.id) !== normalizeInviteKey(row.id),
			),
		])
	}

	function restore(context?: MutationContext) {
		if (!context) return
		if (context.previousRows === undefined) {
			queryClient.removeQueries({ queryKey: context.queryKey, exact: true })
		} else {
			queryClient.setQueryData(context.queryKey, context.previousRows)
		}
		pendingRows.value = context.previousPendingRows
		savePendingRows()
	}

	function savePendingRows() {
		if (typeof localStorage === 'undefined') return
		const key = pendingRowsStorageKey(options.instance.value.id)
		const storedRows = Object.values(pendingRows.value)
		if (storedRows.length === 0) localStorage.removeItem(key)
		else localStorage.setItem(key, JSON.stringify(storedRows))
	}

	watch(
		() => options.instance.value.id,
		(instanceId) => {
			pendingRows.value = loadPendingRows(instanceId)
		},
	)

	return { rows, query, find, invite, remove }
}

function userEntries(users: SharedInstanceUsers): SharedInstanceUser[] {
	if (users.users?.length > 0) return users.users
	return users.user_ids.map((id) => ({
		id,
		joined_at: null,
		join_type: 'invite',
		last_played: null,
	}))
}

function parseDate(value?: string | null) {
	if (!value) return null
	const date = new Date(value)
	return Number.isNaN(date.getTime()) ? null : date
}

function inviteUserToRow(user: InvitePlayersUser): ShareRow {
	return {
		id: user.id,
		username: user.username,
		avatarUrl: user.avatarUrl ?? undefined,
		lastPlayedAt: null,
		joinedAt: null,
		method: 'direct',
		pending: true,
	}
}

function loadPendingRows(instanceId: string): Record<string, ShareRow> {
	if (typeof localStorage === 'undefined') return {}
	try {
		const stored = localStorage.getItem(pendingRowsStorageKey(instanceId))
		if (!stored) return {}
		const rows = JSON.parse(stored) as ShareRow[]
		return Object.fromEntries(
			rows.map((row) => [
				row.id,
				{ ...row, lastPlayedAt: null, joinedAt: null, pending: true } satisfies ShareRow,
			]),
		)
	} catch {
		return {}
	}
}

function pendingRowsStorageKey(instanceId: string) {
	return `modrinth:shared-instance-pending-users:${instanceId}`
}
