<script setup lang="ts">
import {
	DragDropProvider,
	type DragEndEvent,
	type DragMoveEvent,
	type DragOverEvent,
	DragOverlay,
	type DragStartEvent,
} from '@dnd-kit/vue'
import { InfoIcon } from '@modrinth/assets'
import { computed, ref, watch } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { useStickyObserver } from '#ui/composables/sticky-observer'
import { useVirtualScroll } from '#ui/composables/virtual-scroll'
import { commonMessages } from '#ui/utils/common-messages'

import type { ContentCardTableItem, ContentSide } from '../../types'
import ContentEnabledFor from '../ContentEnabledFor.vue'
import DragGather from './drag-gather.vue'
import DragPreview from './drag-preview.vue'
import DragRow from './drag-row.vue'
import TableHeader from './header.vue'
import TableRow from './row.vue'
import SectionHeader from './section-header.vue'
import type { ContentTableColumns, ContentTableSection } from './types'
import {
	SECTION_EMPTY_HEIGHT,
	SECTION_HEADER_HEIGHT,
	useGroupedEntries,
} from './use-grouped-entries'
import { useRowDragGather } from './use-row-drag-gather'
import { useContentTableLayout } from './use-table-layout'

const props = withDefaults(
	defineProps<{
		sections: ContentTableSection[]
		columns?: ContentTableColumns
		highlightedItemId?: string
		virtualized?: boolean
		hideHeader?: boolean
		flat?: boolean
		draggable?: boolean
		dragDisabled?: boolean
		extraActionWidths?: (row: ContentCardTableItem) => number[]
	}>(),
	{
		columns: () => ({}),
		highlightedItemId: undefined,
		virtualized: true,
		hideHeader: false,
		flat: false,
		draggable: false,
		dragDisabled: false,
		extraActionWidths: () => [],
	},
)

const selectedIds = defineModel<string[]>('selectedIds', { default: () => [] })

const emit = defineEmits<{
	toggle: [id: string, value: boolean]
	enabledFor: [id: string, side: ContentSide, value: boolean]
	remove: [id: string, event: MouseEvent]
	update: [id: string]
	switchVersion: [id: string]
	prefetch: [id: string]
	setSectionCollapsed: [sectionId: string, collapsed: boolean]
	moveRows: [rowIds: string[], sectionId: string]
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	enabledFor: {
		id: 'content.enabled-for.label',
		defaultMessage: 'Enabled for',
	},
	enabledForDescription: {
		id: 'content.enabled-for.description',
		defaultMessage:
			'Choose where this content is enabled. Use the Actions toggle to enable or disable it entirely.',
	},
})

const tableRef = ref<HTMLElement | null>(null)
const headerRef = ref<InstanceType<typeof TableHeader> | null>(null)
const enabledForMeasureRef = ref<HTMLElement | null>(null)
const actionsLabelRef = ref<HTMLElement | null>(null)

const { isStuck } = useStickyObserver(
	computed(() => headerRef.value?.element ?? null),
	'ContentTable',
)

const rows = computed(() => props.sections.flatMap((section) => section.items))
const grouped = computed(() => props.sections.some((section) => !!section.label))
const canDrag = computed(() => grouped.value && props.draggable)

const showSelection = computed(() => !!props.columns.selection)
const showVersion = computed(() => props.columns.version !== false)
const showEnabledFor = computed(() => !!props.columns.enabledFor)

const { layout, stacked, separateVersion, hasActionsColumn, actionSet, rowHeight, columnStyles } =
	useContentTableLayout({
		rows,
		actions: computed(() => props.columns.actions),
		showVersion,
		showEnabledFor,
		extraActionWidths: (row) => props.extraActionWidths(row),
		tableRef,
		enabledForMeasureRef,
		actionsLabelRef,
	})

const { listContainer, totalHeight, visibleRange, visibleTop, visibleItems, scrollToIndex } =
	useVirtualScroll(rows, {
		itemHeight: rowHeight,
		bufferSize: 5,
		initialItemCount: 20,
		enabled: computed(() => props.virtualized && !grouped.value),
	})

const groupedEntries = useGroupedEntries({
	sections: computed(() => props.sections),
	rowHeight,
	enabled: computed(() => props.virtualized && grouped.value),
})
const groupedListContainer = groupedEntries.listContainer
const lastGroupedEntryKey = computed(() => {
	const sections = props.sections
	const last = sections[sections.length - 1]
	if (!last) return null
	if (last.collapsed) return `header:${last.id}`
	if (last.items.length === 0) return last.emptyLabel ? `empty:${last.id}` : `header:${last.id}`
	return `row:${last.items[last.items.length - 1].id}`
})

const orderedRows = computed(() => (grouped.value ? groupedEntries.rows.value : rows.value))

watch(
	[
		() => props.highlightedItemId,
		() => (grouped.value ? groupedListContainer.value : listContainer.value),
	],
	([id, container], _, onCleanup) => {
		if (!id || !container) return
		const frame = requestAnimationFrame(() => {
			if (grouped.value) {
				groupedEntries.scrollToRow(id)
			} else {
				const index = rows.value.findIndex((row) => row.id === id)
				if (index >= 0) scrollToIndex(index)
			}
		})
		onCleanup(() => cancelAnimationFrame(frame))
	},
	{ flush: 'post' },
)

defineExpose({ visibleRange, visibleItems })

const selectableRows = computed(() => rows.value.filter((row) => !row.disabled))
const allSelected = computed(
	() =>
		selectableRows.value.length > 0 &&
		selectableRows.value.every((row) => selectedIds.value.includes(row.id)),
)
const someSelected = computed(
	() =>
		!allSelected.value && selectableRows.value.some((row) => selectedIds.value.includes(row.id)),
)

function toggleSelectAll() {
	selectedIds.value =
		allSelected.value || someSelected.value ? [] : selectableRows.value.map((row) => row.id)
}

const lastSelectedIndex = ref<number | null>(null)

function toggleRowSelection(id: string, selected: boolean, index: number, event?: MouseEvent) {
	if (selected && event?.shiftKey && lastSelectedIndex.value !== null) {
		const start = Math.min(lastSelectedIndex.value, index)
		const end = Math.max(lastSelectedIndex.value, index)
		const rangeIds = orderedRows.value
			.slice(start, end + 1)
			.filter((row) => !row.disabled)
			.map((row) => row.id)
		selectedIds.value = [...new Set([...selectedIds.value, ...rangeIds])]
	} else if (selected) {
		if (!selectedIds.value.includes(id)) selectedIds.value = [...selectedIds.value, id]
	} else {
		selectedIds.value = selectedIds.value.filter((selectedId) => selectedId !== id)
	}
	lastSelectedIndex.value = index
}

watch(grouped, () => {
	lastSelectedIndex.value = null
})

const sectionIdByRowId = computed(() => {
	const map = new Map<string, string>()
	for (const section of props.sections) {
		for (const row of section.items) map.set(row.id, section.id)
	}
	return map
})
const rowsById = computed(() => new Map(rows.value.map((row) => [row.id, row])))

const activeDragIds = ref<string[]>([])
const activeDragIdSet = computed(() => new Set(activeDragIds.value))
const activeDropSectionId = ref<string | null>(null)
const activeDragRow = computed(() =>
	activeDragIds.value.length > 0 ? rowsById.value.get(activeDragIds.value[0]) : undefined,
)
const gather = useRowDragGather(rows)

function canDropOnSection(sectionId: string | null) {
	if (!sectionId || activeDragIds.value.length === 0) return false
	const section = props.sections.find((candidate) => candidate.id === sectionId)
	if (!section || section.droppable === false) return false
	return activeDragIds.value.some((id) => sectionIdByRowId.value.get(id) !== sectionId)
}

const dropSectionId = computed(() =>
	canDropOnSection(activeDropSectionId.value) ? activeDropSectionId.value : null,
)

function handleDragStart(event: DragStartEvent) {
	const rowId = (event.operation.source?.data as { rowId?: string } | undefined)?.rowId
	if (!rowId) return
	const selected = new Set(selectedIds.value)
	activeDragIds.value = selected.has(rowId)
		? [
				rowId,
				...orderedRows.value
					.filter((row) => row.id !== rowId && selected.has(row.id))
					.map((row) => row.id),
				...selectedIds.value.filter(
					(id) => id !== rowId && !orderedRows.value.some((row) => row.id === id),
				),
			]
		: [rowId]
	gather.start(activeDragIds.value, event.operation.position.current)
}

function handleDragMove(event: DragMoveEvent) {
	if (gather.items.value.length === 0) return
	gather.updateTarget(event.to ?? event.operation.position.current)
}

function handleDragOver(event: DragOverEvent) {
	const target = event.operation.target?.data as { sectionId?: string } | undefined
	activeDropSectionId.value = target?.sectionId ?? null
}

function handleDragEnd(event: DragEndEvent) {
	const target = event.operation.target?.data as { sectionId?: string } | undefined
	const sectionId = target?.sectionId ?? null
	if (!event.canceled && sectionId && canDropOnSection(sectionId)) {
		emit('moveRows', [...activeDragIds.value], sectionId)
	}
	gather.clear()
	activeDragIds.value = []
	activeDropSectionId.value = null
}

function toggleSection(section: ContentTableSection) {
	emit('setSectionCollapsed', section.id, !section.collapsed)
}

function rowBackground(row: ContentCardTableItem, index: number, sectionId?: string) {
	if (sectionId && dropSectionId.value === sectionId) return 'bg-surface-3'
	if (selectedIds.value.includes(row.id)) return 'bg-surface-2.5'
	return index % 2 === 1 ? 'bg-surface-1.5' : 'bg-surface-2'
}

function rowClass(row: ContentCardTableItem, index: number, isLast: boolean, sectionId?: string) {
	return [
		rowBackground(row, index, sectionId),
		'border-0 border-t border-solid border-surface-4 transition-[background-color,opacity,filter] duration-150 hover:brightness-[1.15]',
		props.highlightedItemId && row.id === props.highlightedItemId
			? 'outline outline-2 -outline-offset-2 outline-brand'
			: '',
		activeDragIdSet.value.has(row.id) ? 'opacity-50' : '',
		isLast && !props.flat ? 'rounded-b-[20px]' : '',
	]
}
</script>

<template>
	<div
		ref="tableRef"
		role="table"
		class="@container relative overflow-clip border border-solid border-surface-4 shadow-sm"
		:class="[flat ? 'border-x-0' : 'rounded-[20px]', isStuck || hideHeader ? 'border-t-0' : '']"
		:style="columnStyles"
	>
		<div aria-hidden="true" inert class="pointer-events-none invisible absolute left-0 top-0 w-max">
			<div v-if="showEnabledFor" ref="enabledForMeasureRef" class="flex w-max flex-col items-start">
				<span class="flex items-center gap-1.5 whitespace-nowrap font-semibold">
					{{ formatMessage(messages.enabledFor) }}
					<InfoIcon class="size-4 shrink-0" />
				</span>
				<ContentEnabledFor
					:model-value="{ server: false, player: false, locked: false }"
					reserve-status-space
				/>
			</div>
			<span v-if="hasActionsColumn" ref="actionsLabelRef" class="block w-max font-semibold">
				{{ formatMessage(commonMessages.actionsLabel) }}
			</span>
		</div>

		<TableHeader
			v-if="!hideHeader"
			ref="headerRef"
			:layout="layout"
			:stacked="stacked"
			:flat="flat"
			:stuck="isStuck"
			:selection="showSelection"
			:all-selected="allSelected"
			:some-selected="someSelected"
			:selection-disabled="selectableRows.length === 0"
			:show-enabled-for="showEnabledFor"
			:show-version="separateVersion"
			:show-actions="hasActionsColumn"
			:drag-handle="canDrag"
			:enabled-for-label="formatMessage(messages.enabledFor)"
			:enabled-for-description="formatMessage(messages.enabledForDescription)"
			@toggle-all="toggleSelectAll"
		/>

		<div
			v-if="rows.length === 0 && !grouped"
			class="flex items-center justify-center py-12"
			:class="flat ? '' : 'rounded-b-[20px]'"
		>
			<slot name="empty">
				<span class="text-secondary">{{ formatMessage(commonMessages.noItemsLabel) }}</span>
			</slot>
		</div>

		<DragDropProvider
			v-else-if="grouped"
			@drag-start="handleDragStart"
			@drag-move="handleDragMove"
			@drag-over="handleDragOver"
			@drag-end="handleDragEnd"
		>
			<div
				ref="groupedListContainer"
				role="rowgroup"
				class="relative w-full"
				:class="flat ? '' : 'rounded-b-[20px]'"
				:style="
					virtualized
						? { minHeight: `${groupedEntries.totalHeight.value}px`, overflowAnchor: 'none' }
						: undefined
				"
			>
				<div
					:class="{ 'absolute w-full': virtualized }"
					:style="virtualized ? { top: `${groupedEntries.visibleTop.value}px` } : undefined"
				>
					<template v-for="entry in groupedEntries.visibleEntries.value" :key="entry.key">
						<SectionHeader
							v-if="entry.kind === 'header'"
							:section="entry.section"
							:droppable="canDrag && entry.section.droppable !== false"
							:drop-active="dropSectionId === entry.section.id"
							:class="lastGroupedEntryKey === entry.key && !flat ? 'rounded-b-[20px]' : ''"
							:style="{ height: `${SECTION_HEADER_HEIGHT}px` }"
							@toggle="toggleSection(entry.section)"
						>
							<template v-if="$slots['section-header']" #default="{ toggle }">
								<slot
									name="section-header"
									:section="entry.section"
									:collapsed="!!entry.section.collapsed"
									:toggle="toggle"
								/>
							</template>
						</SectionHeader>
						<div
							v-else-if="entry.kind === 'empty'"
							role="row"
							class="flex items-center border-0 border-t border-solid border-surface-4 px-4 text-secondary transition-colors duration-150"
							:class="[
								dropSectionId === entry.section.id ? 'bg-surface-3' : 'bg-surface-2',
								lastGroupedEntryKey === entry.key && !flat ? 'rounded-b-[20px]' : '',
							]"
							:style="{ height: `${SECTION_EMPTY_HEIGHT}px` }"
						>
							{{ entry.section.emptyLabel }}
						</div>
						<DragRow
							v-else
							:row-id="entry.row.id"
							:section-id="entry.section.id"
							:disabled="!canDrag || dragDisabled || !!entry.row.disabled"
						>
							<TableRow
								:data-content-card-item="entry.row.id"
								:row="entry.row"
								:layout="layout"
								:actions="actionSet"
								:selection="showSelection"
								:selected="selectedIds.includes(entry.row.id)"
								:show-version="showVersion"
								:show-enabled-for="showEnabledFor"
								:show-actions="hasActionsColumn"
								:drag-handle="canDrag"
								:class="
									rowClass(
										entry.row,
										entry.indexInSection,
										lastGroupedEntryKey === entry.key,
										entry.section.id,
									)
								"
								@select="
									(value, event) => toggleRowSelection(entry.row.id, value, entry.rowIndex, event)
								"
								@toggle="(value) => emit('toggle', entry.row.id, value)"
								@enabled-for="(side, value) => emit('enabledFor', entry.row.id, side, value)"
								@remove="(event) => emit('remove', entry.row.id, event)"
								@update="emit('update', entry.row.id)"
								@switch-version="emit('switchVersion', entry.row.id)"
								@prefetch="emit('prefetch', entry.row.id)"
							>
								<template #title-badges>
									<slot name="row-title-badges" :item="entry.row" />
								</template>
								<template #actions-before>
									<slot name="row-actions-before" :item="entry.row" />
								</template>
								<template #actions-after>
									<slot name="row-actions-after" :item="entry.row" />
								</template>
							</TableRow>
						</DragRow>
					</template>
				</div>
			</div>
			<Teleport to="body">
				<div class="pointer-events-none fixed inset-0 z-[9999]">
					<DragOverlay :drop-animation="null">
						<div
							v-if="activeDragRow"
							class="w-fit transition-transform duration-150 ease-out"
							:class="gather.isGathering.value ? 'scale-[0.975]' : 'scale-100'"
						>
							<DragPreview :row="activeDragRow" :count="activeDragIds.length" />
						</div>
					</DragOverlay>
				</div>
			</Teleport>
			<DragGather
				v-if="gather.items.value.length > 0"
				:items="gather.items.value"
				:target="gather.target.value"
				@complete="gather.finish()"
			/>
		</DragDropProvider>

		<div
			v-else
			ref="listContainer"
			role="rowgroup"
			class="relative w-full"
			:class="flat ? '' : 'rounded-b-[20px]'"
			:style="virtualized ? { minHeight: `${totalHeight}px`, overflowAnchor: 'none' } : undefined"
		>
			<div
				:class="{ 'absolute w-full': virtualized }"
				:style="virtualized ? { top: `${visibleTop}px` } : undefined"
			>
				<TableRow
					v-for="(row, offset) in visibleItems"
					:key="row.id"
					:data-content-card-item="row.id"
					:row="row"
					:layout="layout"
					:actions="actionSet"
					:selection="showSelection"
					:selected="selectedIds.includes(row.id)"
					:show-version="showVersion"
					:show-enabled-for="showEnabledFor"
					:show-actions="hasActionsColumn"
					:class="
						rowClass(
							row,
							visibleRange.start + offset,
							visibleRange.start + offset === rows.length - 1,
						)
					"
					@select="
						(value, event) => toggleRowSelection(row.id, value, visibleRange.start + offset, event)
					"
					@toggle="(value) => emit('toggle', row.id, value)"
					@enabled-for="(side, value) => emit('enabledFor', row.id, side, value)"
					@remove="(event) => emit('remove', row.id, event)"
					@update="emit('update', row.id)"
					@switch-version="emit('switchVersion', row.id)"
					@prefetch="emit('prefetch', row.id)"
				>
					<template #title-badges>
						<slot name="row-title-badges" :item="row" />
					</template>
					<template #actions-before>
						<slot name="row-actions-before" :item="row" />
					</template>
					<template #actions-after>
						<slot name="row-actions-after" :item="row" />
					</template>
				</TableRow>
			</div>
		</div>
	</div>
</template>
