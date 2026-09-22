import type { AbstractModrinthClient } from '@modrinth/api-client'

import { resolveServerShareDiff } from './share-diff'

export function serverShareDiffQueryOptions(
	client: AbstractModrinthClient,
	serverId: string,
	worldId: string,
	userId: string | undefined,
) {
	return {
		queryKey: ['servers', 'share-diff', serverId, worldId, userId] as const,
		queryFn: async () => {
			const diff = await client.archon.content_v1.getShareDiff(serverId, worldId)
			return { diff, items: resolveServerShareDiff(diff) }
		},
		retry: false,
	}
}

export function sharedInstanceInvitesQueryOptions(
	client: AbstractModrinthClient,
	instanceId: string,
	userId: string | undefined,
) {
	return {
		queryKey: ['shared-instances', instanceId, 'invites', userId] as const,
		queryFn: () => client.sharedinstances.invites_v1.list(instanceId),
		retry: false,
	}
}
