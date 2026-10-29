<template>
	<section class="flex flex-col gap-4">
		<h2 class="m-0 text-2xl font-semibold text-contrast">
			{{ formatMessage(messages.title) }}
		</h2>

		<div class="flex flex-col gap-2 md:flex-row md:items-center">
			<Input
				v-model="search"
				:icon="SearchIcon"
				type="search"
				size="medium"
				wrapper-class="min-w-0 md:flex-1"
				:placeholder="formatMessage(messages.searchPlaceholder, { count: backups.length })"
				:aria-label="formatMessage(messages.searchLabel)"
				:clear-label="formatMessage(commonMessages.clearButton)"
				clearable
			/>
			<DropdownFilterBar
				v-model="filters"
				:categories="filterCategories"
				:add-label="formatMessage(messages.addFilter)"
				:clear-label="formatMessage(messages.clearFilters)"
				apply-immediately
				use-filter-icon
				checkbox-position="right"
			/>
		</div>

		<Table
			v-model:selected-ids="tableSelectedIds"
			v-model:sort-column="sortColumn"
			v-model:sort-direction="sortDirection"
			:columns="columns"
			:data="sortedBackups"
			:row-below-visible="showDebugInfo"
			show-selection
			table-min-width="48rem"
		>
			<template #cell-name="{ row: backup }">
				<span class="flex min-w-0 items-center gap-2">
					<span class="truncate text-primary" :title="backup.name">{{ backup.name }}</span>
					<span
						v-if="backup.automated"
						class="inline-flex h-6 shrink-0 items-center rounded-full border border-solid border-blue bg-highlight-blue px-2 text-xs font-semibold text-blue"
					>
						{{ formatMessage(messages.autoBadge) }}
					</span>
				</span>
			</template>

			<template #cell-created="{ row: backup }">
				<span v-tooltip="formatDateTime(backup.created_at)" class="whitespace-nowrap">
					{{ formatRelativeTime(backup.created_at) }}
				</span>
			</template>

			<template #cell-reason="{ row: backup }">
				<span
					v-if="backup.deleted_suspension"
					class="inline-flex h-7 items-center rounded-full border border-solid border-orange bg-highlight-orange px-2.5 text-sm font-semibold text-orange"
				>
					{{ formatMessage(messages.reasonSuspension) }}
				</span>
				<span
					v-else
					class="inline-flex h-7 items-center rounded-full border border-solid border-red bg-highlight-red px-2.5 text-sm font-semibold text-red"
				>
					{{ formatMessage(messages.reasonDeleted) }}
				</span>
			</template>

			<template #cell-recoverableUntil="{ row: backup }">
				<span
					v-if="backup.can_undelete"
					v-tooltip="formatDateTime(backup.undelete_deadline)"
					class="whitespace-nowrap"
				>
					{{ formatRelativeTime(backup.undelete_deadline) }}
				</span>
				<span v-else class="whitespace-nowrap font-semibold text-red">
					{{ formatMessage(messages.unrecoverable) }}
				</span>
			</template>

			<template #cell-actions="{ row: backup }">
				<div class="flex items-center justify-end gap-1">
					<Button
						:disabled="!backup.can_undelete || undeleteMutation.isPending.value"
						:loading="undeletingIds.includes(backup.id)"
						@click="undelete([backup])"
					>
						<UndoIcon aria-hidden="true" />
						{{ formatMessage(messages.undelete) }}
					</Button>
					<IconButton
						v-tooltip="formatMessage(commonMessages.copyIdButton)"
						:label="formatMessage(commonMessages.copyIdButton)"
						type="quiet"
						@click="copyId(backup.id)"
					>
						<ClipboardCopyIcon aria-hidden="true" />
					</IconButton>
				</div>
			</template>

			<template #row-below="{ row: backup }">
				<pre class="m-2 overflow-x-auto rounded-xl bg-surface-3 p-2 text-xs">{{ backup }}</pre>
			</template>

			<template #empty-state>
				<div class="flex h-40 items-center justify-center text-secondary">
					{{ formatMessage(messages.noResults) }}
				</div>
			</template>
		</Table>

		<FloatingActionBar
			:shown="selectedBackups.length > 0"
			:aria-label="formatMessage(messages.selectedCount, { count: selectedBackups.length })"
		>
			<span class="px-4 py-2.5 font-semibold tabular-nums text-contrast">
				{{ formatMessage(messages.selectedCount, { count: selectedBackups.length }) }}
			</span>
			<div class="mx-1 h-6 w-px bg-surface-5" />
			<Button type="quiet" :disabled="undeleteMutation.isPending.value" @click="selectedIds = []">
				{{ formatMessage(commonMessages.clearButton) }}
			</Button>
			<Button
				v-tooltip="bulkUndeleteTooltip"
				type="quiet"
				color="brand"
				interaction="filled"
				class="ml-auto"
				:disabled="selectedRecoverableBackups.length === 0"
				:loading="undeleteMutation.isPending.value"
				@click="undelete(selectedRecoverableBackups)"
			>
				<UndoIcon aria-hidden="true" />
				{{ formatMessage(messages.undelete) }}
			</Button>
		</FloatingActionBar>
	</section>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { ClipboardCopyIcon, SearchIcon, UndoIcon } from '@modrinth/assets'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import type { DropdownFilterBarCategory, SortDirection, TableColumn } from '#ui/components/base'
import { Button, IconButton } from '#ui/components/base/buttons'
import DropdownFilterBar from '#ui/components/base/DropdownFilterBar.vue'
import FloatingActionBar from '#ui/components/base/FloatingActionBar.vue'
import Input from '#ui/components/base/inputs/Input.vue'
import Table from '#ui/components/base/Table.vue'
import { useFormatDateTime } from '#ui/composables/format-date-time'
import { useRelativeTime } from '#ui/composables/how-ago'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'
import { commonMessages } from '#ui/utils/common-messages'

type DeletedBackup = Archon.BackupsQueue.v1.DeletedBackupQueueBackup
type FilterKey = 'reason' | 'type' | 'recoverable'

const props = defineProps<{
	backups: DeletedBackup[]
	showDebugInfo?: boolean
}>()

const selectedIds = defineModel<string[]>('selectedIds', { required: true })
const tableSelectedIds = computed<unknown[]>({
	get: () => selectedIds.value,
	set: (ids) => {
		selectedIds.value = ids.filter((id): id is string => typeof id === 'string')
	},
})

const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()
const formatDateTime = useFormatDateTime({ dateStyle: 'long', timeStyle: 'short' })
const client = injectModrinthClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()
const { serverId, worldId } = injectModrinthServerContext()

const messages = defineMessages({
	title: {
		id: 'servers.backups.deleted.title',
		defaultMessage: 'Deleted backups',
	},
	searchPlaceholder: {
		id: 'servers.backups.deleted.search-placeholder',
		defaultMessage: 'Search {count, plural, one {# deleted backup} other {# deleted backups}}...',
	},
	searchLabel: {
		id: 'servers.backups.deleted.search-label',
		defaultMessage: 'Search deleted backups',
	},
	addFilter: {
		id: 'servers.backups.deleted.filter.add',
		defaultMessage: 'Add filter',
	},
	clearFilters: {
		id: 'servers.backups.deleted.filter.clear',
		defaultMessage: 'Clear filters',
	},
	reasonFilter: {
		id: 'servers.backups.deleted.filter.reason',
		defaultMessage: 'Reason',
	},
	typeFilter: {
		id: 'servers.backups.deleted.filter.type',
		defaultMessage: 'Type',
	},
	typeAuto: {
		id: 'servers.backups.deleted.filter.type.auto',
		defaultMessage: 'Auto',
	},
	typeManual: {
		id: 'servers.backups.deleted.filter.type.manual',
		defaultMessage: 'Manual',
	},
	recoverableFilter: {
		id: 'servers.backups.deleted.filter.recoverable',
		defaultMessage: 'Recoverability',
	},
	recoverable: {
		id: 'servers.backups.deleted.filter.recoverable.yes',
		defaultMessage: 'Recoverable',
	},
	nameColumn: {
		id: 'servers.backups.deleted.column.name',
		defaultMessage: 'Name',
	},
	createdColumn: {
		id: 'servers.backups.deleted.column.created',
		defaultMessage: 'Created',
	},
	reasonColumn: {
		id: 'servers.backups.deleted.column.reason',
		defaultMessage: 'Reason',
	},
	recoverableUntilColumn: {
		id: 'servers.backups.deleted.column.recoverable-until',
		defaultMessage: 'Recoverable until',
	},
	autoBadge: {
		id: 'servers.backups.deleted.auto-badge',
		defaultMessage: 'Auto',
	},
	reasonSuspension: {
		id: 'servers.backups.deleted.reason.suspension',
		defaultMessage: 'Suspension',
	},
	reasonDeleted: {
		id: 'servers.backups.deleted.reason.deleted',
		defaultMessage: 'Manually deleted',
	},
	unrecoverable: {
		id: 'servers.backups.deleted.unrecoverable',
		defaultMessage: 'Unrecoverable',
	},
	undelete: {
		id: 'servers.backups.deleted.undelete',
		defaultMessage: 'Undelete',
	},
	undeleteSuccess: {
		id: 'servers.backups.deleted.undelete.success',
		defaultMessage: '{count, plural, one {Backup undeleted} other {# backups undeleted}}',
	},
	undeleteError: {
		id: 'servers.backups.deleted.undelete.error',
		defaultMessage: 'Failed to undelete {count, plural, one {backup} other {# backups}}',
	},
	selectedCount: {
		id: 'servers.backups.deleted.bulk-bar.selected-count',
		defaultMessage: '{count, plural, one {# backup selected} other {# backups selected}}',
	},
	someUnrecoverable: {
		id: 'servers.backups.deleted.bulk-bar.some-unrecoverable',
		defaultMessage:
			'Only {recoverable} of {count, plural, one {# selected backup} other {# selected backups}} can be undeleted',
	},
	noneRecoverable: {
		id: 'servers.backups.deleted.bulk-bar.none-recoverable',
		defaultMessage: 'None of the selected backups can be undeleted',
	},
	noResults: {
		id: 'servers.backups.deleted.no-results',
		defaultMessage: 'No deleted backups match your search.',
	},
})

const search = ref('')
const filters = ref<Record<FilterKey, string[]>>({
	reason: [],
	type: [],
	recoverable: [],
})

const filterCategories = computed<DropdownFilterBarCategory[]>(() => [
	{
		key: 'reason',
		label: formatMessage(messages.reasonFilter),
		options: [
			{ value: 'suspension', label: formatMessage(messages.reasonSuspension) },
			{ value: 'deleted', label: formatMessage(messages.reasonDeleted) },
		],
	},
	{
		key: 'type',
		label: formatMessage(messages.typeFilter),
		options: [
			{ value: 'auto', label: formatMessage(messages.typeAuto) },
			{ value: 'manual', label: formatMessage(messages.typeManual) },
		],
	},
	{
		key: 'recoverable',
		label: formatMessage(messages.recoverableFilter),
		options: [
			{ value: 'recoverable', label: formatMessage(messages.recoverable) },
			{ value: 'unrecoverable', label: formatMessage(messages.unrecoverable) },
		],
	},
])

function matchesFilter(key: FilterKey, value: string) {
	const selected = filters.value[key] ?? []
	return selected.length === 0 || selected.includes(value)
}

const filteredBackups = computed(() => {
	const term = search.value.trim().toLowerCase()
	return props.backups.filter(
		(backup) =>
			(!term || backup.name.toLowerCase().includes(term) || backup.id.includes(term)) &&
			matchesFilter('reason', backup.deleted_suspension ? 'suspension' : 'deleted') &&
			matchesFilter('type', backup.automated ? 'auto' : 'manual') &&
			matchesFilter('recoverable', backup.can_undelete ? 'recoverable' : 'unrecoverable'),
	)
})

const columns = computed<
	TableColumn<'name' | 'created' | 'reason' | 'recoverableUntil' | 'actions'>[]
>(() => [
	{
		key: 'name',
		label: formatMessage(messages.nameColumn),
		enableSorting: true,
		cellClass: 'pr-4',
	},
	{
		key: 'created',
		label: formatMessage(messages.createdColumn),
		width: '8rem',
		enableSorting: true,
		defaultSortDirection: 'desc',
		cellClass: 'pr-4',
	},
	{
		key: 'reason',
		label: formatMessage(messages.reasonColumn),
		width: '11rem',
		enableSorting: true,
		cellClass: 'pr-4',
	},
	{
		key: 'recoverableUntil',
		label: formatMessage(messages.recoverableUntilColumn),
		width: '10rem',
		enableSorting: true,
		defaultSortDirection: 'desc',
		cellClass: 'pr-4',
	},
	{
		key: 'actions',
		label: formatMessage(commonMessages.actionsLabel),
		width: '12.5rem',
		align: 'right',
	},
])

const sortColumn = ref<string | undefined>('created')
const sortDirection = ref<SortDirection>('desc')

function timeOf(date: string) {
	return new Date(date).getTime()
}

function compareBackups(a: DeletedBackup, b: DeletedBackup) {
	switch (sortColumn.value) {
		case 'name':
			return a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: 'base' })
		case 'reason':
			return Number(b.deleted_suspension) - Number(a.deleted_suspension)
		case 'recoverableUntil':
			return (
				(a.can_undelete ? timeOf(a.undelete_deadline) : 0) -
				(b.can_undelete ? timeOf(b.undelete_deadline) : 0)
			)
		default:
			return timeOf(a.created_at) - timeOf(b.created_at)
	}
}

const sortedBackups = computed(() => {
	const direction = sortDirection.value === 'asc' ? 1 : -1
	return [...filteredBackups.value].sort(
		(a, b) => compareBackups(a, b) * direction || timeOf(b.created_at) - timeOf(a.created_at),
	)
})

const selectedBackups = computed(() =>
	props.backups.filter((backup) => selectedIds.value.includes(backup.id)),
)
const selectedRecoverableBackups = computed(() =>
	selectedBackups.value.filter((backup) => backup.can_undelete),
)

const bulkUndeleteTooltip = computed(() => {
	const recoverable = selectedRecoverableBackups.value.length
	const count = selectedBackups.value.length
	if (recoverable === 0) return formatMessage(messages.noneRecoverable)
	if (recoverable < count) return formatMessage(messages.someUnrecoverable, { recoverable, count })
	return undefined
})

const undeletingIds = ref<string[]>([])
const undeleteMutation = useMutation({
	mutationFn: async (backupIds: string[]) => {
		const results = await Promise.allSettled(
			backupIds.map((backupId) =>
				client.archon.backups_queue_v1.undelete(serverId, worldId.value!, backupId),
			),
		)
		return backupIds.map((id, index) => ({ id, result: results[index] }))
	},
	onSettled: () =>
		Promise.all([
			queryClient.invalidateQueries({ queryKey: ['backups', 'queue', serverId] }),
			queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] }),
		]),
})

function undelete(toUndelete: DeletedBackup[]) {
	const backupIds = toUndelete.filter((backup) => backup.can_undelete).map((backup) => backup.id)
	if (backupIds.length === 0 || !worldId.value) return

	undeletingIds.value = backupIds
	undeleteMutation.mutate(backupIds, {
		onSuccess: (outcomes) => {
			const succeeded = outcomes.filter(({ result }) => result.status === 'fulfilled')
			const failed = outcomes.flatMap(({ result }) =>
				result.status === 'rejected' ? [result.reason] : [],
			)
			const succeededIds = succeeded.map(({ id }) => id)
			selectedIds.value = selectedIds.value.filter((id) => !succeededIds.includes(id))

			if (succeeded.length > 0) {
				addNotification({
					type: 'success',
					title: formatMessage(messages.undeleteSuccess, { count: succeeded.length }),
				})
			}
			if (failed.length > 0) {
				addNotification({
					type: 'error',
					title: formatMessage(messages.undeleteError, { count: failed.length }),
					text: failed[0] instanceof Error ? failed[0].message : String(failed[0]),
				})
			}
		},
		onSettled: () => {
			undeletingIds.value = []
		},
	})
}

function copyId(id: string) {
	navigator.clipboard.writeText(id)
}
</script>
