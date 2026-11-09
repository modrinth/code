import { MinecraftServerIcon } from '@modrinth/assets'
import { keepPreviousData, useQuery } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'

import {
	getSharedInstanceManagerId,
	isOwnedByOthers,
	isUninstalledSharedInstance,
	type LibraryInstance,
	type SharedInstanceOwner,
} from '@/features/library/library-instance'
import { getInstanceIconUrl } from '@/helpers/instance'

import { sharedInstanceOwnersQueryOptions } from './queries'

export function useSharedInstanceOwners(instances: MaybeRefOrGetter<LibraryInstance[]>) {
	const ownerIds = computed(() =>
		[
			...new Set(
				toValue(instances).flatMap((instance) => {
					const managerId = isOwnedByOthers(instance) ? getSharedInstanceManagerId(instance) : null
					return managerId ? [managerId] : []
				}),
			),
		].sort(),
	)

	const ownersQuery = useQuery(
		computed(() => ({
			...sharedInstanceOwnersQueryOptions(ownerIds.value),
			enabled: ownerIds.value.length > 0,
			placeholderData: keepPreviousData,
		})),
	)

	const ownersById = computed(
		() => new Map((ownersQuery.data.value ?? []).map((user) => [user.id, user])),
	)

	const getSharedInstanceOwner = (instance: LibraryInstance): SharedInstanceOwner | null => {
		if (!isOwnedByOthers(instance)) return null

		if (!isUninstalledSharedInstance(instance) && instance.shared_instance?.server_manager_name) {
			return {
				type: 'server',
				name: instance.shared_instance.server_manager_name,
				avatarUrl:
					getInstanceIconUrl(instance.shared_instance.server_manager_icon_url) ??
					MinecraftServerIcon,
			}
		}

		const managerId = getSharedInstanceManagerId(instance)
		const owner = managerId ? ownersById.value.get(managerId) : undefined
		return owner
			? { type: 'user', name: owner.username, avatarUrl: owner.avatar_url ?? null }
			: null
	}

	return { getSharedInstanceOwner }
}
