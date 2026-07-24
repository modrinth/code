import type { InvitePlayersUser } from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type Ref } from 'vue'

import { get_user_many } from '@/helpers/cache.js'
import {
	get_shared_instance_users,
	invite_shared_instance_users,
	remove_shared_instance_users,
	type SharedInstanceUser,
	type SharedInstanceUsers,
} from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'

import { normalizeInviteKey, type ShareRow } from './shared-instance-share-types'

type MembersQueryKey = readonly ['sharedInstanceUsers', string]

type OptimisticChange = {
	queryKey: MembersQueryKey
	userId: string
	previousRow?: ShareRow
	previousIndex: number
}

type InviteVariables = {
	user: InvitePlayersUser
	change: OptimisticChange
}

type RemoveVariables = {
	id: string
	hasPendingRecipients: boolean
	change: OptimisticChange
}

export function useSharedInstanceMembers(options: {
	instance: Ref<GameInstance>
	currentUserId: Ref<string | null>
	isSignedIn: Ref<boolean>
	actionsLocked: Ref<boolean>
	onError: (error: unknown) => void
}) {
	const queryClient = useQueryClient()
	const queryKey = computed(() => ['sharedInstanceUsers', options.instance.value.id] as const)
	const invitingUserIds = new Set<string>()
	const removingUserIds = new Set<string>()

	const query = useQuery({
		queryKey,
		queryFn: ({ queryKey }) => fetchRows(queryKey),
		enabled: () =>
			options.isSignedIn.value && !!options.instance.value.id && !options.actionsLocked.value,
		staleTime: Infinity,
		refetchOnMount: 'always',
		refetchOnReconnect: false,
		refetchOnWindowFocus: false,
	})
	const rows = computed(() => query.data.value ?? [])

	const inviteMutation = useMutation({
		mutationFn: ({ user, change }: InviteVariables) =>
			invite_shared_instance_users(change.queryKey[1], [user.id]),
		onError: (error, { change }) => {
			rollback(change)
			options.onError(error)
		},
		onSettled: (_data, _error, { user }) => {
			invitingUserIds.delete(normalizeInviteKey(user.id))
		},
	})

	const removeMutation = useMutation({
		mutationFn: ({ id, hasPendingRecipients, change }: RemoveVariables) =>
			remove_shared_instance_users(change.queryKey[1], [id], hasPendingRecipients),
		onError: (error, { change }) => {
			rollback(change)
			options.onError(error)
		},
		onSettled: (_data, _error, { id }) => {
			removingUserIds.delete(normalizeInviteKey(id))
		},
	})

	async function fetchRows(activeQueryKey: MembersQueryKey) {
		const users = await get_shared_instance_users(activeQueryKey[1])
		const loadedRows = await usersToRows(users)
		const currentRows = queryClient.getQueryData<ShareRow[]>(activeQueryKey) ?? []
		return preserveRowOrder(loadedRows, currentRows)
	}

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
		const normalizedId = normalizeInviteKey(user.id)
		if (
			options.actionsLocked.value ||
			invitingUserIds.has(normalizedId) ||
			find(user.id, user.username)
		) {
			return
		}

		invitingUserIds.add(normalizedId)
		const change = beginOptimisticChange(user.id)
		updateRows(change.queryKey, (currentRows) => [...currentRows, inviteUserToRow(user)])
		inviteMutation.mutate({ user, change })
	}

	function remove(id: string) {
		const normalizedId = normalizeInviteKey(id)
		if (options.actionsLocked.value || removingUserIds.has(normalizedId)) return

		removingUserIds.add(normalizedId)
		const hasPendingRecipients = rows.value.some(
			(row) => row.pending && normalizeInviteKey(row.id) !== normalizedId,
		)
		const change = beginOptimisticChange(id)
		updateRows(change.queryKey, (currentRows) =>
			currentRows.filter((row) => normalizeInviteKey(row.id) !== normalizedId),
		)
		removeMutation.mutate({ id, hasPendingRecipients, change })
	}

	function beginOptimisticChange(userId: string): OptimisticChange {
		const activeQueryKey = queryKey.value
		void queryClient.cancelQueries({ queryKey: activeQueryKey, exact: true }, { revert: false })

		const currentRows = queryClient.getQueryData<ShareRow[]>(activeQueryKey) ?? []
		const previousIndex = currentRows.findIndex(
			(row) => normalizeInviteKey(row.id) === normalizeInviteKey(userId),
		)
		return {
			queryKey: activeQueryKey,
			userId,
			previousRow: previousIndex === -1 ? undefined : currentRows[previousIndex],
			previousIndex,
		}
	}

	function rollback(change: OptimisticChange) {
		const normalizedId = normalizeInviteKey(change.userId)
		updateRows(change.queryKey, (currentRows) => {
			const rowsWithoutUser = currentRows.filter(
				(row) => normalizeInviteKey(row.id) !== normalizedId,
			)
			if (!change.previousRow) return rowsWithoutUser

			const previousIndex = Math.min(change.previousIndex, rowsWithoutUser.length)
			return [
				...rowsWithoutUser.slice(0, previousIndex),
				change.previousRow,
				...rowsWithoutUser.slice(previousIndex),
			]
		})
	}

	function updateRows(activeQueryKey: MembersQueryKey, update: (rows: ShareRow[]) => ShareRow[]) {
		queryClient.setQueryData<ShareRow[]>(activeQueryKey, (currentRows = []) => update(currentRows))
	}

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

function preserveRowOrder(rows: ShareRow[], previousRows: ShareRow[]) {
	const rowsById = new Map(rows.map((row) => [normalizeInviteKey(row.id), row]))
	const orderedRows = previousRows.flatMap((previousRow) => {
		const id = normalizeInviteKey(previousRow.id)
		const row = rowsById.get(id)
		if (!row) return []
		rowsById.delete(id)
		return [row]
	})
	return [...orderedRows, ...rowsById.values()]
}
