import { getChangelog, type ChangelogEntry, type Product, type VersionEntry } from '@modrinth/blog'
import dayjs from 'dayjs'

export interface AppRelease {
	version: string
	publishedAt: string
	url: string
}

function resolveChangelogEntry(
	entry: ChangelogEntry,
	appReleaseByVersion: Map<string, AppRelease>,
): VersionEntry | null {
	if (entry.date) {
		return entry as VersionEntry
	}

	if (entry.product !== 'app' || !entry.version) {
		return null
	}

	const release = appReleaseByVersion.get(entry.version)
	if (!release) {
		return null
	}

	return {
		...entry,
		date: dayjs(release.publishedAt),
	}
}

export function resolveChangelogEntries(appReleases: AppRelease[] = []): VersionEntry[] {
	const appReleaseByVersion = new Map(appReleases.map((release) => [release.version, release]))

	return getChangelog().flatMap((entry) => {
		const resolvedEntry = resolveChangelogEntry(entry, appReleaseByVersion)
		return resolvedEntry ? [resolvedEntry] : []
	})
}

export function findChangelogEntry(
	entries: VersionEntry[],
	productParam: string | string[],
	dateParam: string | string[],
): VersionEntry | undefined {
	const product = (Array.isArray(productParam) ? productParam[0] : productParam) as Product
	const date = Array.isArray(dateParam) ? dateParam[0] : dateParam

	return entries.find((entry) => {
		if (entry.product !== product) return false
		if (entry.version && entry.version === date) return true
		return entry.date.unix() === Number(date)
	})
}
