<script setup lang="ts">
import { ChevronDownIcon, ChevronUpIcon, InfoIcon } from '@modrinth/assets'
import { computed, getCurrentInstance, ref, toRef } from 'vue'

import Checkbox from '#ui/components/base/Checkbox.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
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

const messages = defineMessages({
	enabledFor: {
		id: 'content.enabled-for.label',
		defaultMessage: 'Enabled for',
	},
	enabledForDescription: {
		id: 'content.enabled-for.description',
		defaultMessage:
			'Choose whether this content runs on the server, is sent to players, or is disabled in both places.',
	},
	sortEnabledFor: {
		id: 'content.enabled-for.sort',
		defaultMessage: 'Sort by where content is enabled',
	},
})

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
	showItemActions?: boolean
	showEnabledForColumn?: boolean
	enabledForSortDirection?: ContentCardTableSortDirection
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
	showItemActions: false,
	showEnabledForColumn: false,
	enabledForSortDirection: undefined,
})

const stickyHeaderRef = ref<HTMLElement | null>(null)
const { isStuck } = useStickyObserver(stickyHeaderRef, 'ContentCardTable')

const selectedIds = defineModel<string[]>('selectedIds', { default: () => [] })

const emit = defineEmits<{
	'update:enabled': [id: string, value: boolean]
	'update:enabled-for': [id: string, side: 'server' | 'player', value: boolean]
	delete: [id: string, event: MouseEvent]
	update: [id: string]
	switchVersion: [id: string]
	sort: [column: ContentCardTableSortColumn, direction: ContentCardTableSortDirection]
	'sort-enabled-for': [direction: ContentCardTableSortDirection]
}>()

// Check if any actions are available
const instance = getCurrentInstance()
const hasDeleteListener = computed(() => typeof instance?.vnode.props?.onDelete === 'function')
const hasUpdateListener = computed(() => typeof instance?.vnode.props?.onUpdate === 'function')
const hasSwitchVersionListener = computed(
	() => typeof instance?.vnode.props?.onSwitchVersion === 'function',
)
const hasEnabledListener = computed(
	() => typeof instance?.vnode.props?.['onUpdate:enabled'] === 'function',
)
const hasEnabledForColumn = computed(
	() => props.showEnabledForColumn || props.items.some((item) => item.enabledFor !== undefined),
)

const hasAnyActions = computed(() => {
	// Check if there are listeners for actions
	const hasListeners =
		(hasDeleteListener.value &&
			props.items.some((item) => !props.hideDelete && !item.hideDelete)) ||
		hasUpdateListener.value ||
		hasSwitchVersionListener.value ||
		hasEnabledListener.value

	// Check if any items have overflow options or updates
	const hasItemActions = props.items.some(
		(item) =>
			(item.overflowOptions && item.overflowOptions.length > 0) ||
			item.hasUpdate ||
			(item.enabled !== undefined && !item.hideToggle),
	)

	return hasListeners || hasItemActions || props.showItemActions
})

// Virtualization
const itemHeight = computed(() => (hasEnabledForColumn.value ? 72 : 74))
const { listContainer, totalHeight, visibleRange, visibleTop, visibleItems } = useVirtualScroll(
	toRef(props, 'items'),
	{
		itemHeight,
		bufferSize: 5,
		initialItemCount: 20,
		enabled: toRef(props, 'virtualized'),
	},
)

// Expose for perf monitoring
defineExpose({
	visibleRange,
	visibleItems,
})

// Selection logic
const selectableItems = computed(() => props.items.filter((item) => !item.disabled))

const allSelected = computed(() => {
	if (selectableItems.value.length === 0) return false
	return selectableItems.value.every((item) => selectedIds.value.includes(item.id))
})

const someSelected = computed(() => {
	return (
		selectableItems.value.some((item) => selectedIds.value.includes(item.id)) && !allSelected.value
	)
})

function toggleSelectAll() {
	if (allSelected.value || someSelected.value) {
		selectedIds.value = []
	} else {
		selectedIds.value = selectableItems.value.map((item) => item.id)
	}
}

const lastSelectedIndex = ref<number | null>(null)

function toggleItemSelection(
	itemId: string,
	selected: boolean,
	index?: number,
	event?: MouseEvent,
) {
	if (selected && event?.shiftKey && lastSelectedIndex.value !== null && index !== undefined) {
		const start = Math.min(lastSelectedIndex.value, index)
		const end = Math.max(lastSelectedIndex.value, index)
		const rangeIds = props.items
			.slice(start, end + 1)
			.filter((item) => !item.disabled)
			.map((item) => item.id)
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

function handleEnabledForSort() {
	const newDirection: ContentCardTableSortDirection =
		props.enabledForSortDirection === 'desc' ? 'asc' : 'desc'
	emit('sort-enabled-for', newDirection)
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
						? hasEnabledForColumn
							? 'flex-1 @[800px]:w-[340px] @[800px]:shrink-0 @[800px]:flex-none'
							: 'flex-1 @[800px]:w-[45%] @[800px]:shrink-0 @[800px]:flex-none'
						: 'flex-1'
				"
			>
				<Checkbox
					v-if="showSelection"
					:model-value="allSelected"
					:indeterminate="someSelected"
					:aria-label="formatMessage(commonMessages.selectAllLabel)"
					:disabled="selectableItems.length === 0"
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

			<div
				v-if="hasEnabledForColumn"
				role="columnheader"
				:aria-sort="
					enabledForSortDirection
						? enabledForSortDirection === 'asc'
							? 'ascending'
							: 'descending'
						: 'none'
				"
				class="hidden w-[200px] shrink-0 items-center gap-1.5 font-semibold text-secondary @[800px]:flex"
			>
				<button
					type="button"
					class="cursor-pointer border-0 bg-transparent p-0 font-semibold text-secondary"
					@click="handleEnabledForSort"
				>
					{{ formatMessage(messages.enabledFor) }}
				</button>
				<span
					v-tooltip="formatMessage(messages.enabledForDescription)"
					class="inline-flex size-4 cursor-help items-center justify-center"
					tabindex="0"
				>
					<InfoIcon class="size-4" />
				</span>
				<button
					type="button"
					class="flex cursor-pointer items-center border-0 bg-transparent p-0 text-secondary"
					:aria-label="formatMessage(messages.sortEnabledFor)"
					@click="handleEnabledForSort"
				>
					<ChevronUpIcon v-if="enabledForSortDirection === 'asc'" class="size-4" />
					<ChevronDownIcon v-else class="size-4" />
				</button>
			</div>

			<div
				class="hidden @[800px]:flex"
				:class="
					hasAnyActions
						? hasEnabledForColumn
							? 'w-[250px] min-w-0 shrink-0'
							: 'flex-1 min-w-0'
						: 'flex-1'
				"
			>
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

			<div
				v-if="hasAnyActions"
				role="columnheader"
				class="shrink-0 text-right"
				:class="hasEnabledForColumn ? 'w-[112px]' : 'min-w-[160px]'"
			>
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
					:source="item.source"
					:external="item.external"
					:enabled="item.enabled"
					:locked="item.locked"
					:installing="item.installing"
					:install-progress="item.installProgress"
					:has-update="item.hasUpdate"
					:is-client-only="item.isClientOnly"
					:client-warning="item.clientWarning"
					:hide-switch-version="item.hideSwitchVersion"
					:overflow-options="item.overflowOptions"
					:disabled="item.disabled"
					:disabled-tooltip="item.disabledTooltip"
					:toggle-disabled="item.toggleDisabled"
					:toggle-disabled-tooltip="item.toggleDisabledTooltip"
					:hide-toggle="item.hideToggle"
					:enabled-for="item.enabledFor"
					:embedded-icon="item.embeddedIcon"
					:show-checkbox="showSelection"
					:hide-delete="hideDelete || item.hideDelete"
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
					@select="
						(val, event) =>
							toggleItemSelection(item.id, val ?? false, visibleRange.start + idx, event)
					"
					@update:enabled="(val) => emit('update:enabled', item.id, val)"
					@update:enabled-for="
						(side, val) => emit('update:enabled-for', item.id, side, val)
					"
					@delete="(e: MouseEvent) => emit('delete', item.id, e)"
					@update="emit('update', item.id)"
					v-on="
						hasSwitchVersionListener ? { switchVersion: () => emit('switchVersion', item.id) } : {}
					"
				>
					<template #title-badges>
						<slot name="itemTitleBadges" :item="item" :index="visibleRange.start + idx" />
					</template>
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
				:source="item.source"
				:external="item.external"
				:enabled="item.enabled"
				:locked="item.locked"
				:installing="item.installing"
				:install-progress="item.installProgress"
				:has-update="item.hasUpdate"
				:is-client-only="item.isClientOnly"
				:client-warning="item.clientWarning"
				:hide-switch-version="item.hideSwitchVersion"
				:overflow-options="item.overflowOptions"
				:disabled="item.disabled"
				:disabled-tooltip="item.disabledTooltip"
				:toggle-disabled="item.toggleDisabled"
				:toggle-disabled-tooltip="item.toggleDisabledTooltip"
				:hide-toggle="item.hideToggle"
				:enabled-for="item.enabledFor"
				:embedded-icon="item.embeddedIcon"
				:show-checkbox="showSelection"
				:hide-delete="hideDelete || item.hideDelete"
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
				@select="(val, event) => toggleItemSelection(item.id, val ?? false, index, event)"
				@update:enabled="(val) => emit('update:enabled', item.id, val)"
				@update:enabled-for="
					(side, val) => emit('update:enabled-for', item.id, side, val)
				"
				@delete="(e: MouseEvent) => emit('delete', item.id, e)"
				@update="emit('update', item.id)"
				@switch-version="emit('switchVersion', item.id)"
			>
				<template #title-badges>
					<slot name="itemTitleBadges" :item="item" :index="index" />
				</template>
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
