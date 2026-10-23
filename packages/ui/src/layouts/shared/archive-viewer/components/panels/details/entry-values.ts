/**
 * The earliest date a zip entry can carry. Archives built without timestamps store it, and it
 * shows as the last hours of 1979 or the first of 1980 depending on the time zone, so dates up to
 * the start of February 1980 are treated as unset.
 */
const PLACEHOLDER_DATE_LIMIT = Date.UTC(1980, 1, 2)

export function isPlaceholderDate(date: Date | undefined | null) {
	return !date || Number.isNaN(date.getTime()) || date.getTime() < PLACEHOLDER_DATE_LIMIT
}

export type ArchiveSourceHost = 'modrinth' | 'curseforge' | 'external'

/** Which CDN an archive was downloaded from, if it's one of the known ones. */
export function sourceHost(url: string): ArchiveSourceHost {
	let hostname: string
	try {
		hostname = new URL(url).hostname
	} catch {
		return 'external'
	}
	const within = (domain: string) => hostname === domain || hostname.endsWith(`.${domain}`)
	if (within('modrinth.com')) return 'modrinth'
	if (within('forgecdn.net') || within('curseforge.com')) return 'curseforge'
	return 'external'
}
