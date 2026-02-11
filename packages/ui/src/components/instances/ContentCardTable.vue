<script setup lang="ts">
import { ChevronDownIcon, ChevronUpIcon } from '@modrinth/assets'
import { computed, getCurrentInstance, toRef } from 'vue'

import { useVIntl } from '../../composables/i18n'
import { useVirtualScroll } from '../../composables/virtual-scroll'
import { commonMessages } from '../../utils/common-messages'
import Checkbox from '../base/Checkbox.vue'
import ContentCardItem from './ContentCardItem.vue'
import type {
	ContentCardTableItem,
	ContentCardTableSortColumn,
	ContentCardTableSortDirection,
} from './types'

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
	isStuck?: boolean
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
	isStuck: false,
})

const selectedIds = defineModel<string[]>('selectedIds', { default: () => [] })

const emit = defineEmits<{
	'update:enabled': [id: string, value: boolean]
	delete: [id: string]
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
	<div
		class="@container border border-solid border-surface-4 shadow-sm"
		:class="[flat ? '' : 'rounded-[20px]', isStuck || hideHeader ? 'border-t-0' : '']"
	>
		<div
			v-if="!hideHeader"
			class="sticky top-0 z-10 flex h-12 items-center justify-between gap-4 bg-surface-3 px-3"
			:class="[
				flat || isStuck ? 'rounded-none' : 'rounded-t-[20px]',
				isStuck
					? 'transition-[border-radius] duration-100 border-0 border-y border-solid border-surface-4 shadow-md before:pointer-events-none before:absolute before:inset-x-0 before:-top-4 before:h-5 before:bg-surface-3'
					: '',
			]"
		>
			<div
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
					class="shrink-0"
					@update:model-value="toggleSelectAll"
				/>

				<button
					v-if="sortable"
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
				<span v-else class="font-semibold text-secondary">{{
					formatMessage(commonMessages.projectLabel)
				}}</span>
			</div>

			<div class="hidden @[800px]:flex" :class="hasAnyActions ? 'w-[335px] min-w-0' : 'flex-1'">
				<button
					v-if="sortable"
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
				<span v-else class="font-semibold text-secondary">{{
					formatMessage(commonMessages.versionLabel)
				}}</span>
			</div>

			<div v-if="hasAnyActions" class="min-w-[160px] shrink-0 text-right">
				<span class="font-semibold text-secondary">{{
					formatMessage(commonMessages.actionsLabel)
				}}</span>
			</div>
		</div>

		<div
			v-if="items.length > 0 && virtualized"
			ref="listContainer"
			class="relative w-full"
			:class="flat ? '' : 'rounded-b-[20px]'"
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
					:version-link="item.versionLink"
					:owner="item.owner"
					:enabled="item.enabled"
					:has-update="item.hasUpdate"
					:overflow-options="item.overflowOptions"
					:disabled="item.disabled"
					:show-checkbox="showSelection"
					:hide-delete="hideDelete"
					:hide-actions="!hasAnyActions"
					:selected="isItemSelected(item.id)"
					:class="[
						(visibleRange.start + idx) % 2 === 1 ? 'bg-surface-1.5' : 'bg-surface-2',
						'border-0 border-t border-solid border-surface-4',
						visibleRange.start + idx === items.length - 1 && !flat ? 'rounded-b-[20px]' : '',
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

		<div v-else-if="items.length > 0" ref="listContainer" :class="flat ? '' : 'rounded-b-[20px]'">
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
				:has-update="item.hasUpdate"
				:overflow-options="item.overflowOptions"
				:disabled="item.disabled"
				:show-checkbox="showSelection"
				:hide-delete="hideDelete"
				:hide-actions="!hasAnyActions"
				:selected="isItemSelected(item.id)"
				:class="[
					index % 2 === 1 ? 'bg-surface-1.5' : 'bg-surface-2',
					'border-0 border-t border-solid border-surface-4',
					index === items.length - 1 && !flat ? 'rounded-b-[20px]' : '',
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
