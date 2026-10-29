import type { AbstractModrinthClient, SharedInstances } from '@modrinth/api-client'

export const SHARED_INSTANCE_MAX_PLAYERS = 50
export const INVITE_LINK_MAX_AGE_SECONDS = 7 * 24 * 60 * 60
export const DEFAULT_INVITE_LINK_MAX_USES = 10

export function remainingPlayerSlots(playerCount: number) {
	return Math.max(0, SHARED_INSTANCE_MAX_PLAYERS - playerCount)
}

export function findActiveInviteLink(
	links: SharedInstances.Invites.v1.InviteLink[] | null | undefined,
) {
	return links?.find(
		(link) => new Date(link.expiration).getTime() > Date.now() && link.uses < link.max_uses,
	)
}

export async function ensureServerInviteLink(
	client: AbstractModrinthClient,
	instanceId: string,
	remainingSlots?: number,
	options: { maximizeUses?: boolean } = {},
): Promise<string> {
	const active = findActiveInviteLink(await client.sharedinstances.invites_v1.list(instanceId))
	if (active) return active.id

	let availableSlots = remainingSlots
	if (availableSlots === undefined) {
		const players = await client.sharedinstances.instances_v1.getUsers(instanceId)
		availableSlots = remainingPlayerSlots(players.users.length)
	}
	if (availableSlots === 0) throw new Error('This shared instance has no available player slots.')

	const invite = await client.sharedinstances.invites_v1.create(instanceId, {
		max_age: INVITE_LINK_MAX_AGE_SECONDS,
		max_uses: options.maximizeUses
			? availableSlots
			: Math.min(DEFAULT_INVITE_LINK_MAX_USES, availableSlots),
	})
	return invite.id
}

/**
 * Shares the world when it has no shared instance yet, then returns a share URL for an
 * invite link that still has uses left. Returns `null` once `isCurrent` reports the caller moved on.
 */
export async function createServerInviteLink(
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
	const inviteId = await ensureServerInviteLink(client, instanceId, undefined, {
		maximizeUses: true,
	})
	return `${siteUrl.replace(/\/$/, '')}/share/${encodeURIComponent(inviteId)}`
}
