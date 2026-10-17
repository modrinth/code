import type { Ref } from 'vue'

import type { RootZipTreeNode, ZipHandler, ZipNode } from '#ui/helpers/toolbox/zip'
import { createContext } from '#ui/providers/create-context'

import type { ArchiveSource } from '../types'

/** The archive open in the archive viewer, for its panels. */
export interface ArchiveViewerContext {
	root: RootZipTreeNode
	handler: ZipHandler
	source: ArchiveSource
	/** Whether every nested archive has been read. */
	fullyLoaded: Readonly<Ref<boolean>>
	/** Bumped whenever nodes are added to the tree. */
	treeVersion: Readonly<Ref<number>>
	/** The archive's own SHA-512, once it has been downloaded in full and hashed. */
	sha512Hash: Readonly<Ref<string | null>>
	/** The node at a path of the files layout (`/` being the archive's root). */
	nodeAt: (path: string) => ZipNode | undefined
}

export const [injectArchiveViewer, provideArchiveViewer] = createContext<ArchiveViewerContext>(
	'ArchiveWorkbench',
	'archiveViewer',
)
