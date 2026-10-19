/** An archive (e.g. a mod's `.jar` or a modpack's `.mrpack`) that can be opened in the archive viewer. */
export interface ArchiveSource {
	/** Where the archive is downloaded from, usually a Modrinth CDN link. */
	url: string
	/** The archive's file name, shown as the root of its paths. */
	name: string
	/** Size in bytes, which saves a request when reading the archive by ranges. */
	size?: number
	/** The archive's SHA-512 hash, so an archive already open under another URL is reused. */
	sha512Hash?: string
}

/** `reading` covers fetching the archive's index; its tree is `ready` before nested archives are read. */
export type ArchiveLoadStage = 'idle' | 'reading' | 'ready' | 'error'

/** A labelled value listed in one of the archive viewer's detail panels. */
export interface ArchiveDetailRow {
	id: string
	label: string
	value: string | null | undefined
	/** Shows a spinner while the value is computed, e.g. a hash. */
	loading?: boolean
	/** Shows the value in a monospace button that copies it. */
	copyable?: boolean
	/** Shows the value shortened, in full on hover, in a button that copies it. */
	hash?: boolean
	/** Shown in place of a `hash` value, which then only shows on hover, e.g. a link's source. */
	display?: string
	/** More about the value, shown on hover. */
	tooltip?: string
	/** Colours the value, e.g. a check that passed (`green`) or failed (`red`). */
	tone?: 'green' | 'red'
	href?: string
	/**
	 * Where the row shows within its item: always, only behind its details pull tab, or never.
	 * Defaults to always.
	 */
	visibility?: 'always' | 'details' | 'never'
}

export interface ArchiveLoadProgress {
	loaded: number
	total: number | null
}

/** How far an archive has been read: nested archives are read in the background. */
export type ArchiveReadStatus = 'read' | 'reading' | 'pending'

/** What an archive holds itself, leaving out the contents of archives nested within it. */
export interface ArchiveCounts {
	files: number
	folders: number
	classFiles: number
	nestedArchives: number
}
