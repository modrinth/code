import type { AbstractModrinthClient } from '@modrinth/api-client'

import { serverKeys, sharedInstanceKeys } from '#ui/composables/server-queries'

import { resolveServerShareDiff } from './share-diff'

export function serverShareDiffQueryOptions(
	client: AbstractModrinthClient,
	serverId: string,
	worldId: string,
	userId: string | undefined,
) {
	return {
		queryKey: serverKeys.shareDiff(serverId, worldId, userId),
		queryFn: async () => {
			const diff = await client.archon.content_v1.getShareDiff(serverId, worldId)
			return { diff, items: resolveServerShareDiff(diff) }
		},
		retry: false,
	}
}

export function serverShareableFilesQueryOptions(
	client: AbstractModrinthClient,
	serverId: string,
	worldId: string,
	userId: string | undefined,
) {
	return {
		queryKey: serverKeys.shareFiles(serverId, worldId, userId),
		queryFn: () => client.archon.content_v1.getShareableFiles(serverId, worldId),
		retry: false,
	}
}

export function sharedInstanceInvitesQueryOptions(
	client: AbstractModrinthClient,
	instanceId: string,
	userId: string | undefined,
) {
	return {
		queryKey: sharedInstanceKeys.invites(instanceId, userId),
		queryFn: () => client.sharedinstances.invites_v1.list(instanceId),
		retry: false,
	}
}
