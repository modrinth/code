import type { AbstractModrinthClient } from '@modrinth/api-client'

import { ensureServerInviteLink } from '#ui/layouts/shared/server-sharing'

export async function createServerOnboardingInvite(
	client: AbstractModrinthClient,
	serverId: string,
	worldId: string,
	siteUrl: string,
	isCurrent: () => boolean,
): Promise<string | null> {
	const details = await client.archon.servers_v1.get(serverId)
	if (!isCurrent()) return null
	const existingId = details.worlds.find((world) => world.id === worldId)?.content
		?.shared_instance_id
	const instanceId =
		existingId ??
		(await client.archon.content_v1.share(serverId, worldId, [], { timeout: 20_000 }))
			.shared_instance_id
	if (!isCurrent()) return null
	const inviteId = await ensureServerInviteLink(client, instanceId)
	return `${siteUrl.replace(/\/$/, '')}/share/${encodeURIComponent(inviteId)}`
}
