import { normalizeServerAddress } from '@/features/servers/address'
import { ensureManagedServerInInstance } from '@/platform/app-lib/worlds/commands'
import type { ServerWorld, World } from '@/platform/app-lib/worlds/types'

import { isServerWorld } from './world-utils'

export function resolveManagedServerWorld(
	worlds: World[],
	managedName: string | null | undefined,
	managedAddress: string | null | undefined,
): ServerWorld | null {
	if (!managedName || !managedAddress) return null

	const normalizedManagedAddress = normalizeServerAddress(managedAddress)
	if (!normalizedManagedAddress) return null

	const servers = worlds.filter(isServerWorld).sort((a, b) => a.index - b.index)
	const matchingAddress = servers.filter(
		(server) => normalizeServerAddress(server.address) === normalizedManagedAddress,
	)

	return matchingAddress.find((server) => server.name === managedName) ?? matchingAddress[0] ?? null
}

export async function ensureManagedServerWorldExists(
	instanceId: string,
	serverName: string | null,
	serverAddress: string | null,
) {
	if (!instanceId || !serverName || !serverAddress) return
	try {
		await ensureManagedServerInInstance(instanceId, serverName, serverAddress)
	} catch (err) {
		console.error('Failed to ensure managed server world exists:', err)
	}
}
