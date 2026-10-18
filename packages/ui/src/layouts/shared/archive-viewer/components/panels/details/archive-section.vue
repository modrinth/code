<template>
	<DetailsEntry
		:kind="formatMessage(messages.nestedArchive, { depth })"
		:title="node.name"
		:icon="FolderArchiveIcon"
		icon-class="text-orange"
		:path="node.path.replace(/^\/+/, '')"
		:status="contents.status"
		:rows="entry.rows"
		:more="entry.more"
		:sizes="sizes"
		:counts="contents.counts"
		:hash="entry.hash"
		details-key="nested"
	/>
</template>

<script setup lang="ts">
import { FolderArchiveIcon } from '@modrinth/assets'

import { computed } from 'vue'

import { useVIntl } from '#ui/composables/i18n'
import type { FileZipTreeNode } from '#ui/helpers/toolbox/zip'

import { detailsMessages as messages } from './messages'
import DetailsEntry from '../../details-section/index.vue'
import { useContentsDetails } from './use-contents-details'
import { useEntryDetails } from './use-entry-details'
import { useSizeRow } from './use-size-row'

/**
 * An archive nested in another: how deep it sits, how it's stored as an entry of the archive
 * holding it, and what it holds itself.
 */
const props = defineProps<{
	node: FileZipTreeNode
	/** How many archives deep it is: 1 for one stored directly in the opened archive. */
	depth: number
}>()

const { formatMessage } = useVIntl()

const entry = useEntryDetails(() => props.node)
const contents = useContentsDetails(() => props.node)
const sizeRow = useSizeRow()

const sizes = computed(() => [
	sizeRow(entry.value.stored?.compressed, entry.value.stored?.size, contents.value.size),
])
</script>
