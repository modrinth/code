<script setup lang="ts">
import { ChevronDownIcon, ChevronUpIcon } from '@modrinth/assets'
import { computed } from 'vue'

import type { ContentCardTableSortColumn, ContentCardTableSortDirection } from '../../types'

const props = defineProps<{
	column: ContentCardTableSortColumn
	label: string
	sortable: boolean
	sortBy?: ContentCardTableSortColumn
	sortDirection: ContentCardTableSortDirection
}>()

const emit = defineEmits<{
	sort: [column: ContentCardTableSortColumn]
}>()

const active = computed(() => props.sortBy === props.column)
const ariaSort = computed(() => {
	if (!active.value) return 'none'
	return props.sortDirection === 'asc' ? 'ascending' : 'descending'
})
</script>

<template>
	<button
		v-if="sortable"
		role="columnheader"
		:aria-sort="ariaSort"
		class="flex items-center gap-1.5 font-semibold text-secondary"
		@click="emit('sort', column)"
	>
		{{ label }}
		<ChevronUpIcon v-if="active && sortDirection === 'asc'" class="size-4" />
		<ChevronDownIcon v-else-if="active && sortDirection === 'desc'" class="size-4" />
	</button>
	<span v-else role="columnheader" class="font-semibold text-secondary">{{ label }}</span>
</template>
