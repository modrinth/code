export function reviewExternalUrl(value: string | undefined): string | undefined {
	if (!value) return
	try {
		const url = new URL(value)
		if (url.protocol === 'https:' || url.protocol === 'http:') return url.href
	} catch {
		return undefined
	}
}
