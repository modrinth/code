import type { Ref, ShallowRef } from 'vue'

import type {
	FileInfo,
	FileManagerContext,
} from '#ui/layouts/shared/files-tab/providers/file-manager'
import { createContext } from '#ui/providers/create-context'

import type { ArchiveLoadProgress, ArchiveLoadStage, ArchiveSource } from '../types'
import type { ArchiveViewerContext } from './archive-viewer'

/** An archive that has been read and can be browsed. */
export interface OpenArchive extends ArchiveViewerContext {
	/** Browses the archive, for the files layout. */
	manager: FileManagerContext
	/** The entry open in the archive. */
	location: Readonly<Ref<FileInfo>>
	/** Opens an entry by its path within the archive, without a leading slash. */
	open: (path: string) => void
}

/**
 * The archive a workbench shows, which can change (e.g. to another file of a project) while the
 * workbench and its panels stay. Its file manager is provided along with it, standing in for
 * whichever archive is open.
 */
export interface ArchiveSessionContext {
	source: Readonly<Ref<ArchiveSource | null>>
	stage: Readonly<Ref<ArchiveLoadStage>>
	/** Bytes of the archive fetched so far, which keeps growing as files are opened. */
	progress: Readonly<Ref<ArchiveLoadProgress>>
	error: Readonly<Ref<Error | null>>
	/** Whether every nested archive of the open archive has been read. */
	fullyLoaded: Readonly<Ref<boolean>>
	retry: () => void
	/** The archive that has been read, until another one replaces it. */
	current: Readonly<ShallowRef<OpenArchive | null>>
	/** Whether any archive has been opened yet. */
	opened: Readonly<Ref<boolean>>
	/** The path the next archive opens at, e.g. from the page's URL. */
	initialPath: () => string | null
	attach: (archive: OpenArchive) => void
	detach: (archive: OpenArchive) => void
}

export const [injectArchiveSession, provideArchiveSession] = createContext<ArchiveSessionContext>(
	'ArchiveSession',
	'archiveSession',
)
