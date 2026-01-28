<template>
	<div class="overflow-hidden rounded-2xl border border-solid border-surface-3">
		<table class="w-full border-separate border-spacing-0">
			<thead>
				<tr class="bg-surface-3">
					<th v-if="showSelection" class="w-10 pl-4">
						<Checkbox
							:model-value="allSelected"
							:indeterminate="someSelected"
							class="shrink-0 py-4"
							@update:model-value="toggleSelectAll"
						/>
					</th>
					<th
						v-for="column in columns"
						:key="column.key"
						class="h-14 first:pl-4 last:pr-4"
						:class="[
							`text-${column.align ?? 'left'}`,
							column.enableSorting ? 'cursor-pointer select-none' : '',
						]"
						:style="column.width ? { width: column.width } : undefined"
						@click="column.enableSorting ? handleSort(column.key) : undefined"
					>
						<slot :name="`header-${column.key}`" :column="column">
							<span
								v-if="column.label || column.enableSorting"
								class="inline-flex items-center gap-1 font-semibold"
								:class="`${sortColumn === column.key ? 'text-contrast' : ''}`"
							>
								{{ column.label ?? '' }}
								<template v-if="column.enableSorting">
									<ChevronUpIcon
										v-if="sortColumn === column.key && sortDirection === 'asc'"
										class="size-4"
									/>
									<ChevronDownIcon
										v-else-if="sortColumn === column.key && sortDirection === 'desc'"
										class="size-4"
									/>
								</template>
							</span>
						</slot>
					</th>
				</tr>
			</thead>
			<tbody>
				<tr
					v-for="(row, rowIndex) in data"
					:key="rowIndex"
					:class="rowIndex % 2 === 0 ? 'bg-surface-2' : 'bg-surface-1.5'"
				>
					<td v-if="showSelection" class="w-10">
						<Checkbox
							:model-value="isSelected(row)"
							class="shrink-0 p-4"
							@update:model-value="toggleSelection(row)"
						/>
					</td>
					<td
						v-for="column in columns"
						:key="column.key"
						class="text-secondary h-14 first:pl-4 last:pr-4"
						:class="`text-${column.align ?? 'left'}`"
						:style="column.width ? { width: column.width } : undefined"
					>
						<slot
							:name="`cell-${column.key}`"
							:row="row"
							:value="row[column.key]"
							:column="column"
							:index="rowIndex"
						>
							{{ row[column.key] ?? '' }}
							{{ row[column.key] ?? '' }}
						</slot>
					</td>
				</tr>
			</tbody>
		</table>
	</div>
</template>

<script
	setup
	lang="ts"
	generic="K extends string = string, T extends Record<string, unknown> = Record<K, unknown>"
>
import { ChevronDownIcon, ChevronUpIcon } from '@modrinth/assets'
import { computed } from 'vue'

import Checkbox from './Checkbox.vue'

export type TableColumnAlign = 'left' | 'center' | 'right'
export type SortDirection = 'asc' | 'desc'

/**
 * Defines a table column configuration.
 * @template K - The column key is used to get cell data of row
 */
export interface TableColumn<K extends string = string> {
	key: K
	label?: string
	align?: TableColumnAlign
	enableSorting?: boolean
	/**
	 * CSS width value for the column.
	 * Accepts any valid CSS width (e.g., '200px', '20%', '10rem', 'auto', 'fit-content').
	 */
	width?: string
}

const props = withDefaults(
	defineProps<{
		columns: TableColumn<K>[]
		data: T[] /* Row data for table */
		showSelection?: boolean
		rowKey?: keyof T /* The key used to uniquely identify each row */
	}>(),
	{
		showSelection: false,
		rowKey: 'id' as keyof T,
	},
)

const selectedIds = defineModel<unknown[]>('selectedIds', { default: () => [] })
const sortColumn = defineModel<string | undefined>('sortColumn')
const sortDirection = defineModel<SortDirection>('sortDirection', { default: 'asc' })

const emit = defineEmits<{
	sort: [column: string, direction: SortDirection]
}>()

const allSelected = computed(
	() => props.data.length > 0 && selectedIds.value.length === props.data.length,
)
const someSelected = computed(
	() => selectedIds.value.length > 0 && selectedIds.value.length < props.data.length,
)

function getRowId(row: T): unknown {
	return row[props.rowKey as keyof T]
}

function isSelected(row: T): boolean {
	return selectedIds.value.includes(getRowId(row))
}

function toggleSelection(row: T) {
	const id = getRowId(row)
	if (isSelected(row)) {
		selectedIds.value = selectedIds.value.filter((selectedId) => selectedId !== id)
	} else {
		selectedIds.value = [...selectedIds.value, id]
	}
}

function toggleSelectAll(selectAll: boolean) {
	if (selectAll) {
		selectedIds.value = props.data.map((row) => getRowId(row))
	} else {
		selectedIds.value = []
	}
}

function handleSort(columnKey: string) {
	const newDirection: SortDirection =
		sortColumn.value === columnKey && sortDirection.value === 'asc' ? 'desc' : 'asc'
	sortColumn.value = columnKey
	sortDirection.value = newDirection
	emit('sort', columnKey, newDirection)
}
</script>
