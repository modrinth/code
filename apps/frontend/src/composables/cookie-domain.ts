export const getCookieDomain = (): string | undefined => {
	if (import.meta.server) {
		const headers = useRequestHeaders(['host'])
		const host = headers.host
		if (!host) return undefined

		const hostname = host.split(':')[0]

		if (
			hostname === 'localhost' ||
			hostname === '127.0.0.1' ||
			hostname.match(/^\d+\.\d+\.\d+\.\d+$/)
		) {
			return undefined
		}

		const parts = hostname.split('.')
		if (parts.length >= 2) {
			return `.${parts.slice(-2).join('.')}`
		}
		return undefined
	}

	if (typeof window === 'undefined') return undefined

	const hostname = window.location.hostname
	if (
		hostname === 'localhost' ||
		hostname === '127.0.0.1' ||
		hostname.match(/^\d+\.\d+\.\d+\.\d+$/)
	) {
		return undefined
	}

	const parts = hostname.split('.')
	if (parts.length >= 2) {
		return `.${parts.slice(-2).join('.')}`
	}
	return undefined
}
