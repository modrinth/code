import type { Archon } from '@modrinth/api-client'

export function getHostingServerAddress(
	net: Archon.Servers.v0.Net | null | undefined,
	subdomain?: string,
) {
	const domain = net?.domain || subdomain
	if (domain) return domain.includes('.') ? domain : `${domain}.modrinth.gg`
	if (!net?.ip) return ''
	const host = net.ip.includes(':') && !net.ip.startsWith('[') ? `[${net.ip}]` : net.ip
	return net.port && net.port !== 25565 ? `${host}:${net.port}` : host
}
