<script setup lang="ts">
import { ChevronDownIcon, ChevronUpIcon } from '@modrinth/assets'
import { computed, onMounted, onUnmounted, ref } from 'vue'

import Checkbox from '../base/Checkbox.vue'
import ContentCardItem from './ContentCardItem.vue'
import type {
	ContentCardTableItem,
	ContentCardTableSortColumn,
	ContentCardTableSortDirection,
} from './types'

const BUFFER_SIZE = 5

interface Props {
	items: ContentCardTableItem[]
	showSelection?: boolean
	sortable?: boolean
	sortBy?: ContentCardTableSortColumn
	sortDirection?: ContentCardTableSortDirection
	virtualized?: boolean
}

const props = withDefaults(defineProps<Props>(), {
	showSelection: false,
	sortable: false,
	sortBy: undefined,
	sortDirection: 'asc',
	virtualized: true,
})

const selectedIds = defineModel<string[]>('selectedIds', { default: () => [] })

const emit = defineEmits<{
	'update:enabled': [id: string, value: boolean]
	delete: [id: string]
	update: [id: string]
	sort: [column: ContentCardTableSortColumn, direction: ContentCardTableSortDirection]
}>()

// Virtualization state
const listContainer = ref<HTMLElement | null>(null)
const scrollContainer = ref<HTMLElement | Window | null>(null)
const scrollTop = ref(0)
const viewportHeight = ref(0)
const itemHeight = 74

const totalHeight = computed(() => props.items.length * itemHeight)

// Find the nearest scrollable ancestor
function findScrollableAncestor(element: HTMLElement | null): HTMLElement | Window {
	if (!element) return window

	let current: HTMLElement | null = element.parentElement
	while (current) {
		const style = getComputedStyle(current)
		const overflowY = style.overflowY
		const isScrollable =
			(overflowY === 'auto' || overflowY === 'scroll') &&
			current.scrollHeight > current.clientHeight

		if (isScrollable) {
			return current
		}
		current = current.parentElement
	}
	return window
}

function getScrollTop(container: HTMLElement | Window): number {
	if (container instanceof Window) {
		return window.scrollY
	}
	return container.scrollTop
}

function getViewportHeight(container: HTMLElement | Window): number {
	if (container instanceof Window) {
		return window.innerHeight
	}
	return container.clientHeight
}

function getContainerOffset(listEl: HTMLElement, container: HTMLElement | Window): number {
	if (container instanceof Window) {
		return listEl.getBoundingClientRect().top + window.scrollY
	}
	// For element containers, get the offset relative to the scroll container
	const listRect = listEl.getBoundingClientRect()
	const containerRect = container.getBoundingClientRect()
	return listRect.top - containerRect.top + container.scrollTop
}

const visibleRange = computed(() => {
	if (!props.virtualized) {
		return { start: 0, end: props.items.length }
	}

	if (!listContainer.value || !scrollContainer.value) return { start: 0, end: 0 }

	const containerOffset = getContainerOffset(listContainer.value, scrollContainer.value)
	const relativeScrollTop = Math.max(0, scrollTop.value - containerOffset)

	const start = Math.floor(relativeScrollTop / itemHeight)
	const visibleCount = Math.ceil(viewportHeight.value / itemHeight)

	return {
		start: Math.max(0, start - BUFFER_SIZE),
		end: Math.min(props.items.length, start + visibleCount + BUFFER_SIZE * 2),
	}
})

const visibleTop = computed(() => (props.virtualized ? visibleRange.value.start * itemHeight : 0))

const visibleItems = computed(() =>
	props.items.slice(visibleRange.value.start, visibleRange.value.end),
)

// Expose for perf monitoring
defineExpose({
	visibleRange,
	visibleItems,
})

function handleScroll() {
	if (scrollContainer.value) {
		scrollTop.value = getScrollTop(scrollContainer.value)
	}
}

function handleResize() {
	if (scrollContainer.value) {
		viewportHeight.value = getViewportHeight(scrollContainer.value)
	}
}

onMounted(() => {
	scrollContainer.value = findScrollableAncestor(listContainer.value)
	viewportHeight.value = getViewportHeight(scrollContainer.value)
	scrollTop.value = getScrollTop(scrollContainer.value)

	scrollContainer.value.addEventListener('scroll', handleScroll, { passive: true })
	window.addEventListener('resize', handleResize, { passive: true })
})

onUnmounted(() => {
	if (scrollContainer.value) {
		scrollContainer.value.removeEventListener('scroll', handleScroll)
	}
	window.removeEventListener('resize', handleResize)
})

// Selection logic
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
	<div class="overflow-hidden rounded-[20px] border border-solid border-[1px] border-surface-3">
		<div
			class="flex h-12 items-center justify-between gap-4 bg-surface-3 px-4"
			:class="showSelection ? '' : ''"
		>
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

			<div class="min-w-[160px] shrink-0 text-right">
				<span class="font-semibold text-secondary">Actions</span>
			</div>
		</div>

		<div
			v-if="items.length > 0 && virtualized"
			ref="listContainer"
			class="relative w-full rounded-b-[20px]"
			:style="{ minHeight: `${totalHeight}px` }"
		>
			<div class="absolute w-full" :style="{ top: `${visibleTop}px` }">
				<ContentCardItem
					v-for="(item, idx) in visibleItems"
					:key="item.id"
					data-content-card-item
					:project="item.project"
					:project-link="item.projectLink"
					:version="item.version"
					:owner="item.owner"
					:enabled="item.enabled"
					:has-update="item.hasUpdate"
					:overflow-options="item.overflowOptions"
					:disabled="item.disabled"
					:show-checkbox="showSelection"
					:selected="isItemSelected(item.id)"
					:class="[
						(visibleRange.start + idx) % 2 === 1 ? 'bg-surface-1' : 'bg-surface-2',
						'border-t border-solid border-[1px] border-surface-3',
						visibleRange.start + idx === items.length - 1 ? 'rounded-b-[20px] !border-none' : '',
					]"
					@update:selected="(val) => toggleItemSelection(item.id, val ?? false)"
					@update:enabled="(val) => emit('update:enabled', item.id, val)"
					@delete="emit('delete', item.id)"
					@update="emit('update', item.id)"
				>
					<template #additionalButtonsLeft>
						<slot name="itemButtonsLeft" :item="item" :index="visibleRange.start + idx" />
					</template>
					<template #additionalButtonsRight>
						<slot name="itemButtonsRight" :item="item" :index="visibleRange.start + idx" />
					</template>
				</ContentCardItem>
			</div>
		</div>

		<div v-else-if="items.length > 0" ref="listContainer" class="rounded-b-[20px]">
			<ContentCardItem
				v-for="(item, index) in items"
				:key="item.id"
				data-content-card-item
				:project="item.project"
				:project-link="item.projectLink"
				:version="item.version"
				:owner="item.owner"
				:enabled="item.enabled"
				:has-update="item.hasUpdate"
				:overflow-options="item.overflowOptions"
				:disabled="item.disabled"
				:show-checkbox="showSelection"
				:selected="isItemSelected(item.id)"
				:class="[
					index % 2 === 1 ? 'bg-surface-1' : 'bg-surface-2',
					'border-t border-solid border-surface-3',
					index === items.length - 1 ? 'rounded-b-[20px] !border-none' : '',
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
		</div>

		<div v-else class="flex items-center justify-center rounded-b-[20px] py-12">
			<slot name="empty">
				<span class="text-secondary">No items</span>
			</slot>
		</div>
	</div>
</template>
