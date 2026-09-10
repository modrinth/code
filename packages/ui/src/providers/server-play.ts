import type { Archon } from '@modrinth/api-client'

import { createContext } from './create-context'

export type ServerPlayTarget = {
	serverId: string
	worldId: string
}

export const [injectServerPlay, provideServerPlay] = createContext<{
	play: (target: ServerPlayTarget) => Promise<void>
}>('root', 'serverPlay')

export function getHostingServerAddress(net: Archon.Servers.v0.Net, subdomain?: string) {
	const domain = net.domain || subdomain
	if (domain) return domain.includes('.') ? domain : `${domain}.modrinth.gg`
	if (!net.ip) return ''
	const host = net.ip.includes(':') && !net.ip.startsWith('[') ? `[${net.ip}]` : net.ip
	return net.port && net.port !== 25565 ? `${host}:${net.port}` : host
}
