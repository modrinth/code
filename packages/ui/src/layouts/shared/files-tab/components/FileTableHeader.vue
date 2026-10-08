<template>
	<div
		class="sticky top-[var(--files-table-header-top,0px)] z-10 flex h-[3rem] w-full select-none flex-row items-center justify-between bg-surface-3 pl-3 pr-3 font-medium transition-[border-radius] duration-100"
		:class="[
			isStuck
				? 'rounded-none border-0 border-y border-solid border-surface-4 shadow-md before:pointer-events-none before:absolute before:inset-x-0 before:-top-4 before:h-5 before:bg-surface-3'
				: '',
			props.advancedView ? 'border-0 border-t border-solid border-surface-5' : '',
		]"
	>
		<div class="flex min-w-0 flex-1 items-center gap-3">
			<Checkbox
				:model-value="allSelected"
				:indeterminate="someSelected && !allSelected"
				@update:model-value="$emit('toggle-all')"
			/>
			<button
				class="flex appearance-none items-center gap-1.5 border-0 bg-transparent p-0 font-semibold hover:text-primary"
				:class="sortField === 'name' ? 'text-contrast' : 'text-secondary'"
				@click="$emit('sort', 'name')"
			>
				<span>{{ formatMessage(messages.name) }}</span>
				<ChevronUpIcon
					v-if="sortField === 'name' && !sortDesc"
					class="h-4 w-4"
					aria-hidden="true"
				/>
				<ChevronDownIcon
					v-if="sortField === 'name' && sortDesc"
					class="h-4 w-4"
					aria-hidden="true"
				/>
			</button>
		</div>
		<div class="flex shrink-0 items-center gap-6">
			<button
				v-for="column in shownColumnDefinitions"
				:key="column.id"
				class="flex appearance-none items-center justify-start gap-1 border-0 bg-transparent p-0 font-semibold hover:text-primary"
				:class="sortField === column.sortField ? 'text-contrast' : 'text-secondary'"
				:style="{ width: `${column.width}px` }"
				@click="$emit('sort', column.sortField)"
			>
				<span class="truncate">{{ formatMessage(columnMessages[column.id]) }}</span>
				<ChevronUpIcon
					v-if="sortField === column.sortField && !sortDesc"
					class="h-4 w-4 shrink-0"
					aria-hidden="true"
				/>
				<ChevronDownIcon
					v-if="sortField === column.sortField && sortDesc"
					class="h-4 w-4 shrink-0"
					aria-hidden="true"
				/>
			</button>
			<div class="flex min-w-[51px] shrink-0 justify-end">
				<TeleportOverflowMenu
					v-if="viewSettingsEnabled"
					v-tooltip="formatMessage(messages.viewSettings)"
					type="quiet"
					:label="formatMessage(messages.viewSettings)"
					:options="columnOptions"
				>
					<SettingsIcon class="h-5 w-5" aria-hidden="true" />
				</TeleportOverflowMenu>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ChevronDownIcon, ChevronUpIcon, SettingsIcon } from '@modrinth/assets'
import { computed } from 'vue'

import type { ButtonMenuOption } from '#ui/components/base/buttons'
import { TeleportOverflowMenu } from '#ui/components/base/buttons'
import Checkbox from '#ui/components/base/Checkbox.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import { FILE_COLUMNS, type FileColumn } from '../composables/file-columns'
import type { FileSortField } from '../types'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	name: {
		id: 'files.table-header.name',
		defaultMessage: 'Name',
	},
	showDetails: {
		id: 'files.table-header.show-details',
		defaultMessage: 'Show details',
	},
	viewSettings: {
		id: 'files.table-header.view-settings',
		defaultMessage: 'View settings',
	},
	advancedView: {
		id: 'files.table-header.advanced-view',
		defaultMessage: 'Advanced view',
	},
	coloredIcons: {
		id: 'files.table-header.colored-icons',
		defaultMessage: 'Colored icons',
	},
})

const columnMessages = defineMessages({
	size: {
		id: 'files.table-header.size',
		defaultMessage: 'Size',
	},
	items: {
		id: 'files.table-header.items',
		defaultMessage: 'Items',
	},
	created: {
		id: 'files.table-header.created',
		defaultMessage: 'Created',
	},
	modified: {
		id: 'files.table-header.modified',
		defaultMessage: 'Modified',
	},
})

const props = defineProps<{
	sortField: FileSortField
	sortDesc: boolean
	allSelected: boolean
	someSelected: boolean
	isStuck: boolean
	/** Columns currently rendered, after fitting them to the available width. */
	columns: FileColumn[]
	/** Columns the user has chosen to show. */
	enabledColumns: FileColumn[]
	detailsEnabled: boolean
	/** Whether the column picker is offered. */
	columnsAdjustable?: boolean
	/** Whether the view settings menu is offered at all. */
	viewSettingsEnabled?: boolean
	advancedView?: boolean
	coloredIcons?: boolean
}>()

const emit = defineEmits<{
	sort: [field: FileSortField]
	'toggle-all': []
	'toggle-column': [column: FileColumn]
	'toggle-details': []
	'toggle-advanced-view': []
	'toggle-colored-icons': []
}>()

const shownColumnDefinitions = computed(() =>
	props.columns.flatMap((id) => FILE_COLUMNS.filter((column) => column.id === id)),
)

const columnOptions = computed<ButtonMenuOption[]>(() => [
	{
		id: 'advanced-view',
		label: formatMessage(messages.advancedView),
		selected: props.advancedView,
		remainOpen: true,
		action: () => emit('toggle-advanced-view'),
	},
	{
		id: 'colored-icons',
		label: formatMessage(messages.coloredIcons),
		selected: props.coloredIcons,
		remainOpen: true,
		action: () => emit('toggle-colored-icons'),
	},
	{ type: 'divider', shown: props.columnsAdjustable },
	{
		shown: props.columnsAdjustable,
		id: 'show-details',
		label: formatMessage(messages.showDetails),
		selected: props.detailsEnabled,
		remainOpen: true,
		action: () => emit('toggle-details'),
	},
	{ type: 'divider', shown: props.columnsAdjustable },
	...FILE_COLUMNS.map(
		(column): ButtonMenuOption => ({
			shown: props.columnsAdjustable,
			id: `column-${column.id}`,
			label: formatMessage(columnMessages[column.id]),
			selected: props.enabledColumns.includes(column.id),
			disabled: !props.detailsEnabled,
			remainOpen: true,
			action: () => emit('toggle-column', column.id),
		}),
	),
])
</script>
