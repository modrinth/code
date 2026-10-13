import type { AbstractModrinthClient } from '@modrinth/api-client'

const INVITE_MAX_AGE = 7 * 24 * 60 * 60
const DEFAULT_INVITE_MAX_USES = 10

export async function ensureServerInviteLink(
	client: AbstractModrinthClient,
	instanceId: string,
	remainingSlots?: number,
	options: { maximizeUses?: boolean } = {},
): Promise<string> {
	const links = await client.sharedinstances.invites_v1.list(instanceId)
	const active = links.find(
		(link) => new Date(link.expiration).getTime() > Date.now() && link.uses < link.max_uses,
	)
	if (active) return active.id

	let availableSlots = remainingSlots
	if (availableSlots === undefined) {
		const players = await client.sharedinstances.instances_v1.getUsers(instanceId)
		availableSlots = Math.max(0, 50 - players.users.length)
	}
	if (availableSlots === 0) throw new Error('This shared instance has no available player slots.')

	const invite = await client.sharedinstances.invites_v1.create(instanceId, {
		max_age: INVITE_MAX_AGE,
		max_uses: options.maximizeUses
			? availableSlots
			: Math.min(DEFAULT_INVITE_MAX_USES, availableSlots),
	})
	return invite.id
}
