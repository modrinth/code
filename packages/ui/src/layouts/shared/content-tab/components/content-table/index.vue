<script setup lang="ts">
import { InfoIcon } from '@modrinth/assets'
import { computed, ref, watch } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { useStickyObserver } from '#ui/composables/sticky-observer'
import { useVirtualScroll } from '#ui/composables/virtual-scroll'
import { commonMessages } from '#ui/utils/common-messages'

import type { ContentCardTableItem, ContentSide } from '../../types'
import ContentEnabledFor from '../ContentEnabledFor.vue'
import TableHeader from './header.vue'
import TableRow from './row.vue'
import type { ContentTableColumns, ContentTableSection } from './types'
import { useContentTableLayout } from './use-table-layout'

const props = withDefaults(
	defineProps<{
		sections: ContentTableSection[]
		columns?: ContentTableColumns
		highlightedItemId?: string
		virtualized?: boolean
		hideHeader?: boolean
		flat?: boolean
		extraActionWidths?: (row: ContentCardTableItem) => number[]
	}>(),
	{
		columns: () => ({}),
		highlightedItemId: undefined,
		virtualized: true,
		hideHeader: false,
		flat: false,
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
const sectionOffsets = computed(() => {
	let offset = 0
	return props.sections.map((section) => {
		const start = offset
		offset += section.items.length
		return start
	})
})

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

watch(
	[
		() =>
			props.highlightedItemId
				? rows.value.findIndex((row) => row.id === props.highlightedItemId)
				: -1,
		listContainer,
	],
	([index, container], _, onCleanup) => {
		if (index < 0 || !container) return
		const frame = requestAnimationFrame(() => scrollToIndex(index))
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
		const rangeIds = rows.value
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

function rowClass(row: ContentCardTableItem, index: number) {
	const selected = selectedIds.value.includes(row.id)
	return [
		selected ? 'bg-surface-2.5' : index % 2 === 1 ? 'bg-surface-1.5' : 'bg-surface-2',
		'border-0 border-t border-solid border-surface-4',
		props.highlightedItemId && row.id === props.highlightedItemId
			? 'outline outline-2 -outline-offset-2 outline-brand'
			: '',
		index === rows.value.length - 1 && !props.flat ? 'rounded-b-[20px]' : '',
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
			:enabled-for-label="formatMessage(messages.enabledFor)"
			:enabled-for-description="formatMessage(messages.enabledForDescription)"
			@toggle-all="toggleSelectAll"
		/>

		<div
			v-if="rows.length === 0"
			class="flex items-center justify-center py-12"
			:class="flat ? '' : 'rounded-b-[20px]'"
		>
			<slot name="empty">
				<span class="text-secondary">{{ formatMessage(commonMessages.noItemsLabel) }}</span>
			</slot>
		</div>

		<div
			v-else-if="grouped"
			ref="listContainer"
			role="rowgroup"
			:class="flat ? '' : 'rounded-b-[20px]'"
		>
			<template v-for="(section, sectionIndex) in sections" :key="section.id">
				<div
					v-if="section.label"
					class="flex h-11 items-center border-0 border-t border-solid border-surface-4 bg-surface-2 px-4 font-semibold text-secondary"
				>
					{{ section.label }}
				</div>
				<TableRow
					v-for="(row, rowIndex) in section.items"
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
					:class="rowClass(row, sectionOffsets[sectionIndex] + rowIndex)"
					@select="
						(value, event) =>
							toggleRowSelection(row.id, value, sectionOffsets[sectionIndex] + rowIndex, event)
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
			</template>
		</div>

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
					:class="rowClass(row, visibleRange.start + offset)"
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
