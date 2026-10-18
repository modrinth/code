<template>
	<ul
		class="m-0 flex list-none flex-wrap gap-2 p-0 text-sm"
		:aria-label="formatMessage(messages.contentsCounts)"
	>
		<li
			v-for="entry in entries"
			:key="entry.id"
			v-tooltip="entry.label"
			class="flex items-center gap-1.5 rounded-lg border border-solid border-surface-4 bg-surface-2 px-2 py-1 tabular-nums text-primary"
		>
			<component :is="entry.icon" class="size-4 shrink-0" :class="entry.iconClass" aria-hidden="true" />
			<span aria-hidden="true">{{ entry.count }}</span>
			<span class="sr-only">{{ entry.label }}</span>
		</li>
	</ul>
</template>

<script setup lang="ts">
import { CoffeeIcon, FileIcon, FolderArchiveIcon, FolderOpenIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { useVIntl } from '#ui/composables/i18n'

import { sectionMessages as messages } from './messages'
import type { ArchiveCounts } from '../../types'

/** How many of each kind of entry an archive holds, as icons and counts on one line. */
const props = defineProps<{
	counts: ArchiveCounts
}>()

const { formatMessage } = useVIntl()

const entries = computed(() => [
	{
		id: 'files',
		icon: FileIcon,
		iconClass: 'text-secondary',
		count: props.counts.files,
		label: formatMessage(messages.filesCount, { count: props.counts.files }),
	},
	{
		id: 'folders',
		icon: FolderOpenIcon,
		iconClass: 'text-amber-400',
		count: props.counts.folders,
		label: formatMessage(messages.foldersCount, { count: props.counts.folders }),
	},
	{
		id: 'class-files',
		icon: CoffeeIcon,
		iconClass: 'text-orange',
		count: props.counts.classFiles,
		label: formatMessage(messages.classFilesCount, { count: props.counts.classFiles }),
	},
	{
		id: 'nested-archives',
		icon: FolderArchiveIcon,
		iconClass: 'text-orange',
		count: props.counts.nestedArchives,
		label: formatMessage(messages.nestedArchivesCount, { count: props.counts.nestedArchives }),
	},
])
</script>
