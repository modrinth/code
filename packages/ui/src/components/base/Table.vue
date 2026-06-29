<template>
	<div class="overflow-hidden rounded-2xl border border-solid border-surface-4">
		<div
			v-if="hasHeaderSlot"
			class="border-solid border-0 border-b border-surface-4 bg-surface-3 p-4"
		>
			<slot name="header" />
		</div>
		<div class="overflow-x-auto overflow-y-hidden">
			<table
				class="w-full table-fixed border-separate border-spacing-0 border-surface-4"
				:style="tableMinWidth ? { minWidth: tableMinWidth } : undefined"
			>
				<colgroup>
					<col v-if="showSelection" class="w-12" />
					<col
						v-for="column in columns"
						:key="column.key"
						:style="column.width ? { width: column.width } : undefined"
					/>
				</colgroup>
				<thead class="">
					<tr class="bg-surface-3">
						<th v-if="showSelection" class="w-12">
							<Checkbox
								:model-value="allSelected"
								:indeterminate="someSelected"
								class="shrink-0 p-4 focus-visible:!outline-none"
								@update:model-value="toggleSelectAll"
							/>
						</th>
						<th
							v-for="column in columns"
							:key="column.key"
							class="h-12 first:pl-4 last:pr-4"
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
									class="inline-flex min-w-0 max-w-full items-center gap-1 font-semibold"
									:class="`${sortColumn === column.key ? 'text-contrast -mr-1' : ''}`"
								>
									<span class="min-w-0 truncate">{{ column.label ?? '' }}</span>
									<template v-if="column.enableSorting">
										<ChevronUpIcon
											v-if="sortColumn === column.key && sortDirection === 'asc'"
											class="size-4 shrink-0"
										/>
										<ChevronDownIcon
											v-else-if="sortColumn === column.key && sortDirection === 'desc'"
											class="size-4 shrink-0"
										/>
									</template>
								</span>
							</slot>
						</th>
					</tr>
				</thead>
				<TransitionGroup
					v-if="rowTransitionName && !virtualized"
					:name="rowTransitionName"
					tag="tbody"
				>
					<tr v-if="data.length === 0" key="empty" class="bg-surface-2">
						<td :colspan="columnSpan" class="border-solid border-0 border-t border-surface-4 p-0">
							<slot name="empty-state">
								<div class="text-secondary flex h-64 items-center justify-center">
									No data available.
								</div>
							</slot>
						</td>
					</tr>
					<template v-else>
						<tr
							v-for="(row, rowIndex) in renderedRows"
							:key="getRowRenderKey(row, getAbsoluteRowIndex(rowIndex))"
							:class="getRowClass(getAbsoluteRowIndex(rowIndex))"
						>
							<td
								v-if="showSelection"
								class="w-12 border-solid border-0 border-t border-surface-4 focus:outline-none"
								:class="getBodyCellClass(row, getAbsoluteRowIndex(rowIndex))"
							>
								<Checkbox
									:model-value="isSelected(row)"
									class="shrink-0 p-4 -outline-offset-[14px] outline rounded-2xl"
									@update:model-value="(selectRow, event) => toggleSelection(row, selectRow, event)"
								/>
							</td>
							<td
								v-for="column in columns"
								:key="column.key"
								class="text-secondary h-14 overflow-hidden first:pl-4 last:pr-4 border-solid border-0 border-t border-surface-4"
								:class="[
									getBodyCellClass(row, getAbsoluteRowIndex(rowIndex)),
									`text-${column.align ?? 'left'}`,
								]"
							>
								<slot
									:name="`cell-${column.key}`"
									:row="row"
									:value="row[column.key]"
									:column="column"
									:index="getAbsoluteRowIndex(rowIndex)"
								>
									{{ row[column.key] ?? '' }}
								</slot>
							</td>
						</tr>
					</template>
				</TransitionGroup>
				<tbody v-else :ref="setListContainer">
					<tr v-if="data.length === 0" class="bg-surface-2">
						<td :colspan="columnSpan" class="border-solid border-0 border-t border-surface-4 p-0">
							<slot name="empty-state">
								<div class="text-secondary flex h-64 items-center justify-center">
									No data available.
								</div>
							</slot>
						</td>
					</tr>
					<template v-else>
						<tr v-if="virtualized && topSpacerHeight > 0" aria-hidden="true">
							<td
								:colspan="columnSpan"
								class="border-0 p-0"
								:style="{ height: `${topSpacerHeight}px` }"
							></td>
						</tr>
						<tr
							v-for="(row, rowIndex) in renderedRows"
							:key="getRowRenderKey(row, getAbsoluteRowIndex(rowIndex))"
							:class="getRowClass(getAbsoluteRowIndex(rowIndex))"
						>
							<td
								v-if="showSelection"
								class="w-12 border-solid border-0 border-t border-surface-4 focus:outline-none"
								:class="getBodyCellClass(row, getAbsoluteRowIndex(rowIndex))"
							>
								<Checkbox
									:model-value="isSelected(row)"
									class="shrink-0 p-4 -outline-offset-[14px] outline rounded-2xl"
									@update:model-value="(selectRow, event) => toggleSelection(row, selectRow, event)"
								/>
							</td>
							<td
								v-for="column in columns"
								:key="column.key"
								class="text-secondary h-14 overflow-hidden first:pl-4 last:pr-4 border-solid border-0 border-t border-surface-4"
								:class="[
									getBodyCellClass(row, getAbsoluteRowIndex(rowIndex)),
									`text-${column.align ?? 'left'}`,
								]"
							>
								<slot
									:name="`cell-${column.key}`"
									:row="row"
									:value="row[column.key]"
									:column="column"
									:index="getAbsoluteRowIndex(rowIndex)"
								>
									{{ row[column.key] ?? '' }}
								</slot>
							</td>
						</tr>
						<tr v-if="virtualized && bottomSpacerHeight > 0" aria-hidden="true">
							<td
								:colspan="columnSpan"
								class="border-0 p-0"
								:style="{ height: `${bottomSpacerHeight}px` }"
							></td>
						</tr>
					</template>
				</tbody>
			</table>
		</div>
	</div>
</template>

<script
	setup
	lang="ts"
	generic="K extends string = string, T extends Record<string, unknown> = Record<K, unknown>"
>
import { ChevronDownIcon, ChevronUpIcon } from '@modrinth/assets'
import { computed, ref, toRef, useSlots } from 'vue'

import { useVirtualScroll } from '../../composables/virtual-scroll'
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
	defaultSortDirection?: SortDirection
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
		selectionKey?: keyof T /* The key used to identify selectable rows */
		selectionData?: T[] /* The complete selectable data set when data is paginated */
		selectionIds?: unknown[] /* Complete selectable IDs when callers do not want to retain row objects */
		virtualized?: boolean
		virtualRowHeight?: number
		virtualBufferSize?: number /* The number of extra rows rendered above and below the visible viewport */
		rowTransitionName?: string
		bodyCellClass?: string | ((row: T, rowIndex: number) => string)
		/**
		 * Sets a minimum width for the table content, allowing horizontal overflow below that width.
		 */
		tableMinWidth?: string
	}>(),
	{
		showSelection: false,
		rowKey: 'id' as keyof T,
		virtualized: false,
		virtualRowHeight: 56,
		virtualBufferSize: 5,
	},
)

const selectedIds = defineModel<unknown[]>('selectedIds', { default: () => [] })
const sortColumn = defineModel<string | undefined>('sortColumn')
const sortDirection = defineModel<SortDirection>('sortDirection', { default: 'asc' })
const slots = useSlots()
const selectionAnchorId = ref<unknown>()
const hasHeaderSlot = computed(() => Boolean(slots.header))
const columnSpan = computed(() => Math.max(props.columns.length + (props.showSelection ? 1 : 0), 1))

const {
	listContainer,
	totalHeight,
	visibleRange,
	visibleTop: topSpacerHeight,
	visibleItems,
} = useVirtualScroll(toRef(props, 'data'), {
	itemHeight: props.virtualRowHeight,
	bufferSize: props.virtualBufferSize,
	enabled: toRef(props, 'virtualized'),
})

const renderedRows = computed(() => (props.virtualized ? visibleItems.value : props.data))
const bottomSpacerHeight = computed(() => {
	if (!props.virtualized) {
		return 0
	}

	return Math.max(
		0,
		totalHeight.value - topSpacerHeight.value - renderedRows.value.length * props.virtualRowHeight,
	)
})

const emit = defineEmits<{
	sort: [column: string, direction: SortDirection]
}>()

const selectableRows = computed(() => props.selectionData ?? props.data)
const selectableRowIds = computed(
	() => props.selectionIds ?? selectableRows.value.map((row) => getSelectionId(row)),
)
const selectedIdSet = computed(() => new Set(selectedIds.value))
const selectedSelectableIdCount = computed(() => {
	let count = 0
	for (const id of selectableRowIds.value) {
		if (selectedIdSet.value.has(id)) {
			count++
		}
	}
	return count
})
const allSelected = computed(
	() =>
		selectableRowIds.value.length > 0 &&
		selectedSelectableIdCount.value === selectableRowIds.value.length,
)
const someSelected = computed(
	() =>
		selectedSelectableIdCount.value > 0 &&
		selectedSelectableIdCount.value < selectableRowIds.value.length,
)

function getRowId(row: T): unknown {
	return row[props.rowKey as keyof T]
}

function getSelectionId(row: T): unknown {
	return row[(props.selectionKey ?? props.rowKey) as keyof T]
}

function setListContainer(element: unknown) {
	listContainer.value = props.virtualized ? (element as HTMLElement | null) : null
}

function getAbsoluteRowIndex(rowIndex: number): number {
	return props.virtualized ? visibleRange.value.start + rowIndex : rowIndex
}

function getRowRenderKey(row: T, rowIndex: number): PropertyKey {
	const rowId = getRowId(row)
	if (typeof rowId === 'string' || typeof rowId === 'number' || typeof rowId === 'symbol') {
		return rowId
	}

	return rowIndex
}

function getRowClass(rowIndex: number): string {
	return rowIndex % 2 === 0 ? 'bg-surface-2' : 'bg-surface-1.5'
}

function getBodyCellClass(row: T, rowIndex: number): string {
	if (typeof props.bodyCellClass === 'function') {
		return props.bodyCellClass(row, rowIndex)
	}

	return props.bodyCellClass ?? 'h-14'
}

function isSelected(row: T): boolean {
	return selectedIdSet.value.has(getSelectionId(row))
}

function toggleSelection(row: T, selectRow: boolean, event?: MouseEvent) {
	const id = getSelectionId(row)
	const rowIndex = selectableRowIds.value.findIndex((selectableId) => selectableId === id)
	const anchorIndex = selectableRowIds.value.findIndex(
		(selectableId) => selectableId === selectionAnchorId.value,
	)

	if (event?.shiftKey && rowIndex !== -1 && anchorIndex !== -1) {
		const startIndex = Math.min(rowIndex, anchorIndex)
		const endIndex = Math.max(rowIndex, anchorIndex)
		const rangeIds = selectableRowIds.value.slice(startIndex, endIndex + 1)

		if (selectRow) {
			const nextSelectedIds = [...selectedIds.value]
			const nextSelectedIdSet = new Set(nextSelectedIds)
			for (const rangeId of rangeIds) {
				if (!nextSelectedIdSet.has(rangeId)) {
					nextSelectedIds.push(rangeId)
					nextSelectedIdSet.add(rangeId)
				}
			}
			selectedIds.value = nextSelectedIds
		} else {
			const rangeIdSet = new Set(rangeIds)
			selectedIds.value = selectedIds.value.filter((selectedId) => !rangeIdSet.has(selectedId))
		}
	} else {
		selectedIds.value = selectRow
			? [...selectedIds.value, id]
			: selectedIds.value.filter((selectedId) => selectedId !== id)
	}

	selectionAnchorId.value = id
}

function toggleSelectAll(selectAll: boolean) {
	selectionAnchorId.value = undefined
	if (selectAll) {
		selectedIds.value = [...selectableRowIds.value]
	} else {
		selectedIds.value = []
	}
}

function handleSort(columnKey: string) {
	const column = props.columns.find((column) => column.key === columnKey)
	const defaultDirection = column?.defaultSortDirection ?? 'asc'
	const newDirection: SortDirection =
		sortColumn.value === columnKey && sortDirection.value === defaultDirection
			? getOppositeSortDirection(defaultDirection)
			: defaultDirection
	sortColumn.value = columnKey
	sortDirection.value = newDirection
	emit('sort', columnKey, newDirection)
}

function getOppositeSortDirection(direction: SortDirection): SortDirection {
	return direction === 'asc' ? 'desc' : 'asc'
}
</script>
