<script setup lang="ts">
import {
	defineMessages,
	DropdownFilterBar,
	type DropdownFilterBarCategory,
	formatLoader,
	useVIntl,
} from '@modrinth/ui'
import { computed } from 'vue'

import { useLibrary } from '@/components/ui/library/use-library'

const { filters, instances } = useLibrary()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	instanceType: { id: 'app.library.filter.instance-type', defaultMessage: 'Instance type' },
	modpack: { id: 'app.library.filter.instance-type.modpack', defaultMessage: 'Modpack' },
	server: { id: 'app.library.filter.instance-type.server', defaultMessage: 'Server' },
	custom: { id: 'app.library.filter.instance-type.custom', defaultMessage: 'Custom' },
	gameVersion: { id: 'app.library.filter.game-version', defaultMessage: 'Game version' },
	loader: { id: 'app.library.filter.loader', defaultMessage: 'Loader' },
	filterBy: { id: 'app.library.filter.label', defaultMessage: 'Filter by' },
	addFilter: { id: 'app.library.filter.add', defaultMessage: 'Add filter' },
	clearFilters: { id: 'app.library.filter.clear', defaultMessage: 'Clear filters' },
})

const filterCategories = computed<DropdownFilterBarCategory[]>(() => [
	{
		key: 'instanceType',
		label: formatMessage(messages.instanceType),
		options: [
			{ value: 'modpack', label: formatMessage(messages.modpack) },
			{ value: 'server', label: formatMessage(messages.server) },
			{ value: 'custom', label: formatMessage(messages.custom) },
		],
	},
	{
		key: 'gameVersion',
		label: formatMessage(messages.gameVersion),
		searchable: true,
		options: [...new Set(instances.value.map((instance) => instance.game_version))]
			.sort((a, b) => b.localeCompare(a, undefined, { numeric: true }))
			.map((version) => ({ value: version, label: version })),
	},
	{
		key: 'loader',
		label: formatMessage(messages.loader),
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
		:label="formatMessage(messages.filterBy)"
		:add-label="formatMessage(messages.addFilter)"
		:clear-label="formatMessage(messages.clearFilters)"
		apply-immediately
		checkbox-position="right"
	/>
</template>
