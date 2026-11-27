import type { ContentGroup, ContentGroupsManager, ContentType } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'

import { MAX_CONTENT_GROUP_NAME_LENGTH } from '@/platform/app-lib/content-groups/types'

import { contentGroupListQueryOptions } from './queries'
import { useContentGroupMutations } from './use-content-group-mutations'

const UNGROUPABLE_TYPES = new Set<ContentType>(['resourcepack'])

export function useInstanceContentGroups(
	instanceId: MaybeRefOrGetter<string>,
): ContentGroupsManager {
	const query = useQuery(computed(() => contentGroupListQueryOptions(toValue(instanceId))))
	const mutations = useContentGroupMutations(instanceId)

	const groups = computed<ContentGroup[]>(() =>
		(query.data.value ?? []).map((group) => ({
			id: group.id,
			type: group.content_type as ContentType,
			name: group.name,
			itemKeys: group.item_keys,
		})),
	)

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
			mutations.create.mutateAsync({ contentType: type, name, itemKeys }).then(
				(group) => ({
					id: group.id,
					type: group.content_type as ContentType,
					name: group.name,
					itemKeys: group.item_keys,
				}),
				() => null,
			),
		rename: (groupId, name) => succeeded(mutations.rename.mutateAsync({ groupId, name })),
		remove: (groupId) => succeeded(mutations.remove.mutateAsync(groupId)),
		reorder: (type, groupIds) =>
			succeeded(mutations.reorder.mutateAsync({ contentType: type, groupIds })),
		move: (type, itemKeys, groupId) =>
			succeeded(mutations.move.mutateAsync({ contentType: type, itemKeys, groupId })),
	}
}
