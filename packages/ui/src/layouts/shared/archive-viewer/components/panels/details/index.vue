<template>
	<div class="flex flex-col divide-x-0 divide-y divide-solid divide-surface-4">
		<slot name="source" />
		<RootSection />
		<div v-if="chain.length > 1" class="px-2 py-1">
			<Button type="quiet" size="sm" :aria-expanded="expanded" @click="expanded = !expanded">
				<ChevronUpIcon v-if="expanded" aria-hidden="true" />
				<ChevronDownIcon v-else aria-hidden="true" />
				{{
					expanded
						? formatMessage(messages.hideParents)
						: formatMessage(messages.showParents, { count: chain.length - 1 })
				}}
			</Button>
		</div>
		<ArchiveSection
			v-for="(nested, index) in shownChain"
			:key="nested.path"
			:node="nested"
			:depth="chain.length - shownChain.length + index + 1"
		/>
		<EntrySection v-if="entry" :key="entry.path" :node="entry" />
	</div>
</template>

<script setup lang="ts">
import { ChevronDownIcon, ChevronUpIcon } from '@modrinth/assets'
import { computed, ref } from 'vue'

import { Button } from '#ui/components/base/buttons'
import { useVIntl } from '#ui/composables/i18n'
import { type FileZipTreeNode, isZipArchive, type ZipNode } from '#ui/helpers/toolbox/zip'
import { injectFileBrowserUI } from '#ui/layouts/shared/files-tab/providers/file-browser-ui'

import { injectArchiveViewer } from '../../../providers/archive-viewer'
import ArchiveSection from './archive-section.vue'
import EntrySection from './entry-section.vue'
import { detailsMessages as messages } from './messages'
import RootSection from './root-section.vue'

/**
 * Details of what's being viewed, stacked from the outside in: where the archive was published
 * (the `source` slot, e.g. a project's version, when the host knows), the opened archive, the
 * archives nested within it down to the one the entry sits in (or is), and the entry itself. Each
 * is only shown when it differs from the one above, so nothing is described twice, and the
 * archives between the opened one and the closest can be folded away.
 */
const { formatMessage } = useVIntl()
const ui = injectFileBrowserUI()
const archive = injectArchiveViewer()

const expanded = ref(false)

const node = computed<ZipNode | undefined>(() => {
	void archive.treeVersion.value
	return archive.nodeAt(ui.activeLocation.value.path)
})

/** The archive the entry is stored in, or the entry itself when it's an archive. */
const targetArchive = computed(() => {
	const current = node.value
	if (!current) return archive.root
	if (current.type === 'file' && isZipArchive(current)) return current as FileZipTreeNode
	return current.getArchive()
})

/** Every nested archive from the opened one's down to the target, outermost first. */
const chain = computed(() => {
	const archives: FileZipTreeNode[] = []
	for (
		let current = targetArchive.value;
		current !== archive.root;
		current = current.getArchive()
	) {
		archives.unshift(current as FileZipTreeNode)
	}
	return archives
})

const shownChain = computed(() => (expanded.value ? chain.value : chain.value.slice(-1)))

const entry = computed(() => {
	const current = node.value
	if (!current || current === archive.root || current === targetArchive.value) return undefined
	return current
})
</script>
