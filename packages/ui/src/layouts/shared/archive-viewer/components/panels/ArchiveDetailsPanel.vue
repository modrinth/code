<template>
	<div class="flex flex-col gap-3 p-3">
		<div class="flex min-w-0 items-center gap-2">
			<PackageIcon class="size-5 shrink-0 text-secondary" aria-hidden="true" />
			<h3 class="m-0 truncate text-base font-semibold text-contrast" :title="title">{{ title }}</h3>
		</div>
		<DetailsList :rows="rows" />
	</div>
</template>

<script setup lang="ts">
import { PackageIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { useFormatBytes } from '#ui/composables/format-bytes'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import {
	type FileZipTreeNode,
	isZipArchive,
	type RootZipTreeNode,
	type ZipNode,
} from '#ui/helpers/toolbox/zip'
import { injectFileBrowserUI } from '#ui/layouts/shared/files-tab/providers/file-browser-ui'
import { getFileExtension } from '#ui/utils/file-extensions'

import { useNodeLazyValue } from '../../composables/node-values'
import { injectArchiveViewer } from '../../providers/archive-viewer'
import type { ArchiveDetailRow } from '../../types'
import DetailsList from './DetailsList.vue'

const { formatMessage } = useVIntl()
const formatBytes = useFormatBytes()

const messages = defineMessages({
	source: { id: 'archive-viewer.archive-details.source', defaultMessage: 'Source' },
	path: { id: 'archive-viewer.archive-details.path', defaultMessage: 'Path' },
	inside: { id: 'archive-viewer.archive-details.inside', defaultMessage: 'Inside' },
	size: { id: 'archive-viewer.archive-details.size', defaultMessage: 'Size' },
	compressedSize: {
		id: 'archive-viewer.archive-details.compressed-size',
		defaultMessage: 'Compressed size',
	},
	contentSize: {
		id: 'archive-viewer.archive-details.content-size',
		defaultMessage: 'Uncompressed contents',
	},
	files: { id: 'archive-viewer.archive-details.files', defaultMessage: 'Files' },
	folders: { id: 'archive-viewer.archive-details.folders', defaultMessage: 'Folders' },
	classFiles: { id: 'archive-viewer.archive-details.class-files', defaultMessage: 'Class files' },
	nestedArchives: {
		id: 'archive-viewer.archive-details.nested-archives',
		defaultMessage: 'Nested archives',
	},
	sha1: { id: 'archive-viewer.archive-details.sha1', defaultMessage: 'SHA-1' },
	sha512: { id: 'archive-viewer.archive-details.sha512', defaultMessage: 'SHA-512' },
	hashPending: {
		id: 'archive-viewer.archive-details.hash-pending',
		defaultMessage: 'Available once fully downloaded',
	},
	status: { id: 'archive-viewer.archive-details.status', defaultMessage: 'Status' },
	statusReading: {
		id: 'archive-viewer.archive-details.status-reading',
		defaultMessage: 'Reading nested archives...',
	},
	statusPending: {
		id: 'archive-viewer.archive-details.status-pending',
		defaultMessage: 'Not read yet',
	},
	statusRead: { id: 'archive-viewer.archive-details.status-read', defaultMessage: 'Fully read' },
})

const ui = injectFileBrowserUI()
const archive = injectArchiveViewer()

/**
 * The archive the active entry is stored in. A nested archive that is itself open (browsed as a
 * folder) is described rather than the archive holding it.
 */
const target = computed<RootZipTreeNode | FileZipTreeNode>(() => {
	void archive.treeVersion.value
	const node = archive.nodeAt(ui.activeLocation.value.path)
	if (!node) return archive.root
	if (node.type === 'file' && isZipArchive(node)) return node as FileZipTreeNode
	return node.getArchive()
})

const nested = computed(() => (target.value === archive.root ? undefined : (target.value as FileZipTreeNode)))

const nestedSha1 = useNodeLazyValue(() => nested.value, 'sha1')
const nestedSha512 = useNodeLazyValue(() => nested.value, 'sha512')

const title = computed(() => nested.value?.name ?? archive.source.name)

interface ArchiveSummary {
	files: number
	folders: number
	classFiles: number
	nestedArchives: number
	size: number
}

/** Counts what the archive holds itself, leaving out the contents of archives nested within it. */
function summarize(node: ZipNode, summary: ArchiveSummary = {
	files: 0,
	folders: 0,
	classFiles: 0,
	nestedArchives: 0,
	size: 0,
}): ArchiveSummary {
	for (const child of node.children.values()) {
		if (child.type === 'directory') {
			summary.folders++
			summarize(child, summary)
			continue
		}
		summary.files++
		summary.size += child.metadata.uncompressedSize ?? 0
		if (isZipArchive(child)) summary.nestedArchives++
		else if (getFileExtension(child.name) === 'class') summary.classFiles++
	}
	return summary
}

const summary = computed(() => {
	void archive.treeVersion.value
	return summarize(target.value)
})

const status = computed(() => {
	if (archive.fullyLoaded.value) return formatMessage(messages.statusRead)
	if (nested.value && nested.value.children.size === 0) return formatMessage(messages.statusPending)
	return formatMessage(messages.statusReading)
})

const rows = computed<ArchiveDetailRow[]>(() => {
	const counts = summary.value
	const contents: ArchiveDetailRow[] = [
		{ id: 'files', label: formatMessage(messages.files), value: String(counts.files) },
		{ id: 'folders', label: formatMessage(messages.folders), value: String(counts.folders) },
		{ id: 'class-files', label: formatMessage(messages.classFiles), value: String(counts.classFiles) },
		{
			id: 'nested-archives',
			label: formatMessage(messages.nestedArchives),
			value: String(counts.nestedArchives),
		},
		{ id: 'content-size', label: formatMessage(messages.contentSize), value: formatBytes(counts.size) },
		{ id: 'status', label: formatMessage(messages.status), value: status.value },
	]

	const inner = nested.value
	if (!inner) {
		const { source } = archive
		const hash = archive.sha512Hash.value
		return [
			{ id: 'source', label: formatMessage(messages.source), value: source.url, href: source.url },
			{
				id: 'size',
				label: formatMessage(messages.size),
				value: source.size != null ? formatBytes(source.size) : null,
			},
			{
				id: 'sha512',
				label: formatMessage(messages.sha512),
				value: hash ?? formatMessage(messages.hashPending),
				hash: hash != null,
			},
			...contents,
		]
	}

	const parent = inner.getArchive()
	return [
		{ id: 'path', label: formatMessage(messages.path), value: inner.path.replace(/^\/+/, '') },
		{
			id: 'inside',
			label: formatMessage(messages.inside),
			value: parent === archive.root ? archive.source.name : parent.name,
		},
		{ id: 'size', label: formatMessage(messages.size), value: formatBytes(inner.metadata.uncompressedSize) },
		{
			id: 'compressed-size',
			label: formatMessage(messages.compressedSize),
			value: formatBytes(inner.metadata.compressedSize),
		},
		{
			id: 'sha1',
			label: formatMessage(messages.sha1),
			value: nestedSha1.value.value,
			loading: nestedSha1.loading.value,
			hash: true,
		},
		{
			id: 'sha512',
			label: formatMessage(messages.sha512),
			value: nestedSha512.value.value,
			loading: nestedSha512.loading.value,
			hash: true,
		},
		...contents,
	]
})
</script>
