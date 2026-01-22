<script setup lang="ts">
import { ChevronDownIcon, ChevronUpIcon } from '@modrinth/assets'
import { computed } from 'vue'

import Checkbox from '../base/Checkbox.vue'
import ContentCardItem from './ContentCardItem.vue'
import type {
	ContentCardTableItem,
	ContentCardTableSortColumn,
	ContentCardTableSortDirection,
} from './types'

interface Props {
	items: ContentCardTableItem[]
	showSelection?: boolean
	sortable?: boolean
	sortBy?: ContentCardTableSortColumn
	sortDirection?: ContentCardTableSortDirection
	disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
	showSelection: false,
	sortable: false,
	sortBy: undefined,
	sortDirection: 'asc',
	disabled: false,
})

const selectedIds = defineModel<string[]>('selectedIds', { default: () => [] })

const emit = defineEmits<{
	'update:enabled': [id: string, value: boolean]
	delete: [id: string]
	update: [id: string]
	sort: [column: ContentCardTableSortColumn, direction: ContentCardTableSortDirection]
}>()

const allSelected = computed(() => {
	if (props.items.length === 0) return false
	return props.items.every((item) => selectedIds.value.includes(item.id))
})

const someSelected = computed(() => {
	return props.items.some((item) => selectedIds.value.includes(item.id)) && !allSelected.value
})

function toggleSelectAll() {
	if (allSelected.value) {
		selectedIds.value = []
	} else {
		selectedIds.value = props.items.map((item) => item.id)
	}
}

function toggleItemSelection(itemId: string, selected: boolean) {
	if (selected) {
		if (!selectedIds.value.includes(itemId)) {
			selectedIds.value = [...selectedIds.value, itemId]
		}
	} else {
		selectedIds.value = selectedIds.value.filter((id) => id !== itemId)
	}
}

function isItemSelected(itemId: string): boolean {
	return selectedIds.value.includes(itemId)
}

function handleSort(column: ContentCardTableSortColumn) {
	if (!props.sortable) return

	const newDirection: ContentCardTableSortDirection =
		props.sortBy === column && props.sortDirection === 'asc' ? 'desc' : 'asc'

	emit('sort', column, newDirection)
}
</script>

<template>
	<div class="overflow-hidden rounded-[20px] border border-surface-3">
		<!-- Header Row -->
		<div
			class="flex h-12 items-center justify-between gap-4 bg-surface-3 px-4"
			:class="showSelection ? '' : ''"
		>
			<!-- Checkbox + Project header -->
			<div class="flex shrink-0 items-center gap-4" :class="showSelection ? 'w-[350px]' : ''">
				<Checkbox
					v-if="showSelection"
					:model-value="allSelected"
					:indeterminate="someSelected"
					class="shrink-0"
					@update:model-value="toggleSelectAll"
				/>

				<button
					v-if="sortable"
					class="flex items-center gap-1.5 font-semibold text-contrast"
					@click="handleSort('project')"
				>
					Project
					<ChevronUpIcon v-if="sortBy === 'project' && sortDirection === 'asc'" class="size-4" />
					<ChevronDownIcon
						v-else-if="sortBy === 'project' && sortDirection === 'desc'"
						class="size-4"
					/>
				</button>
				<span v-else class="font-semibold text-contrast">Project</span>
			</div>

			<!-- Version header -->
			<div class="hidden w-[335px] shrink-0 md:block">
				<button
					v-if="sortable"
					class="flex items-center gap-1.5 font-semibold text-secondary"
					@click="handleSort('version')"
				>
					Version
					<ChevronUpIcon v-if="sortBy === 'version' && sortDirection === 'asc'" class="size-4" />
					<ChevronDownIcon
						v-else-if="sortBy === 'version' && sortDirection === 'desc'"
						class="size-4"
					/>
				</button>
				<span v-else class="font-semibold text-secondary">Version</span>
			</div>

			<!-- Actions header -->
			<div class="shrink-0 text-right">
				<span class="font-semibold text-secondary">Actions</span>
			</div>
		</div>

		<!-- Content Rows -->
		<template v-if="items.length > 0">
			<ContentCardItem
				v-for="(item, index) in items"
				:key="item.id"
				:project="item.project"
				:version="item.version"
				:owner="item.owner"
				:enabled="item.enabled"
				:overflow-options="item.overflowOptions"
				:disabled="disabled"
				:show-checkbox="showSelection"
				:selected="isItemSelected(item.id)"
				:class="[
					index % 2 === 1 ? 'bg-surface-1' : 'bg-surface-2',
					index === items.length - 1 && 'rounded-b-[20px]',
					'border-t border-solid border-[1px] border-surface-3',
				]"
				@update:selected="(val) => toggleItemSelection(item.id, val ?? false)"
				@update:enabled="(val) => emit('update:enabled', item.id, val)"
				@delete="emit('delete', item.id)"
				@update="emit('update', item.id)"
			>
				<template #additionalButtonsLeft>
					<slot name="itemButtonsLeft" :item="item" :index="index" />
				</template>
				<template #additionalButtonsRight>
					<slot name="itemButtonsRight" :item="item" :index="index" />
				</template>
			</ContentCardItem>
		</template>

		<!-- Empty State -->
		<div v-else class="flex items-center justify-center py-12">
			<slot name="empty">
				<span class="text-secondary">No items</span>
			</slot>
		</div>
	</div>
</template>
