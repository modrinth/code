import { computed } from 'vue'

import { isZipArchive, type ZipNode } from '#ui/helpers/toolbox/zip'
import { getFileExtension } from '#ui/utils/file-extensions'

import { injectArchiveViewer } from '../../../providers/archive-viewer'
import type { ArchiveCounts, ArchiveReadStatus } from './types'

/** Counts what the archive holds itself, leaving out the contents of archives nested within it. */
function summarize(
	node: ZipNode,
	totals = { files: 0, folders: 0, classFiles: 0, nestedArchives: 0, size: 0 },
) {
	for (const child of node.children.values()) {
		if (child.type === 'directory') {
			totals.folders++
			summarize(child, totals)
			continue
		}
		totals.files++
		totals.size += child.metadata.uncompressedSize ?? 0
		if (isZipArchive(child)) totals.nestedArchives++
		else if (getFileExtension(child.name) === 'class') totals.classFiles++
	}
	return totals
}

/**
 * What an archive holds: how many of each kind of entry, how many bytes they hold uncompressed,
 * and how far it has been read, since nested archives are read in the background.
 */
export function useContentsDetails(archiveNode: () => ZipNode) {
	const archive = injectArchiveViewer()

	return computed<{ counts: ArchiveCounts; size: number; status: ArchiveReadStatus }>(
		() => {
			void archive.treeVersion.value
			const node = archiveNode()
			const { size, ...counts } = summarize(node)

			const status: ArchiveReadStatus = archive.fullyLoaded.value
				? 'read'
				: node !== archive.root && node.children.size === 0
					? 'pending'
					: 'reading'

			return {
				counts,
				size,
				status,
			}
		},
	)
}
