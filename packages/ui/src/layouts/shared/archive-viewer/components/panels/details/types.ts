import type { ArchiveDetailRow } from '../../../types'

/** How far an archive has been read: nested archives are read in the background. */
export type ArchiveReadStatus = 'read' | 'reading' | 'pending'

/** What an archive holds itself, leaving out the contents of archives nested within it. */
export interface ArchiveCounts {
	files: number
	folders: number
	classFiles: number
	nestedArchives: number
}

/** A described item's details, grouped as they're laid out. */
export interface DetailGroups {
	rows: ArchiveDetailRow[]
	/** Its size as stored in its archive, and in full. */
	stored: { compressed: number; size: number } | null
	hashes: ArchiveDetailRow[]
}
