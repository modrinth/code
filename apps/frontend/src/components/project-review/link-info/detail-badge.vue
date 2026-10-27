<template>
	<TagItem
		v-tooltip="tooltip"
		:aria-label="tooltip"
		tabindex="0"
		class="max-w-full !text-xs"
		:class="
			entry.warning ? '!border-orange-highlight !bg-orange-highlight !text-orange' : '!text-primary'
		"
	>
		<component :is="detailIcon(entry)" aria-hidden="true" />
		<time
			v-if="entry.format === 'date' && typeof entry.value === 'string'"
			:datetime="entry.value"
			class="min-w-0 whitespace-normal break-words"
			>{{ value }}</time
		>
		<span v-else class="min-w-0 whitespace-normal break-words">{{ value }}</span>
	</TagItem>
</template>

<script setup lang="ts">
import { TagItem } from '@modrinth/ui'
import { computed } from 'vue'

import {
	createDateFormatter,
	createRelativeTimeFormatter,
	formatBytes,
	formatNumber,
} from '../format'
import { detailIcon } from './detail-icons'
import { linkInfoLabels } from './labels'
import type { Detail } from './types'

const props = defineProps<{ entry: Detail }>()

const relativeTime = createRelativeTimeFormatter({ numeric: 'always' })
const formatDateTime = createDateFormatter({
	dateStyle: 'long',
	timeStyle: 'short',
})

const value = computed(() => {
	const entry = props.entry
	if (typeof entry.value === 'object') return linkInfoLabels[entry.value.message]
	if (entry.format === 'date') return relativeTime(entry.value as string) || String(entry.value)
	if (entry.format === 'bytes' && typeof entry.value === 'number') return formatBytes(entry.value)
	if (typeof entry.value === 'number') return formatNumber(entry.value)
	if (typeof entry.value === 'boolean') return entry.value ? 'Enabled' : 'Disabled'
	return entry.value
})

const tooltip = computed(() => {
	const entry = props.entry
	const date =
		entry.format === 'date' && typeof entry.value === 'string' ? new Date(entry.value) : undefined
	const description = `${linkInfoLabels[entry.label]}: ${date && !Number.isNaN(date.getTime()) ? formatDateTime(date) : value.value}`
	return entry.warning ? `${description}\n${linkInfoLabels[entry.warning]}` : description
})
</script>
