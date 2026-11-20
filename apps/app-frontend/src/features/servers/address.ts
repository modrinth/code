const DEFAULT_MINECRAFT_SERVER_PORT = 25565

function parseServerPort(port: string): number | null {
	const parsed = Number.parseInt(port, 10)
	return Number.isInteger(parsed) && parsed > 0 && parsed <= 65535 ? parsed : null
}

function parseServerHost(address: string): string {
	const trimmedAddress = address.trim()
	if (!trimmedAddress) return ''

	if (trimmedAddress.startsWith('[')) {
		const closingBracket = trimmedAddress.indexOf(']')
		if (closingBracket > 0) {
			return trimmedAddress.slice(1, closingBracket).trim().toLowerCase()
		}
	}

	const firstColon = trimmedAddress.indexOf(':')
	const lastColon = trimmedAddress.lastIndexOf(':')

	if (firstColon !== -1 && firstColon === lastColon) {
		return trimmedAddress.slice(0, firstColon).trim().toLowerCase()
	}

	return trimmedAddress.toLowerCase()
}

function parseExplicitPort(address: string): number | null {
	if (address.startsWith('[')) {
		const closingBracket = address.indexOf(']')
		if (closingBracket <= 0) return null
		const suffix = address.slice(closingBracket + 1)
		return suffix.startsWith(':') ? parseServerPort(suffix.slice(1)) : null
	}

	const firstColon = address.indexOf(':')
	const lastColon = address.lastIndexOf(':')
	if (firstColon === -1 || firstColon !== lastColon) return null
	return parseServerPort(address.slice(firstColon + 1))
}

export function normalizeServerAddress(address: string): string {
	const trimmedAddress = address.trim()
	const host = parseServerHost(trimmedAddress)
	if (!host) return ''
	return `${host}:${parseExplicitPort(trimmedAddress) ?? DEFAULT_MINECRAFT_SERVER_PORT}`
}

export function getServerAddress(javaServer?: { address?: string | null } | null) {
	return javaServer?.address ?? null
}
