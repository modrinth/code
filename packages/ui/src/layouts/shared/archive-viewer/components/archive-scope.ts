import { defineComponent, onBeforeUnmount, type PropType } from 'vue'

import type { RootZipTreeNode, ZipHandler } from '#ui/helpers/toolbox/zip'

import { useArchiveFileManager } from '../composables/archive-file-manager'
import { injectArchiveSession, type OpenArchive } from '../providers/archive-session'
import type { ArchiveSource } from '../types'

/**
 * Sets up browsing one archive and hands it to the surrounding session for as long as it's
 * mounted. Keyed by the archive, so opening another one replaces it. Renders nothing.
 */
export default defineComponent({
	name: 'ArchiveScope',
	props: {
		root: { type: Object as PropType<RootZipTreeNode>, required: true },
		handler: { type: Object as PropType<ZipHandler>, required: true },
		source: { type: Object as PropType<ArchiveSource>, required: true },
	},
	setup(props) {
		const session = injectArchiveSession()
		const { context, location, open, fullyLoaded, treeVersion, sha512Hash, archiveHash, nodeAt } =
			useArchiveFileManager({
				root: props.root,
				handler: props.handler,
				source: props.source,
				initialPath: session.initialPath(),
			})

		const archive: OpenArchive = {
			root: props.root,
			handler: props.handler,
			source: props.source,
			manager: context,
			location,
			open,
			fullyLoaded,
			treeVersion,
			sha512Hash,
			archiveHash,
			nodeAt,
		}
		session.attach(archive)
		onBeforeUnmount(() => session.detach(archive))

		return () => null
	},
})
