<script setup lang="ts">
import { ChevronDownIcon, ChevronUpIcon } from '@modrinth/assets'
import { computed, getCurrentInstance, onMounted, onUnmounted, ref, toRef } from 'vue'

import Checkbox from '#ui/components/base/Checkbox.vue'
import { useVIntl } from '#ui/composables/i18n'
import { useStickyObserver } from '#ui/composables/sticky-observer'
import { useVirtualScroll } from '#ui/composables/virtual-scroll'
import { commonMessages } from '#ui/utils/common-messages'

import type {
	ContentCardTableItem,
	ContentCardTableSortColumn,
	ContentCardTableSortDirection,
} from '../types'
import ContentCardItem from './ContentCardItem.vue'

const { formatMessage } = useVIntl()

interface Props {
	items: ContentCardTableItem[]
	showSelection?: boolean
	sortable?: boolean
	sortBy?: ContentCardTableSortColumn
	sortDirection?: ContentCardTableSortDirection
	virtualized?: boolean
	hideDelete?: boolean
	hideHeader?: boolean
	flat?: boolean
}

const props = withDefaults(defineProps<Props>(), {
	showSelection: false,
	sortable: false,
	sortBy: undefined,
	sortDirection: 'asc',
	virtualized: true,
	hideDelete: false,
	hideHeader: false,
	flat: false,
})

const stickyHeaderRef = ref<HTMLElement | null>(null)
const { isStuck } = useStickyObserver(stickyHeaderRef, 'ContentCardTable')

const selectedIds = defineModel<string[]>('selectedIds', { default: () => [] })

const emit = defineEmits<{
	'update:enabled': [id: string, value: boolean]
	delete: [id: string, event: MouseEvent]
	update: [id: string]
	sort: [column: ContentCardTableSortColumn, direction: ContentCardTableSortDirection]
}>()

// Check if any actions are available
const instance = getCurrentInstance()
const hasDeleteListener = computed(() => typeof instance?.vnode.props?.onDelete === 'function')
const hasUpdateListener = computed(() => typeof instance?.vnode.props?.onUpdate === 'function')
const hasEnabledListener = computed(
	() => typeof instance?.vnode.props?.['onUpdate:enabled'] === 'function',
)

const hasAnyActions = computed(() => {
	// Check if there are listeners for actions
	const hasListeners =
		(hasDeleteListener.value && !props.hideDelete) ||
		hasUpdateListener.value ||
		hasEnabledListener.value

	// Check if any items have overflow options or updates
	const hasItemActions = props.items.some(
		(item) =>
			(item.overflowOptions && item.overflowOptions.length > 0) ||
			item.hasUpdate ||
			item.enabled !== undefined,
	)

	return hasListeners || hasItemActions
})

// Virtualization
const { listContainer, totalHeight, visibleRange, visibleTop, visibleItems } = useVirtualScroll(
	toRef(props, 'items'),
	{
		itemHeight: 74,
		bufferSize: 5,
		enabled: toRef(props, 'virtualized'),
	},
)

// Expose for perf monitoring
defineExpose({
	visibleRange,
	visibleItems,
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
	if (allSelected.value || someSelected.value) {
		selectedIds.value = []
	} else {
		selectedIds.value = props.items.map((item) => item.id)
	}
}

const lastSelectedIndex = ref<number | null>(null)
const shiftHeld = ref(false)

function onKeyDown(e: KeyboardEvent) {
	if (e.key === 'Shift') shiftHeld.value = true
}
function onKeyUp(e: KeyboardEvent) {
	if (e.key === 'Shift') shiftHeld.value = false
}

onMounted(() => {
	window.addEventListener('keydown', onKeyDown)
	window.addEventListener('keyup', onKeyUp)
})
onUnmounted(() => {
	window.removeEventListener('keydown', onKeyDown)
	window.removeEventListener('keyup', onKeyUp)
})

function toggleItemSelection(itemId: string, selected: boolean, index?: number) {
	if (selected && shiftHeld.value && lastSelectedIndex.value !== null && index !== undefined) {
		const start = Math.min(lastSelectedIndex.value, index)
		const end = Math.max(lastSelectedIndex.value, index)
		const rangeIds = props.items.slice(start, end + 1).map((item) => item.id)
		const merged = new Set([...selectedIds.value, ...rangeIds])
		selectedIds.value = [...merged]
	} else if (selected) {
		if (!selectedIds.value.includes(itemId)) {
			selectedIds.value = [...selectedIds.value, itemId]
		}
	} else {
		selectedIds.value = selectedIds.value.filter((id) => id !== itemId)
	}

	if (index !== undefined) {
		lastSelectedIndex.value = index
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
	<div
		role="table"
		class="@container border border-solid border-surface-4 shadow-sm overflow-clip"
		:class="[flat ? '' : 'rounded-[20px]', isStuck || hideHeader ? 'border-t-0' : '']"
	>
		<div
			v-if="!hideHeader"
			ref="stickyHeaderRef"
			role="rowgroup"
			class="sticky top-0 z-10 flex h-12 items-center justify-between gap-4 bg-surface-3 px-3"
			:class="[
				flat || isStuck ? 'rounded-none' : 'rounded-t-[20px]',
				isStuck
					? 'transition-[border-radius] duration-100 border-0 border-y border-solid border-surface-4 shadow-md before:pointer-events-none before:absolute before:inset-x-0 before:-top-4 before:h-5 before:bg-surface-3'
					: '',
			]"
		>
			<div
				role="row"
				class="flex min-w-0 items-center gap-4"
				:class="
					hasAnyActions
						? 'flex-1 @[800px]:w-[350px] @[800px]:shrink-0 @[800px]:flex-none'
						: 'flex-1'
				"
			>
				<Checkbox
					v-if="showSelection"
					:model-value="allSelected"
					:indeterminate="someSelected"
					:aria-label="formatMessage(commonMessages.selectAllLabel)"
					class="shrink-0"
					@update:model-value="toggleSelectAll"
				/>

				<button
					v-if="sortable"
					role="columnheader"
					:aria-sort="
						sortBy === 'project' ? (sortDirection === 'asc' ? 'ascending' : 'descending') : 'none'
					"
					class="flex items-center gap-1.5 font-semibold text-secondary"
					@click="handleSort('project')"
				>
					{{ formatMessage(commonMessages.projectLabel) }}
					<ChevronUpIcon v-if="sortBy === 'project' && sortDirection === 'asc'" class="size-4" />
					<ChevronDownIcon
						v-else-if="sortBy === 'project' && sortDirection === 'desc'"
						class="size-4"
					/>
				</button>
				<span v-else role="columnheader" class="font-semibold text-secondary">{{
					formatMessage(commonMessages.projectLabel)
				}}</span>
			</div>

			<div class="hidden @[800px]:flex" :class="hasAnyActions ? 'flex-1 min-w-0' : 'flex-1'">
				<button
					v-if="sortable"
					role="columnheader"
					:aria-sort="
						sortBy === 'version' ? (sortDirection === 'asc' ? 'ascending' : 'descending') : 'none'
					"
					class="flex items-center gap-1.5 font-semibold text-secondary"
					@click="handleSort('version')"
				>
					{{ formatMessage(commonMessages.versionLabel) }}
					<ChevronUpIcon v-if="sortBy === 'version' && sortDirection === 'asc'" class="size-4" />
					<ChevronDownIcon
						v-else-if="sortBy === 'version' && sortDirection === 'desc'"
						class="size-4"
					/>
				</button>
				<span v-else role="columnheader" class="font-semibold text-secondary">{{
					formatMessage(commonMessages.versionLabel)
				}}</span>
			</div>

			<div v-if="hasAnyActions" role="columnheader" class="min-w-[160px] shrink-0 text-right">
				<span class="font-semibold text-secondary">{{
					formatMessage(commonMessages.actionsLabel)
				}}</span>
			</div>
		</div>

		<div
			v-if="items.length > 0 && virtualized"
			ref="listContainer"
			role="rowgroup"
			class="relative w-full"
			:class="flat ? '' : 'rounded-b-[20px]'"
			:style="{ minHeight: `${totalHeight}px`, overflowAnchor: 'none' }"
		>
			<div class="absolute w-full" :style="{ top: `${visibleTop}px` }">
				<ContentCardItem
					v-for="(item, idx) in visibleItems"
					:key="item.id"
					data-content-card-item
					:project="item.project"
					:project-link="item.projectLink"
					:version="item.version"
					:version-link="item.versionLink"
					:owner="item.owner"
					:enabled="item.enabled"
					:installing="item.installing"
					:has-update="item.hasUpdate"
					:is-client-only="item.isClientOnly"
					:overflow-options="item.overflowOptions"
					:disabled="item.disabled"
					:show-checkbox="showSelection"
					:hide-delete="hideDelete"
					:hide-actions="!hasAnyActions"
					:selected="isItemSelected(item.id)"
					:class="[
						isItemSelected(item.id)
							? 'bg-surface-2.5'
							: (visibleRange.start + idx) % 2 === 1
								? 'bg-surface-1.5'
								: 'bg-surface-2',
						'border-0 border-t border-solid border-surface-4',
						visibleRange.start + idx === items.length - 1 && !flat ? 'rounded-b-[20px]' : '',
					]"
					@update:selected="
						(val) => toggleItemSelection(item.id, val ?? false, visibleRange.start + idx)
					"
					@update:enabled="(val) => emit('update:enabled', item.id, val)"
					@delete="(e: MouseEvent) => emit('delete', item.id, e)"
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

		<div
			v-else-if="items.length > 0"
			ref="listContainer"
			role="rowgroup"
			:class="flat ? '' : 'rounded-b-[20px]'"
		>
			<ContentCardItem
				v-for="(item, index) in items"
				:key="item.id"
				data-content-card-item
				:project="item.project"
				:project-link="item.projectLink"
				:version="item.version"
				:version-link="item.versionLink"
				:owner="item.owner"
				:enabled="item.enabled"
				:installing="item.installing"
				:has-update="item.hasUpdate"
				:overflow-options="item.overflowOptions"
				:disabled="item.disabled"
				:show-checkbox="showSelection"
				:hide-delete="hideDelete"
				:hide-actions="!hasAnyActions"
				:selected="isItemSelected(item.id)"
				:class="[
					isItemSelected(item.id)
						? 'bg-surface-2.5'
						: index % 2 === 1
							? 'bg-surface-1.5'
							: 'bg-surface-2',
					'border-0 border-t border-solid border-surface-4',
					index === items.length - 1 && !flat ? 'rounded-b-[20px]' : '',
				]"
				@update:selected="(val) => toggleItemSelection(item.id, val ?? false, index)"
				@update:enabled="(val) => emit('update:enabled', item.id, val)"
				@delete="(e: MouseEvent) => emit('delete', item.id, e)"
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

		<div
			v-else
			class="flex items-center justify-center py-12"
			:class="flat ? '' : 'rounded-b-[20px]'"
		>
			<slot name="empty">
				<span class="text-secondary">{{ formatMessage(commonMessages.noItemsLabel) }}</span>
			</slot>
		</div>
	</div>
</template>
