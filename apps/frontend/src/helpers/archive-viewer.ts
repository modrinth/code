import type { Labrinth } from '@modrinth/api-client'
import type { ArchiveSource } from '@modrinth/ui'

/** Whether a URL is one of Modrinth's own, which the archive viewer is allowed to open. */
export function isModrinthCdnUrl(url: string) {
	try {
		const { protocol, hostname } = new URL(url)
		return (
			protocol === 'https:' && (hostname === 'modrinth.com' || hostname.endsWith('.modrinth.com'))
		)
	} catch {
		return false
	}
}

/** A version's primary file, or its first one if none is marked primary. */
export function primaryFile(version: Labrinth.Versions.v3.Version) {
	return version.files.find((file) => file.primary) ?? version.files[0]
}

export function archiveSourceFrom(file: Labrinth.Versions.v3.VersionFile): ArchiveSource {
	return { url: file.url, name: file.filename, size: file.size, sha512Hash: file.hashes.sha512 }
}

/** The name an archive is shown by when only its URL is known, i.e. its file name. */
export function archiveNameFrom(url: string) {
	return decodeURIComponent(new URL(url).pathname.split('/').pop() ?? url)
}

/**
 * The path of an entry within an archive. Slicer's entries start with the archive's name, which
 * is dropped, so its links (and entries made for it) open as they are.
 */
export function archivePathFrom(entry: string | null | undefined, archiveName: string) {
	if (!entry) return null
	return entry.startsWith(`${archiveName}/`) ? entry.slice(archiveName.length + 1) : entry
}

/**
 * A link to the archive viewer: a project's version (its latest by default), or an archive by its
 * URL, optionally opened at an entry within it (a path within the archive, or a Slicer entry).
 */
export function archiveViewerUrl(options: {
	project?: string
	version?: string
	url?: string
	file?: string | null
}) {
	const params = new URLSearchParams()
	if (options.project) params.set('project', options.project)
	if (options.version) params.set('version', options.version)
	if (options.url) params.set('url', options.url)
	if (options.file) params.set('file', options.file)
	return `/archive?${params}`
}
