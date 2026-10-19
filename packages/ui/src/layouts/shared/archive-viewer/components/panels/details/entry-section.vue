<template>
	<DetailsEntry
		:kind="formatMessage(file ? messages.file : messages.folder)"
		:title="node.name"
		:icon="iconStyle.icon"
		:icon-class="iconStyle.color"
		:path="node.path.replace(/^\/+/, '') || '/'"
		:rows="rows"
		:more="file ? entry.more : undefined"
		:sizes="sizes"
		:hash="file ? entry.hash : undefined"
		details-key="entry"
	/>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import { useFormatDateTime } from '#ui/composables/format-date-time'
import { useVIntl } from '#ui/composables/i18n'
import type { ZipNode } from '#ui/helpers/toolbox/zip'
import { injectFileBrowserUI } from '#ui/layouts/shared/files-tab/providers/file-browser-ui'
import { fileIconFor } from '#ui/layouts/shared/files-tab/utils'

import type { ArchiveDetailRow } from '../../../types'
import { isPlaceholderDate } from './entry-values'
import { detailsMessages as messages } from './messages'
import DetailsEntry from '../../details-section/index.vue'
import { useEntryDetails } from './use-entry-details'
import { useSizeRow } from './use-size-row'

/** The file or folder being viewed, as stored in the archive holding it. */
const props = defineProps<{ node: ZipNode }>()

const { formatMessage } = useVIntl()
const formatDate = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })
const ui = injectFileBrowserUI()

const file = computed(() => props.node.asFile())
const entry = useEntryDetails(() => file.value)
const sizeRow = useSizeRow()

const sizes = computed(() => {
	const stored = entry.value.stored
	return stored ? [sizeRow(stored.compressed, stored.size)] : undefined
})

const iconStyle = computed(() => {
	const style = fileIconFor({ name: props.node.name, type: props.node.type })
	return ui.coloredIcons.value ? style : { ...style, color: 'text-secondary' }
})

const rows = computed<ArchiveDetailRow[]>(() => {
	if (file.value) return entry.value.rows

	const modified = props.node.metadata.lastModDate
	return [
		{
			id: 'entries',
			label: formatMessage(messages.entries),
			value: String(props.node.children.size),
		},
		...(isPlaceholderDate(modified)
			? []
			: [
					{ id: 'modified', label: formatMessage(messages.modified), value: formatDate(modified!) },
				]),
	]
})
</script>
