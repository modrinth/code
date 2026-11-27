import { injectNotificationManager } from '@modrinth/ui'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { type MaybeRefOrGetter, toValue } from 'vue'

import { toError } from '@/helpers/errors'
import {
	createContentGroup,
	deleteContentGroup,
	renameContentGroup,
	setContentGroupMemberships,
	setContentGroupOrder,
} from '@/platform/app-lib/content-groups/commands'
import type { InstanceContentGroup } from '@/platform/app-lib/content-groups/types'

import { contentGroupKeys, contentGroupMutationKey } from './queries'

type Snapshot = { data: InstanceContentGroup[] | undefined }

export function useContentGroupMutations(instanceId: MaybeRefOrGetter<string>) {
	const queryClient = useQueryClient()
	const { handleError } = injectNotificationManager()

	const queryKey = () => contentGroupKeys.list(toValue(instanceId))

	const snapshot = async (): Promise<Snapshot> => {
		await queryClient.cancelQueries({ queryKey: queryKey() })
		return { data: queryClient.getQueryData<InstanceContentGroup[]>(queryKey()) }
	}

	const rollback = (error: unknown, context: Snapshot | undefined) => {
		if (context) queryClient.setQueryData(queryKey(), context.data)
		handleError(toError(error))
	}

	const settle = async () => {
		if (queryClient.isMutating({ mutationKey: contentGroupMutationKey }) > 1) return
		await queryClient.invalidateQueries({ queryKey: queryKey() })
	}

	const updateGroups = (update: (groups: InstanceContentGroup[]) => InstanceContentGroup[]) =>
		queryClient.setQueryData<InstanceContentGroup[]>(queryKey(), (groups) => update(groups ?? []))

	const withoutItemKeys = (
		groups: InstanceContentGroup[],
		contentType: string,
		itemKeys: Set<string>,
	) =>
		groups.map((group) =>
			group.content_type === contentType
				? { ...group, item_keys: group.item_keys.filter((key) => !itemKeys.has(key)) }
				: group,
		)

	const create = useMutation({
		mutationKey: contentGroupMutationKey,
		networkMode: 'always',
		mutationFn: ({
			contentType,
			name,
			itemKeys,
		}: {
			contentType: string
			name: string
			itemKeys: string[]
		}) => createContentGroup(toValue(instanceId), contentType, name, itemKeys),
		onSuccess: (group) => {
			const keys = new Set(group.item_keys)
			updateGroups((groups) => [
				group,
				...withoutItemKeys(
					groups.filter((existing) => existing.id !== group.id),
					group.content_type,
					keys,
				),
			])
		},
		onError: (error) => handleError(toError(error)),
		onSettled: settle,
	})

	const rename = useMutation({
		mutationKey: contentGroupMutationKey,
		networkMode: 'always',
		mutationFn: ({ groupId, name }: { groupId: string; name: string }) =>
			renameContentGroup(groupId, name),
		onMutate: async ({ groupId, name }) => {
			const context = await snapshot()
			updateGroups((groups) =>
				groups.map((group) => (group.id === groupId ? { ...group, name } : group)),
			)
			return context
		},
		onError: (error, _, context) => rollback(error, context),
		onSettled: settle,
	})

	const remove = useMutation({
		mutationKey: contentGroupMutationKey,
		networkMode: 'always',
		mutationFn: (groupId: string) => deleteContentGroup(groupId),
		onMutate: async (groupId) => {
			const context = await snapshot()
			updateGroups((groups) => groups.filter((group) => group.id !== groupId))
			return context
		},
		onError: (error, _, context) => rollback(error, context),
		onSettled: settle,
	})

	const reorder = useMutation({
		mutationKey: contentGroupMutationKey,
		networkMode: 'always',
		mutationFn: ({ contentType, groupIds }: { contentType: string; groupIds: string[] }) =>
			setContentGroupOrder(toValue(instanceId), contentType, groupIds),
		onMutate: async ({ contentType, groupIds }) => {
			const context = await snapshot()
			const order = new Map(groupIds.map((groupId, index) => [groupId, index]))
			updateGroups((groups) => {
				const ordered = groups
					.filter((group) => group.content_type === contentType)
					.sort((a, b) => (order.get(a.id) ?? 0) - (order.get(b.id) ?? 0))
				let index = 0
				return groups.map((group) =>
					group.content_type === contentType ? ordered[index++] : group,
				)
			})
			return context
		},
		onError: (error, _, context) => rollback(error, context),
		onSettled: settle,
	})

	const move = useMutation({
		mutationKey: contentGroupMutationKey,
		networkMode: 'always',
		mutationFn: ({
			contentType,
			itemKeys,
			groupId,
		}: {
			contentType: string
			itemKeys: string[]
			groupId: string | null
		}) => setContentGroupMemberships(toValue(instanceId), contentType, itemKeys, groupId),
		onMutate: async ({ contentType, itemKeys, groupId }) => {
			const context = await snapshot()
			const keys = new Set(itemKeys)
			updateGroups((groups) =>
				withoutItemKeys(groups, contentType, keys).map((group) =>
					group.id === groupId ? { ...group, item_keys: [...group.item_keys, ...keys] } : group,
				),
			)
			return context
		},
		onError: (error, _, context) => rollback(error, context),
		onSettled: settle,
	})

	return { create, rename, remove, reorder, move }
}
