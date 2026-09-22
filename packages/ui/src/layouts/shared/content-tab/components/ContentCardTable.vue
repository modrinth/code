<script setup lang="ts">
import { ChevronDownIcon, ChevronUpIcon, InfoIcon } from '@modrinth/assets'
import { useElementSize } from '@vueuse/core'
import { computed, getCurrentInstance, ref, toRef, watch } from 'vue'

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
import ContentEnabledFor from './ContentEnabledFor.vue'

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

interface Props {
	items: ContentCardTableItem[]
	highlightedItemId?: string
	showSelection?: boolean
	sortable?: boolean
	sortBy?: ContentCardTableSortColumn
	sortDirection?: ContentCardTableSortDirection
	virtualized?: boolean
	hideDelete?: boolean
	hideHeader?: boolean
	flat?: boolean
	showItemActions?: boolean
	showVersion?: boolean
	showEnabledForColumn?: boolean
	getAdditionalActionWidths?: (item: ContentCardTableItem) => number[]
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
	showVersion: true,
	showEnabledForColumn: false,
	getAdditionalActionWidths: () => [],
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

const enabledForMeasureRef = ref<HTMLElement | null>(null)
const actionsLabelRef = ref<HTMLElement | null>(null)
const { width: enabledForWidth } = useElementSize(enabledForMeasureRef)
const { width: actionsLabelWidth } = useElementSize(actionsLabelRef)

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
			item.synced ||
			item.locked ||
			props.getAdditionalActionWidths(item).length > 0 ||
			(item.enabled !== undefined && !item.hideToggle),
	)

	return hasListeners || hasItemActions || props.showItemActions
})

const actionsWidth = computed(() =>
	props.items.reduce((maximum, item) => {
		const widths = props.getAdditionalActionWidths(item).slice()
		if (item.synced) widths.push(42)
		if (
			item.locked ||
			(hasUpdateListener.value && item.hasUpdate) ||
			(hasSwitchVersionListener.value && item.version && !item.hideSwitchVersion)
		) {
			widths.push(36)
		}
		if (item.enabled !== undefined && !item.hideToggle) widths.push(48)
		if (hasDeleteListener.value && !props.hideDelete && !item.hideDelete) widths.push(36)
		if (item.overflowOptions?.length) widths.push(36)
		const width = widths.reduce((sum, width) => sum + width, 0)
		return Math.max(maximum, width + Math.max(0, widths.length - 1) * 8)
	}, actionsLabelWidth.value),
)

function getItemListeners(id: string) {
	return {
		...(hasDeleteListener.value ? { delete: (event: MouseEvent) => emit('delete', id, event) } : {}),
		...(hasUpdateListener.value ? { update: () => emit('update', id) } : {}),
		...(hasSwitchVersionListener.value ? { switchVersion: () => emit('switchVersion', id) } : {}),
	}
}

const tableRef = ref<HTMLElement | null>(null)
const { width: tableWidth } = useElementSize(tableRef)
const controlColumns = computed(() => [
	...(hasEnabledForColumn.value ? [Math.ceil(enabledForWidth.value)] : []),
	...(hasAnyActions.value ? [Math.ceil(actionsWidth.value)] : []),
])
const layout = computed(() => {
	const controls = controlColumns.value
	const controlBudget = controls.reduce((sum, width) => sum + width, 0)
	const compactBudget = 24 + controlBudget + controls.length * 16 + 240
	const wideBudget = compactBudget + (props.showVersion ? 288 + 16 : 0)
	if (tableWidth.value >= wideBudget) return 'wide'
	if (tableWidth.value >= compactBudget) return 'compact'
	if (tableWidth.value >= 24 + controlBudget + Math.max(0, controls.length - 1) * 16) {
		return 'stacked'
	}
	return 'narrow'
})
const separateVersion = computed(() => props.showVersion && layout.value === 'wide')
const stacked = computed(() => layout.value === 'stacked' || layout.value === 'narrow')
const contentColumnStyles = computed(() => {
	const controls = controlColumns.value.map((width) => `${width}px`)
	let columns: string[]
	if (layout.value === 'narrow') {
		columns = ['minmax(0, 1fr)']
	} else if (stacked.value) {
		columns = controls.length > 1 ? [controls[0], 'minmax(0, 1fr)'] : ['minmax(0, 1fr)']
	} else {
		columns = [
			'minmax(0, 1.2fr)',
			...(hasEnabledForColumn.value ? [controls[0]] : []),
			...(separateVersion.value ? ['minmax(0, 1fr)'] : []),
			...(hasAnyActions.value ? [`${Math.ceil(actionsWidth.value)}px`] : []),
		]
	}
	return {
		'--content-columns': columns.join(' '),
		'--content-row-height': `${itemHeight.value}px`,
	}
})
const itemHeight = computed(() => {
	if (!stacked.value) return 74
	const controlRows = layout.value === 'narrow' ? controlColumns.value.length : Math.min(1, controlColumns.value.length)
	return 25 + 48 + controlRows * 44
})
const { listContainer, totalHeight, visibleRange, visibleTop, visibleItems, scrollToIndex } =
	useVirtualScroll(toRef(props, 'items'), {
		itemHeight,
		bufferSize: 5,
		initialItemCount: 20,
		enabled: toRef(props, 'virtualized'),
	})

watch(
	[() => props.items.findIndex((item) => item.id === props.highlightedItemId), listContainer],
	([index, container], _, onCleanup) => {
		if (index < 0 || !container) return
		const frame = requestAnimationFrame(() => scrollToIndex(index))
		onCleanup(() => cancelAnimationFrame(frame))
	},
	{ flush: 'post' },
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
</script>

<template>
	<div
		ref="tableRef"
		role="table"
		class="@container relative border border-solid border-surface-4 shadow-sm overflow-clip"
		:class="[flat ? '' : 'rounded-[20px]', isStuck || hideHeader ? 'border-t-0' : '']"
		:style="contentColumnStyles"
	>
		<div aria-hidden="true" inert class="pointer-events-none invisible absolute left-0 top-0 w-max">
			<div
				v-if="hasEnabledForColumn"
				ref="enabledForMeasureRef"
				class="flex w-max flex-col items-start"
			>
				<span class="flex items-center gap-1.5 whitespace-nowrap font-semibold">
					{{ formatMessage(messages.enabledFor) }}
					<InfoIcon class="size-4 shrink-0" />
				</span>
				<ContentEnabledFor
					:model-value="{ server: false, player: false, locked: false }"
					reserve-status-space
				/>
			</div>
			<span v-if="hasAnyActions" ref="actionsLabelRef" class="block w-max font-semibold">
				{{ formatMessage(commonMessages.actionsLabel) }}
			</span>
		</div>
		<div
			v-if="!hideHeader"
			ref="stickyHeaderRef"
			role="rowgroup"
			class="sticky top-0 z-10 grid grid-cols-[var(--content-columns)] items-center gap-x-4 bg-surface-3 px-3"
			:class="[
				stacked ? 'gap-y-2 py-2' : 'h-12',
				flat || isStuck ? 'rounded-none' : 'rounded-t-[20px]',
				isStuck
					? 'transition-[border-radius] duration-100 border-0 border-y border-solid border-surface-4 shadow-md before:pointer-events-none before:absolute before:inset-x-0 before:-top-4 before:h-5 before:bg-surface-3'
					: '',
			]"
		>
			<div
				role="row"
				class="flex min-w-0 items-center gap-4"
				:class="{ 'col-span-full': stacked }"
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
				class="flex min-w-0 items-center gap-1.5 whitespace-nowrap font-semibold text-secondary"
			>
				<span>{{ formatMessage(messages.enabledFor) }}</span>
				<span
					v-tooltip="formatMessage(messages.enabledForDescription)"
					class="inline-flex size-4 cursor-help items-center justify-center"
					tabindex="0"
				>
					<InfoIcon class="size-4" />
				</span>
			</div>

			<div v-if="separateVersion" class="min-w-0">
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
				:class="layout === 'narrow' ? 'justify-self-start' : 'justify-self-end'"
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
					:data-content-card-item="item.id"
					:project="item.project"
					:project-link="item.projectLink"
					:version="item.version"
					:show-version="showVersion"
				:table-layout="layout"
				:enabled-for-column="hasEnabledForColumn"
					:version-link="item.versionLink"
					:owner="item.owner"
					:source="item.source"
					:external="item.external"
					:external-file="item.externalFile"
					:enabled="item.enabled"
					:locked="item.locked"
					:installing="item.installing"
					:install-progress="item.installProgress"
					:has-update="item.hasUpdate"
					:is-client-only="item.isClientOnly"
					:client-warning="item.clientWarning"
					:synced="item.synced"
					:sync-update-pending="item.syncUpdatePending"
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
						item.id === highlightedItemId
							? 'outline outline-2 -outline-offset-2 outline-brand'
							: '',
						visibleRange.start + idx === items.length - 1 && !flat ? 'rounded-b-[20px]' : '',
					]"
					@select="
						(val, event) =>
							toggleItemSelection(item.id, val ?? false, visibleRange.start + idx, event)
					"
					@update:enabled="(val) => emit('update:enabled', item.id, val)"
					@update:enabled-for="(side, val) => emit('update:enabled-for', item.id, side, val)"
					v-on="getItemListeners(item.id)"
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
				:data-content-card-item="item.id"
				:project="item.project"
				:project-link="item.projectLink"
				:version="item.version"
				:show-version="showVersion"
				:table-layout="layout"
				:enabled-for-column="hasEnabledForColumn"
				:version-link="item.versionLink"
				:owner="item.owner"
				:source="item.source"
				:external="item.external"
				:external-file="item.externalFile"
				:enabled="item.enabled"
				:locked="item.locked"
				:installing="item.installing"
				:install-progress="item.installProgress"
				:has-update="item.hasUpdate"
				:is-client-only="item.isClientOnly"
				:client-warning="item.clientWarning"
				:synced="item.synced"
				:sync-update-pending="item.syncUpdatePending"
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
					item.id === highlightedItemId ? 'outline outline-2 -outline-offset-2 outline-brand' : '',
					index === items.length - 1 && !flat ? 'rounded-b-[20px]' : '',
				]"
				@select="(val, event) => toggleItemSelection(item.id, val ?? false, index, event)"
				@update:enabled="(val) => emit('update:enabled', item.id, val)"
				@update:enabled-for="(side, val) => emit('update:enabled-for', item.id, side, val)"
				v-on="getItemListeners(item.id)"
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
