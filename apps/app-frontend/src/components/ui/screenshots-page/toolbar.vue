<script setup lang="ts">
import { ArrowUpDownIcon, LayoutGridIcon, SearchIcon, SquarePlusIcon } from '@modrinth/assets'
import {
	Button,
	Combobox,
	type ComboboxOption,
	defineMessages,
	DropdownFilterBar,
	type DropdownFilterBarCategory,
	formatLoader,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
import { computed } from 'vue'

import type { GameInstance } from '@/helpers/types'

const search = defineModel<string>('search', { required: true })
const sort = defineModel<string>('sort', { required: true })
const group = defineModel<string>('group', { required: true })
const filters = defineModel<Record<'loader' | 'gameVersion' | 'modpack', string[]>>('filters', {
	required: true,
})

const props = defineProps<{
	sortOptions: ComboboxOption<string>[]
	groupOptions: ComboboxOption<string>[]
	instances: GameInstance[]
	modpackOptions: ComboboxOption<string>[]
}>()

const emit = defineEmits<{
	(e: 'new-group'): void
}>()

const { formatMessage } = useVIntl()
const messages = defineMessages({
	search: { id: 'app.screenshots.search', defaultMessage: 'Search' },
	newGroup: { id: 'app.screenshots.group.new', defaultMessage: 'New group' },
	sortBy: { id: 'app.screenshots.sort-by', defaultMessage: 'Sort by' },
	groupBy: { id: 'app.screenshots.group-by', defaultMessage: 'Group by' },
	loader: { id: 'app.screenshots.filter.loader', defaultMessage: 'Loader' },
	gameVersion: { id: 'app.screenshots.filter.game-version', defaultMessage: 'Game version' },
	modpack: { id: 'app.screenshots.filter.modpack', defaultMessage: 'Modpack' },
	filterBy: { id: 'app.screenshots.filter.label', defaultMessage: 'Filter by' },
	addFilter: { id: 'app.screenshots.filter.add', defaultMessage: 'Add filter' },
	clearFilters: { id: 'app.screenshots.filter.clear', defaultMessage: 'Clear filters' },
})

const filterCategories = computed<DropdownFilterBarCategory[]>(() => [
	{
		key: 'loader',
		label: formatMessage(messages.loader),
		options: [...new Set(props.instances.map((instance) => instance.loader))]
			.map((loader) => ({
				value: loader,
				label: formatLoader(formatMessage, loader),
			}))
			.sort((a, b) => a.label.localeCompare(b.label)),
	},
	{
		key: 'gameVersion',
		label: formatMessage(messages.gameVersion),
		searchable: true,
		options: [...new Set(props.instances.map((instance) => instance.game_version))]
			.sort((a, b) => b.localeCompare(a, undefined, { numeric: true }))
			.map((version) => ({ value: version, label: version })),
	},
	...(props.modpackOptions.length > 0
		? [
				{
					key: 'modpack',
					label: formatMessage(messages.modpack),
					searchable: true,
					options: props.modpackOptions,
				},
			]
		: []),
])
</script>

<template>
	<div class="flex flex-col gap-2">
		<div class="flex flex-wrap gap-2">
			<StyledInput
				v-model="search"
				:icon="SearchIcon"
				type="text"
				:placeholder="formatMessage(messages.search)"
				clearable
				wrapper-class="min-w-[16rem] flex-1"
			/>
			<Button @click="emit('new-group')">
				<SquarePlusIcon />
				{{ formatMessage(messages.newGroup) }}
			</Button>
		</div>
		<div class="flex flex-wrap items-center gap-2">
			<Combobox
				v-model="sort"
				class="w-max"
				:options="sortOptions"
				:show-icon-in-selected="false"
				dropdown-min-width="160px"
			>
				<template #prefix>
					<ArrowUpDownIcon
						class="size-5 text-primary"
						:aria-label="formatMessage(messages.sortBy)"
					/>
				</template>
				<template #selected="{ label }">
					<span>{{ label }}</span>
				</template>
			</Combobox>
			<Combobox
				v-model="group"
				class="w-max"
				:options="groupOptions"
				:show-icon-in-selected="false"
				dropdown-min-width="160px"
			>
				<template #prefix>
					<LayoutGridIcon
						class="size-5 text-primary"
						:aria-label="formatMessage(messages.groupBy)"
					/>
				</template>
				<template #selected="{ label }">
					<span>{{ label }}</span>
				</template>
			</Combobox>
			<div class="mx-2 h-6 w-px bg-surface-5" />
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
		</div>
	</div>
</template>
