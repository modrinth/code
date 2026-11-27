import type { Archon } from '@modrinth/api-client'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type Ref } from 'vue'

import { serverKeys } from '#ui/composables/server-queries'
import type {
	ContentGroup,
	ContentGroupsManager,
} from '#ui/layouts/shared/content-tab/providers/content-manager'
import type { ContentItem, ContentType } from '#ui/layouts/shared/content-tab/types'
import { getContentGroupItemKey } from '#ui/layouts/shared/content-tab/utils/content-groups'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'

import type { ContentQueries } from './use-content-queries'

const MAX_CONTENT_GROUP_NAME_LENGTH = 256
const STALE_TIME = 30_000
const UNGROUPABLE_TYPES = new Set<ContentType>(['resourcepack'])

type Group = Archon.Content.v1.WorldContentGroup

export interface ContentGroupsOptions {
	serverId: string
	queries: ContentQueries
	items: Readonly<Ref<ContentItem[]>>
}

export function useContentGroups({
	serverId,
	queries,
	items,
}: ContentGroupsOptions): ContentGroupsManager {
	const client = injectModrinthClient()
	const queryClient = useQueryClient()
	const { worldId } = injectModrinthServerContext()
	const { handleError } = injectNotificationManager()

	const queryKey = computed(() => serverKeys.contentGroups(serverId, worldId.value ?? ''))
	const mutationKey = ['content', 'groups', 'v1', serverId, 'mutation'] as const

	const groupsQuery = useQuery({
		queryKey,
		queryFn: () => client.archon.content_v1.getContentGroups(serverId, worldId.value!),
		enabled: computed(() => worldId.value !== null),
		staleTime: STALE_TIME,
	})

	const addonsByItemKey = computed(() => {
		const map = new Map<string, Archon.Content.v1.Addon>()
		for (const item of items.value) {
			const addon = queries.getAddonForItem(item)
			if (addon) map.set(getContentGroupItemKey(item), addon)
		}
		return map
	})

	const groups = computed<ContentGroup[]>(() => {
		const itemKeysByGroup = new Map<string, string[]>()
		for (const [itemKey, addon] of addonsByItemKey.value) {
			if (!addon.group_id) continue
			const keys = itemKeysByGroup.get(addon.group_id) ?? []
			keys.push(itemKey)
			itemKeysByGroup.set(addon.group_id, keys)
		}
		return (groupsQuery.data.value ?? []).map((group) => ({
			id: group.id,
			type: group.content_type,
			name: group.name,
			itemKeys: itemKeysByGroup.get(group.id) ?? [],
		}))
	})

	function filenamesFor(itemKeys: string[]) {
		return itemKeys
			.map((key) => addonsByItemKey.value.get(key)?.filename)
			.filter((filename): filename is string => !!filename)
	}

	async function snapshotGroups() {
		const key = queryKey.value
		await queryClient.cancelQueries({ queryKey: key })
		const previous = queryClient.getQueryData<Group[]>(key)
		return () => queryClient.setQueryData(key, previous)
	}

	function updateGroups(update: (groups: Group[]) => Group[]) {
		queryClient.setQueryData<Group[]>(queryKey.value, (current) => update(current ?? []))
	}

	function setAddonGroups(type: ContentType, filenames: string[], groupId: string | null) {
		const targets = new Set(filenames)
		return queries.optimisticallyUpdateAddons((addons) =>
			addons.map((addon) =>
				addon.kind === type && targets.has(addon.filename) ? { ...addon, group_id: groupId } : addon,
			),
		)
	}

	function fail(error: unknown, rollbacks: Array<() => void> = []) {
		for (const rollback of rollbacks) rollback()
		handleError(error instanceof Error ? error : new Error(String(error)))
	}

	async function settle() {
		if (queryClient.isMutating({ mutationKey }) > 1) return
		await queryClient.invalidateQueries({ queryKey: queryKey.value })
	}

	const create = useMutation({
		mutationKey,
		mutationFn: ({ type, name, filenames }: { type: ContentType; name: string; filenames: string[] }) =>
			client.archon.content_v1.createContentGroup(serverId, worldId.value!, {
				content_type: type,
				name,
				filenames,
			}),
		onSuccess: async (group, { type, filenames }) => {
			updateGroups((current) => [group, ...current.filter((existing) => existing.id !== group.id)])
			await setAddonGroups(type, filenames, group.id)
		},
		onError: (error) => fail(error),
		onSettled: settle,
	})

	const rename = useMutation({
		mutationKey,
		mutationFn: ({ groupId, name }: { groupId: string; name: string }) =>
			client.archon.content_v1.renameContentGroup(serverId, worldId.value!, groupId, { name }),
		onMutate: async ({ groupId, name }) => {
			const rollback = await snapshotGroups()
			updateGroups((current) =>
				current.map((group) => (group.id === groupId ? { ...group, name } : group)),
			)
			return [rollback]
		},
		onError: (error, _, rollbacks) => fail(error, rollbacks),
		onSettled: settle,
	})

	const remove = useMutation({
		mutationKey,
		mutationFn: (groupId: string) =>
			client.archon.content_v1.deleteContentGroup(serverId, worldId.value!, groupId),
		onMutate: async (groupId) => {
			const rollbackGroups = await snapshotGroups()
			updateGroups((current) => current.filter((group) => group.id !== groupId))
			const rollbackAddons = await queries.optimisticallyUpdateAddons((addons) =>
				addons.map((addon) => (addon.group_id === groupId ? { ...addon, group_id: null } : addon)),
			)
			return [rollbackGroups, rollbackAddons]
		},
		onError: (error, _, rollbacks) => fail(error, rollbacks),
		onSettled: settle,
	})

	const reorder = useMutation({
		mutationKey,
		mutationFn: ({ type, groupIds }: { type: ContentType; groupIds: string[] }) =>
			client.archon.content_v1.setContentGroupOrder(serverId, worldId.value!, {
				content_type: type,
				group_ids: groupIds,
			}),
		onMutate: async ({ type, groupIds }) => {
			const rollback = await snapshotGroups()
			const order = new Map(groupIds.map((groupId, index) => [groupId, index]))
			updateGroups((current) => {
				const ordered = current
					.filter((group) => group.content_type === type)
					.sort((a, b) => (order.get(a.id) ?? 0) - (order.get(b.id) ?? 0))
				let index = 0
				return current.map((group) => (group.content_type === type ? ordered[index++] : group))
			})
			return [rollback]
		},
		onError: (error, _, rollbacks) => fail(error, rollbacks),
		onSettled: settle,
	})

	const move = useMutation({
		mutationKey,
		mutationFn: ({
			type,
			filenames,
			groupId,
		}: {
			type: ContentType
			filenames: string[]
			groupId: string | null
		}) =>
			client.archon.content_v1.setContentGroupMemberships(serverId, worldId.value!, {
				content_type: type,
				filenames,
				group_id: groupId,
			}),
		onMutate: async ({ type, filenames, groupId }) => [
			await setAddonGroups(type, filenames, groupId),
		],
		onError: (error, _, rollbacks) => fail(error, rollbacks),
		onSettled: settle,
	})

	const succeeded = (promise: Promise<unknown>) =>
		promise.then(
			() => true,
			() => false,
		)

	return {
		groups,
		supportsType: (type) => !UNGROUPABLE_TYPES.has(type),
		maxNameLength: MAX_CONTENT_GROUP_NAME_LENGTH,
		create: (type, name, itemKeys) =>
			create.mutateAsync({ type, name, filenames: filenamesFor(itemKeys) }).then(
				(group) => ({
					id: group.id,
					type: group.content_type,
					name: group.name,
					itemKeys: itemKeys.filter((key) => addonsByItemKey.value.has(key)),
				}),
				() => null,
			),
		rename: (groupId, name) => succeeded(rename.mutateAsync({ groupId, name })),
		remove: (groupId) => succeeded(remove.mutateAsync(groupId)),
		reorder: (type, groupIds) => succeeded(reorder.mutateAsync({ type, groupIds })),
		move: (type, itemKeys, groupId) =>
			succeeded(move.mutateAsync({ type, filenames: filenamesFor(itemKeys), groupId })),
	}
}
