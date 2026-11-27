import { autoToHTML } from '@sfirew/minecraft-motd-parser'

import { getServerStatus } from '@/platform/app-lib/worlds/commands'
import type { Chat, ProtocolVersion, ServerStatus, World } from '@/platform/app-lib/worlds/types'

const LEGACY_PROTOCOL: ProtocolVersion = { version: 74, legacy: true }

export type ServerData = {
	refreshing: boolean
	lastSuccessfulRefresh?: number
	status?: ServerStatus
	rawMotd?: string | Chat
	renderedMotd?: string
}

export async function getServerLatency(
	address: string,
	protocolVersion: ProtocolVersion | null = null,
): Promise<number | undefined> {
	const pings: number[] = []
	for (let i = 0; i < 3; i++) {
		const status = await getServerStatus(address, protocolVersion).catch(() => undefined)
		if (status?.ping != null) pings.push(status.ping)
	}
	if (pings.length === 0) return undefined
	return Math.round(pings.reduce((sum, ping) => sum + ping, 0) / pings.length)
}

export async function refreshServerData(
	serverData: ServerData,
	protocolVersion: ProtocolVersion | null,
	address: string,
): Promise<void> {
	const refreshTime = Date.now()
	serverData.refreshing = true
	try {
		const status = await getServerStatus(address, protocolVersion)
		if (serverData.lastSuccessfulRefresh && serverData.lastSuccessfulRefresh > refreshTime) return
		serverData.lastSuccessfulRefresh = Date.now()
		serverData.status = status
		if (status.description) {
			serverData.rawMotd = status.description
			serverData.renderedMotd = autoToHTML(status.description)
		} else {
			delete serverData.rawMotd
			delete serverData.renderedMotd
		}
	} catch (err) {
		console.error(`Refreshing addr ${address}`, protocolVersion, err)
		if (!protocolVersion?.legacy) {
			await refreshServerData(serverData, LEGACY_PROTOCOL, address)
			return
		}
		if (!serverData.lastSuccessfulRefresh || serverData.lastSuccessfulRefresh <= refreshTime) {
			delete serverData.status
			delete serverData.rawMotd
			delete serverData.renderedMotd
		}
	} finally {
		serverData.refreshing = false
	}
}

export async function refreshServers(
	worlds: World[],
	serverData: Record<string, ServerData>,
	protocolVersion: ProtocolVersion | null,
	ping = true,
): Promise<void> {
	const addresses = new Set<string>()
	for (const world of worlds) {
		if (world.type !== 'server') continue
		addresses.add(world.address)
		if (serverData[world.address]) {
			serverData[world.address].refreshing = true
		} else {
			serverData[world.address] = { refreshing: true }
		}
	}

	if (!ping) return

	await Promise.all(
		[...addresses].map((address) =>
			refreshServerData(serverData[address], protocolVersion, address),
		),
	)
}
