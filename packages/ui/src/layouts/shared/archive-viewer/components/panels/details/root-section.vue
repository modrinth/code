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
		:hash="archive.archiveHash"
		details-key="root"
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
import DetailsEntry from '../../details-section/index.vue'
import { useContentsDetails } from './use-contents-details'
import { useSizeRow } from './use-size-row'

/** The archive that was opened: where it came from, its size and hash, and what it holds. */
const { formatMessage } = useVIntl()
const sizeRow = useSizeRow()
const archive = injectArchiveViewer()

const contents = useContentsDetails(() => archive.root)

/**
 * Where the archive came from, and whether its hash matches the one it was listed with (e.g. by
 * the project version it was published in), once all of it has been downloaded.
 */
const rows = computed<ArchiveDetailRow[]>(() => {
	const known = archive.sha512Hash.value
	const listed = archive.source.sha512Hash
	const result: ArchiveDetailRow[] = [
		{
			id: 'source',
			label: formatMessage(messages.source),
			value: archive.source.url,
			hash: true,
			display: formatMessage(messages.sourceName, { host: sourceHost(archive.source.url) }),
		},
	]
	if (listed) {
		const matches = known ? known === listed : null
		result.push({
			id: 'listed-hash',
			label: formatMessage(messages.listedHash),
			value: formatMessage(
				matches == null
					? messages.listedHashPending
					: matches
						? messages.listedHashMatch
						: messages.listedHashMismatch,
			),
			tone: matches == null ? undefined : matches ? 'green' : 'red',
			tooltip: formatMessage(messages.listedHashValue, { hash: listed }),
		})
	}
	return result
})

/** The downloaded archive against what it holds once uncompressed. */
const sizes = computed(() => [sizeRow(archive.source.size, contents.value.size)])

</script>
