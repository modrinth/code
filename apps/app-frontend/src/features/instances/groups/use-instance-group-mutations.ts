import { injectNotificationManager } from '@modrinth/ui'
import { type QueryKey, useMutation, useQueryClient } from '@tanstack/vue-query'

import { instanceKeys } from '@/features/instances/queries'
import { toError } from '@/helpers/errors'
import type { GameInstance } from '@/helpers/types'
import {
	createGroup,
	deleteGroup,
	renameGroup,
	setGroupMemberships,
	setGroupOrder,
} from '@/platform/app-lib/instance-groups/commands'
import type {
	InstanceGroupDefinition,
	InstanceGroupMembershipUpdate,
} from '@/platform/app-lib/instance-groups/types'

import { instanceGroupKeys } from './queries'

const instanceGroupMutationKey = [...instanceGroupKeys.list(), 'mutation'] as const

type Snapshot = Array<[QueryKey, unknown]>

export function useInstanceGroupMutations() {
	const queryClient = useQueryClient()
	const { handleError } = injectNotificationManager()

	const snapshot = async (...keys: QueryKey[]): Promise<Snapshot> => {
		await Promise.all(keys.map((queryKey) => queryClient.cancelQueries({ queryKey })))
		return keys.map((queryKey) => [queryKey, queryClient.getQueryData(queryKey)])
	}

	const rollback = (error: unknown, context: { snapshot: Snapshot } | undefined) => {
		for (const [queryKey, data] of context?.snapshot ?? []) {
			queryClient.setQueryData(queryKey, data)
		}
		handleError(toError(error))
	}

	const settle = async () => {
		if (queryClient.isMutating({ mutationKey: instanceGroupMutationKey }) > 1) return
		await Promise.all([
			queryClient.invalidateQueries({ queryKey: instanceGroupKeys.list() }),
			queryClient.invalidateQueries({ queryKey: instanceKeys.list() }),
		])
	}

	const updateGroups = (update: (groups: InstanceGroupDefinition[]) => InstanceGroupDefinition[]) =>
		queryClient.setQueryData<InstanceGroupDefinition[]>(instanceGroupKeys.list(), (groups) =>
			groups ? update(groups) : groups,
		)

	const updateInstanceGroupIds = (getGroupIds: (instance: GameInstance) => string[] | undefined) =>
		queryClient.setQueryData<GameInstance[]>(instanceKeys.list(), (instances) =>
			instances?.map((instance) => {
				const groupIds = getGroupIds(instance)
				return groupIds ? { ...instance, group_ids: groupIds } : instance
			}),
		)

	const setMemberships = useMutation({
		mutationKey: instanceGroupMutationKey,
		networkMode: 'always',
		mutationFn: (updates: InstanceGroupMembershipUpdate[]) => setGroupMemberships(updates),
		onMutate: async (updates) => {
			const context = { snapshot: await snapshot(instanceKeys.list()) }
			const groupIdsByInstanceId = new Map(
				updates.map((update) => [update.instance_id, update.group_ids]),
			)
			updateInstanceGroupIds((instance) => groupIdsByInstanceId.get(instance.id))
			return context
		},
		onError: (error, _, context) => rollback(error, context),
		onSettled: settle,
	})

	const create = useMutation({
		mutationKey: instanceGroupMutationKey,
		networkMode: 'always',
		mutationFn: async ({ name, instanceIds }: { name: string; instanceIds: Iterable<string> }) => {
			const group = await createGroup(name)
			const selectedIds = new Set(instanceIds)
			const instances = queryClient.getQueryData<GameInstance[]>(instanceKeys.list()) ?? []
			const updates = instances
				.filter((instance) => selectedIds.has(instance.id))
				.map((instance) => ({
					instance_id: instance.id,
					group_ids: [...instance.group_ids, group.id],
				}))
			updateGroups((groups) => [group, ...groups.filter((existing) => existing.id !== group.id)])
			const membershipsAdded =
				updates.length === 0 ||
				(await setMemberships.mutateAsync(updates).then(
					() => true,
					() => false,
				))
			return { group, membershipsAdded }
		},
		onError: (error) => handleError(toError(error)),
		onSettled: settle,
	})

	const rename = useMutation({
		mutationKey: instanceGroupMutationKey,
		networkMode: 'always',
		mutationFn: ({ groupId, name }: { groupId: string; name: string }) =>
			renameGroup(groupId, name),
		onMutate: async ({ groupId, name }) => {
			const context = { snapshot: await snapshot(instanceGroupKeys.list()) }
			updateGroups((groups) =>
				groups.map((group) => (group.id === groupId ? { ...group, name } : group)),
			)
			return context
		},
		onError: (error, _, context) => rollback(error, context),
		onSettled: settle,
	})

	const remove = useMutation({
		mutationKey: instanceGroupMutationKey,
		networkMode: 'always',
		mutationFn: (groupId: string) => deleteGroup(groupId),
		onMutate: async (groupId) => {
			const context = { snapshot: await snapshot(instanceGroupKeys.list(), instanceKeys.list()) }
			updateGroups((groups) => groups.filter((group) => group.id !== groupId))
			updateInstanceGroupIds((instance) =>
				instance.group_ids.includes(groupId)
					? instance.group_ids.filter((id) => id !== groupId)
					: undefined,
			)
			return context
		},
		onError: (error, _, context) => rollback(error, context),
		onSettled: settle,
	})

	const reorder = useMutation({
		mutationKey: instanceGroupMutationKey,
		networkMode: 'always',
		mutationFn: (groupIds: string[]) => setGroupOrder(groupIds),
		onMutate: async (groupIds) => {
			const context = { snapshot: await snapshot(instanceGroupKeys.list()) }
			const order = new Map(groupIds.map((groupId, index) => [groupId, index]))
			updateGroups((groups) =>
				[...groups].sort(
					(a, b) =>
						(order.get(a.id) ?? Number.MIN_SAFE_INTEGER) -
						(order.get(b.id) ?? Number.MIN_SAFE_INTEGER),
				),
			)
			return context
		},
		onError: (error, _, context) => rollback(error, context),
		onSettled: settle,
	})

	return { setMemberships, create, rename, remove, reorder }
}
