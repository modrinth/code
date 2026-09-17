import type { AbstractModrinthClient } from '@modrinth/api-client'

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
