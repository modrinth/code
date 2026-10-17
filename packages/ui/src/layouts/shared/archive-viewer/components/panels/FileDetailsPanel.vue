<template>
	<div class="flex flex-col gap-3 p-3">
		<h3 class="m-0 truncate text-base font-semibold text-contrast" :title="node?.name">
			{{ node === archive.root ? archive.source.name : (node?.name ?? '-') }}
		</h3>
		<DetailsList :rows="rows" />
	</div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import { useFormatBytes } from '#ui/composables/format-bytes'
import { useFormatDateTime } from '#ui/composables/format-date-time'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { isZipArchive, type ZipNode } from '#ui/helpers/toolbox/zip'
import { injectFileBrowserUI } from '#ui/layouts/shared/files-tab/providers/file-browser-ui'

import { useNodeLazyValue } from '../../composables/node-values'
import { injectArchiveViewer } from '../../providers/archive-viewer'
import type { ArchiveDetailRow } from '../../types'
import DetailsList from './DetailsList.vue'

const { formatMessage } = useVIntl()
const formatBytes = useFormatBytes()
const formatDate = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })

const messages = defineMessages({
	path: { id: 'archive-viewer.file-details.path', defaultMessage: 'Path' },
	kind: { id: 'archive-viewer.file-details.kind', defaultMessage: 'Kind' },
	kindFile: { id: 'archive-viewer.file-details.kind-file', defaultMessage: 'File' },
	kindFolder: { id: 'archive-viewer.file-details.kind-folder', defaultMessage: 'Folder' },
	kindArchive: { id: 'archive-viewer.file-details.kind-archive', defaultMessage: 'Nested archive' },
	entries: { id: 'archive-viewer.file-details.entries', defaultMessage: 'Entries' },
	size: { id: 'archive-viewer.file-details.size', defaultMessage: 'Size' },
	compressedSize: {
		id: 'archive-viewer.file-details.compressed-size',
		defaultMessage: 'Compressed size',
	},
	compressedRatio: {
		id: 'archive-viewer.file-details.compressed-ratio',
		defaultMessage: '{size} ({ratio, number, percent})',
	},
	modified: { id: 'archive-viewer.file-details.modified', defaultMessage: 'Modified' },
	detectedType: {
		id: 'archive-viewer.file-details.detected-type',
		defaultMessage: 'Detected type',
	},
	compression: { id: 'archive-viewer.file-details.compression', defaultMessage: 'Compression' },
	compressionStored: {
		id: 'archive-viewer.file-details.compression-stored',
		defaultMessage: 'Stored',
	},
	compressionDeflate: {
		id: 'archive-viewer.file-details.compression-deflate',
		defaultMessage: 'Deflate',
	},
	compressionOther: {
		id: 'archive-viewer.file-details.compression-other',
		defaultMessage: 'Method {method}',
	},
	crc32: { id: 'archive-viewer.file-details.crc32', defaultMessage: 'CRC-32' },
	encrypted: { id: 'archive-viewer.file-details.encrypted', defaultMessage: 'Encrypted' },
	yes: { id: 'archive-viewer.file-details.yes', defaultMessage: 'Yes' },
	no: { id: 'archive-viewer.file-details.no', defaultMessage: 'No' },
	comment: { id: 'archive-viewer.file-details.comment', defaultMessage: 'Comment' },
	sha1: { id: 'archive-viewer.file-details.sha1', defaultMessage: 'SHA-1' },
	sha256: { id: 'archive-viewer.file-details.sha256', defaultMessage: 'SHA-256' },
	sha512: { id: 'archive-viewer.file-details.sha512', defaultMessage: 'SHA-512' },
})

const ui = injectFileBrowserUI()
const archive = injectArchiveViewer()

const node = computed<ZipNode | undefined>(() => {
	void archive.treeVersion.value
	return archive.nodeAt(ui.activeLocation.value.path)
})
const file = computed(() => node.value?.asFile())

const fileType = useNodeLazyValue(() => file.value, 'fileTypeInfo')
const sha1 = useNodeLazyValue(() => file.value, 'sha1')
const sha256 = useNodeLazyValue(() => file.value, 'sha256')
const sha512 = useNodeLazyValue(() => file.value, 'sha512')

function compressionName(method: number | undefined) {
	if (method == null) return null
	if (method === 0) return formatMessage(messages.compressionStored)
	if (method === 8) return formatMessage(messages.compressionDeflate)
	return formatMessage(messages.compressionOther, { method })
}

const rows = computed<ArchiveDetailRow[]>(() => {
	const current = node.value
	if (!current) return []

	const path = current.path.replace(/^\/+/, '')
	const shared: ArchiveDetailRow[] = [
		{ id: 'path', label: formatMessage(messages.path), value: path || '/' },
	]

	const target = file.value
	if (!target) {
		const modified = current.metadata.lastModDate
		return [
			...shared,
			{ id: 'kind', label: formatMessage(messages.kind), value: formatMessage(messages.kindFolder) },
			{ id: 'entries', label: formatMessage(messages.entries), value: String(current.children.size) },
			...(modified
				? [{ id: 'modified', label: formatMessage(messages.modified), value: formatDate(modified) }]
				: []),
		]
	}

	const { metadata } = target
	const ratio =
		metadata.uncompressedSize > 0 ? metadata.compressedSize / metadata.uncompressedSize : null

	return [
		...shared,
		{
			id: 'kind',
			label: formatMessage(messages.kind),
			value: formatMessage(isZipArchive(target) ? messages.kindArchive : messages.kindFile),
		},
		{ id: 'size', label: formatMessage(messages.size), value: formatBytes(metadata.uncompressedSize) },
		{
			id: 'compressed-size',
			label: formatMessage(messages.compressedSize),
			value:
				ratio == null
					? formatBytes(metadata.compressedSize)
					: formatMessage(messages.compressedRatio, {
							size: formatBytes(metadata.compressedSize),
							ratio,
						}),
		},
		{
			id: 'modified',
			label: formatMessage(messages.modified),
			value: metadata.lastModDate ? formatDate(metadata.lastModDate) : null,
		},
		{
			id: 'detected-type',
			label: formatMessage(messages.detectedType),
			value: fileType.value.value?.mime ?? null,
			loading: fileType.loading.value,
		},
		{
			id: 'compression',
			label: formatMessage(messages.compression),
			value: compressionName(metadata.compressionMethod),
		},
		{
			id: 'crc32',
			label: formatMessage(messages.crc32),
			value: metadata.signature != null ? metadata.signature.toString(16).padStart(8, '0') : null,
			copyable: true,
		},
		{
			id: 'encrypted',
			label: formatMessage(messages.encrypted),
			value: formatMessage(metadata.encrypted ? messages.yes : messages.no),
		},
		...(metadata.comment
			? [{ id: 'comment', label: formatMessage(messages.comment), value: metadata.comment }]
			: []),
		{
			id: 'sha1',
			label: formatMessage(messages.sha1),
			value: sha1.value.value,
			loading: sha1.loading.value,
			hash: true,
		},
		{
			id: 'sha256',
			label: formatMessage(messages.sha256),
			value: sha256.value.value,
			loading: sha256.loading.value,
			hash: true,
		},
		{
			id: 'sha512',
			label: formatMessage(messages.sha512),
			value: sha512.value.value,
			loading: sha512.loading.value,
			hash: true,
		},
	]
})
</script>
