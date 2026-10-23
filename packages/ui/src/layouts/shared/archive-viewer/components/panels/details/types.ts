import type { HashType } from '#ui/helpers/toolbox/file'

import type { ArchiveDetailRow } from '../../../types'

/** A described item's details, grouped as they're laid out. */
export interface DetailGroups {
	rows: ArchiveDetailRow[]
	/** In-depth details, folded away by default. */
	more: ArchiveDetailRow[]
	/** Its size as stored in its archive, and in full. */
	stored: { compressed: number; size: number } | null
	/** Computes the file's hash of a type, for those the viewer's settings list. */
	hash: ((type: HashType) => Promise<string>) | undefined
}
