import type { Labrinth } from '@modrinth/api-client'
import type { injectModrinthClient } from '@modrinth/ui'
import type { QueryClient } from '@tanstack/vue-query'

import { hiddenInstanceIdsQueryOptions } from '@/features/library/hidden-instances/queries'
import { get_user, get_user_many } from '@/helpers/cache.js'
import { install_get_shared_instance_preview } from '@/helpers/install'
import type { InstanceLoader } from '@/helpers/types'
import type { CachedSharedInstanceSummary } from '@/platform/app-lib/shared-instances/commands'

import type { UninstalledSharedInstance } from './types'

type ModrinthClient = ReturnType<typeof injectModrinthClient>

export const sharedInstanceLibraryKeys = {
	all: ['shared-instances', 'library'] as const,
	cached: (userId: string | null) => [...sharedInstanceLibraryKeys.all, 'cached', userId] as const,
	ids: (userId: string | null) => [...sharedInstanceLibraryKeys.all, 'ids', userId] as const,
	detail: (userId: string | null, sharedInstanceId: string) =>
		[...sharedInstanceLibraryKeys.all, 'detail', userId, sharedInstanceId] as const,
	owners: (ownerIds: string[]) =>
		[...sharedInstanceLibraryKeys.all, 'owners', ...ownerIds] as const,
	installPreview: (sharedInstanceId: string) =>
		[...sharedInstanceLibraryKeys.all, 'install-preview', sharedInstanceId] as const,
}

export const SHARED_INSTANCE_ID_PREFIX = 'shared-instance:'

export const getUninstalledSharedInstanceId = (sharedInstanceId: string) =>
	`${SHARED_INSTANCE_ID_PREFIX}${sharedInstanceId}`

export const fromCachedSummary = (
	summary: CachedSharedInstanceSummary,
): UninstalledSharedInstance => ({
	type: 'uninstalled_shared_instance',
	id: getUninstalledSharedInstanceId(summary.shared_instance_id),
	shared_instance_id: summary.shared_instance_id,
	role: summary.role,
	manager_id: summary.manager_id,
	name: summary.name,
	icon_path: summary.icon_url ?? undefined,
	game_version: summary.game_version,
	loader: summary.loader,
})

export const toCachedSummary = (
	instance: UninstalledSharedInstance,
): CachedSharedInstanceSummary => ({
	shared_instance_id: instance.shared_instance_id,
	role: instance.role,
	manager_id: instance.manager_id,
	name: instance.name,
	icon_url: instance.icon_path ?? null,
	game_version: instance.game_version,
	loader: instance.loader,
})

export const sharedInstanceIdsQueryOptions = (client: ModrinthClient, userId: string) => ({
	queryKey: sharedInstanceLibraryKeys.ids(userId),
	queryFn: () => client.sharedinstances.instances_v1.getForUser(userId, { include_joined: true }),
	staleTime: 60_000,
})

const loadSharedInstance = async (
	client: ModrinthClient,
	currentUserId: string,
	sharedInstanceId: string,
): Promise<UninstalledSharedInstance | null> => {
	const [instance, version, users] = await Promise.all([
		client.sharedinstances.instances_v1.get(sharedInstanceId),
		client.sharedinstances.instances_v1.getLatestVersion(sharedInstanceId),
		client.sharedinstances.instances_v1.getUsers(sharedInstanceId),
	])
	if (instance.quarantine || !version.ready) return null

	const ownerId = users.users.find((user) => user.join_type === 'owner')?.id ?? null
	const isOwner = ownerId === currentUserId

	return {
		type: 'uninstalled_shared_instance',
		id: getUninstalledSharedInstanceId(sharedInstanceId),
		shared_instance_id: sharedInstanceId,
		role: isOwner ? 'owner' : 'member',
		manager_id: isOwner ? null : ownerId,
		name: instance.name,
		icon_path: instance.icon ?? undefined,
		game_version: version.game_version,
		loader: version.loader as InstanceLoader,
	}
}

export const sharedInstanceDetailQueryOptions = (
	client: ModrinthClient,
	userId: string,
	sharedInstanceId: string,
) => ({
	queryKey: sharedInstanceLibraryKeys.detail(userId, sharedInstanceId),
	queryFn: () => loadSharedInstance(client, userId, sharedInstanceId),
	staleTime: 60_000,
})

export const sharedInstanceOwnersQueryOptions = (ownerIds: string[]) => ({
	queryKey: sharedInstanceLibraryKeys.owners(ownerIds),
	queryFn: async () => (await get_user_many(ownerIds)) as Labrinth.Users.v2.User[],
	staleTime: 5 * 60_000,
})

export const sharedInstanceInstallPreviewQueryOptions = (instance: UninstalledSharedInstance) => ({
	queryKey: sharedInstanceLibraryKeys.installPreview(instance.shared_instance_id),
	queryFn: async () => {
		const [preview, manager] = await Promise.all([
			install_get_shared_instance_preview(instance.shared_instance_id, instance.name),
			instance.manager_id
				? (get_user(instance.manager_id) as Promise<Labrinth.Users.v2.User>).catch(() => null)
				: null,
		])
		return { preview, manager }
	},
	staleTime: 60_000,
})

export async function prefetchSharedInstanceLibrary(
	queryClient: QueryClient,
	client: ModrinthClient,
	userId: string,
) {
	void queryClient.prefetchQuery(hiddenInstanceIdsQueryOptions(userId))
	const sharedInstanceIds = await queryClient.ensureQueryData(
		sharedInstanceIdsQueryOptions(client, userId),
	)
	await Promise.all(
		[...new Set(sharedInstanceIds)].map((sharedInstanceId) =>
			queryClient.prefetchQuery(sharedInstanceDetailQueryOptions(client, userId, sharedInstanceId)),
		),
	)
}
