import type { AbstractModrinthClient } from '@modrinth/api-client'

const INVITE_MAX_AGE = 7 * 24 * 60 * 60

export async function ensureServerInviteLink(
	client: AbstractModrinthClient,
	instanceId: string,
	remainingSlots?: number,
): Promise<string> {
	const links = await client.sharedinstances.invites_v1.list(instanceId)
	const active = links.find(
		(link) => new Date(link.expiration).getTime() > Date.now() && link.uses < link.max_uses,
	)
	if (active) return active.id

	let availableSlots = remainingSlots
	if (availableSlots === undefined) {
		const players = await client.sharedinstances.instances_v1.getUsers(instanceId)
		availableSlots = Math.max(0, 50 - players.users.length - players.tokens)
	}
	if (availableSlots === 0) throw new Error('This shared instance has no available player slots.')

	const invite = await client.sharedinstances.invites_v1.create(instanceId, {
		max_age: INVITE_MAX_AGE,
		max_uses: Math.min(10, availableSlots),
	})
	return invite.id
}
