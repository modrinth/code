<template>
	<DetailsEntry
		:kind="formatMessage(messages.rootArchive)"
		:title="archive.source.name"
		:icon="FolderArchiveIcon"
		icon-class="text-orange"
		:status="contents.status"
		:rows="rows"
		:sizes="sizes"
		:counts="contents.counts"
		:hashes="hashes"
	/>
</template>

<script setup lang="ts">
import { FolderArchiveIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { useVIntl } from '#ui/composables/i18n'

import { injectArchiveViewer } from '../../../providers/archive-viewer'
import type { ArchiveDetailRow } from '../../../types'
import { sourceHost } from './entry-values'
import { detailsMessages as messages } from './messages'
import DetailsEntry from './entry.vue'
import { useContentsDetails } from './use-contents-details'
import { useSizeRow } from './use-size-row'

/** The archive that was opened: where it came from, its size and hash, and what it holds. */
const { formatMessage } = useVIntl()
const sizeRow = useSizeRow()
const archive = injectArchiveViewer()

const contents = useContentsDetails(() => archive.root)

const rows = computed<ArchiveDetailRow[]>(() => [
	{
		id: 'source',
		label: formatMessage(messages.source),
		value: archive.source.url,
		hash: true,
		display: formatMessage(messages.sourceName, { host: sourceHost(archive.source.url) }),
	},
])

/** The downloaded archive against what it holds once uncompressed. */
const sizes = computed(() => [sizeRow(archive.source.size, contents.value.size)])

/** The archive's hashes, which are only known once all of it has been downloaded. */
const hashes = computed<ArchiveDetailRow[]>(() => {
	const known = archive.rootHashes.value
	if (!known) {
		return [
			{
				id: 'hashes',
				label: formatMessage(messages.hashes),
				value: formatMessage(messages.hashPending),
			},
		]
	}
	return [
		{ id: 'sha1', label: formatMessage(messages.sha1), value: known.sha1, hash: true },
		{ id: 'sha256', label: formatMessage(messages.sha256), value: known.sha256, hash: true },
		{ id: 'sha512', label: formatMessage(messages.sha512), value: known.sha512, hash: true },
		{ id: 'murmur2', label: formatMessage(messages.murmur2), value: known.murmur2, hash: true },
	]
})
</script>
