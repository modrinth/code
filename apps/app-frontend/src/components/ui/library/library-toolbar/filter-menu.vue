<script setup lang="ts">
import {
	DropdownFilterBar,
	type DropdownFilterBarCategory,
	formatLoader,
	useVIntl,
} from '@modrinth/ui'
import { computed } from 'vue'

import { useLibrary } from '@/components/ui/library/use-library'

const { filters, instances } = useLibrary()
const { formatMessage } = useVIntl()

const filterCategories = computed<DropdownFilterBarCategory[]>(() => [
	{
		key: 'instanceType',
		label: 'Instance type',
		options: [
			{ value: 'modpack', label: 'Modpack' },
			{ value: 'server', label: 'Server' },
			{ value: 'custom', label: 'Custom' },
		],
	},
	{
		key: 'gameVersion',
		label: 'Game version',
		searchable: true,
		options: [...new Set(instances.value.map((instance) => instance.game_version))]
			.sort((a, b) => b.localeCompare(a, undefined, { numeric: true }))
			.map((version) => ({ value: version, label: version })),
	},
	{
		key: 'loader',
		label: 'Loader',
		options: [...new Set(instances.value.map((instance) => instance.loader))]
			.map((loader) => ({
				value: loader,
				label: formatLoader(formatMessage, loader),
			}))
			.sort((a, b) => a.label.localeCompare(b.label)),
	},
])
</script>

<template>
	<DropdownFilterBar
		v-model="filters"
		:categories="filterCategories"
		use-filter-icon
		label="Filter by"
		add-label="Add filter"
		clear-label="Clear filters"
		apply-immediately
		checkbox-position="right"
	/>
</template>
