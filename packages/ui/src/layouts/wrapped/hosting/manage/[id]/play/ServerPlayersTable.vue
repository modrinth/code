<template>
	<div class="flex flex-col gap-4">
		<div class="flex flex-col gap-2 md:flex-row">
			<StyledInput
				v-model="search"
				:icon="SearchIcon"
				:placeholder="`Search ${rows.length} users...`"
				wrapper-class="min-w-0 flex-1"
				input-class="!h-10"
				clearable
			/>
			<Combobox
				v-model="methodFilter"
				class="md:!w-auto md:shrink-0"
				:options="methodFilterOptions"
				:display-value="selectedMethodFilterLabel"
				trigger-class="min-w-[176px] !h-10 !min-h-10 !py-0"
			>
				<template #prefix>
					<FilterIcon class="size-5 text-secondary" aria-hidden="true" />
				</template>
			</Combobox>
		</div>

		<Table
			v-model:sort-column="sortColumn"
			v-model:sort-direction="sortDirection"
			:columns="columns"
			:data="sortedRows"
			row-key="id"
			table-min-width="50rem"
			@sort="handleSort"
		>
			<template #empty-state>
				<div class="flex h-64 items-center justify-center px-4 text-center text-secondary">
					No users match your filters.
				</div>
			</template>
			<template #cell-username="{ row }">
				<div class="flex min-w-0 max-w-full items-center gap-2">
					<Avatar
						:src="row.avatarUrl"
						:alt="`${row.username}'s avatar`"
						:tint-by="row.username"
						size="24px"
						circle
						no-shadow
					/>
					<span class="min-w-0 truncate font-medium text-primary">{{ row.username }}</span>
				</div>
			</template>
			<template #cell-lastPlayed="{ row }">
				<span v-if="row.lastPlayedAt" v-tooltip="formatDateTime(row.lastPlayedAt)">
					{{ formatPlayerDate(row.lastPlayedAt) }}
				</span>
				<span v-else>Never</span>
			</template>
			<template #cell-joined="{ row }">
				<span
					v-if="row.pending"
					class="inline-flex h-7 items-center rounded-full border border-solid border-surface-5 bg-surface-4 px-2.5 py-1 text-sm font-semibold text-secondary"
				>
					Pending
				</span>
				<span v-else-if="row.joinedAt" v-tooltip="formatDateTime(row.joinedAt)">
					{{ formatPlayerDate(row.joinedAt) }}
				</span>
			</template>
			<template #cell-method="{ row }">
				<span class="inline-flex min-w-0 items-center gap-2">
					<UserPlusIcon v-if="row.method === 'direct'" class="size-5 shrink-0" />
					<LinkIcon v-else class="size-5 shrink-0" />
					<span class="min-w-0 truncate">{{ methodLabel(row.method) }}</span>
				</span>
			</template>
			<template #cell-actions="{ row }">
				<div class="flex items-center justify-end">
					<ButtonStyled circular type="transparent">
						<button
							v-tooltip="row.pending ? 'Revoke invite' : 'More actions'"
							:aria-label="`${row.pending ? 'Revoke invite' : 'More actions'} for ${row.username}`"
							class="text-secondary hover:!filter-none"
							:class="row.pending ? 'hover:text-red' : ''"
							@click="row.pending ? emit('remove', row) : emit('open-actions', row)"
						>
							<XIcon v-if="row.pending" aria-hidden="true" />
							<MoreHorizontalIcon v-else aria-hidden="true" />
						</button>
					</ButtonStyled>
				</div>
			</template>
		</Table>
	</div>
</template>

<script setup lang="ts">
import {
	FilterIcon,
	LinkIcon,
	MoreHorizontalIcon,
	SearchIcon,
	UserPlusIcon,
	XIcon,
} from '@modrinth/assets'
import { computed, ref } from 'vue'

import Avatar from '#ui/components/base/Avatar.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import Combobox, { type ComboboxOption } from '#ui/components/base/Combobox.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'
import Table, { type SortDirection, type TableColumn } from '#ui/components/base/Table.vue'
import { useFormatDateTime, useRelativeTime } from '#ui/composables'

import type { ServerPlayerMethod, ServerPlayerRow } from './types'

type MethodFilter = ServerPlayerMethod | 'all'
type PlayerColumn = 'username' | 'lastPlayed' | 'joined' | 'method' | 'actions'

const props = defineProps<{ rows: ServerPlayerRow[] }>()
const emit = defineEmits<{
	remove: [row: ServerPlayerRow]
	'open-actions': [row: ServerPlayerRow]
}>()

const search = ref('')
const methodFilter = ref<MethodFilter>('all')
const sortColumn = ref<string | undefined>('joined')
const sortDirection = ref<SortDirection>('desc')
const formatRelativeTime = useRelativeTime({ style: 'narrow' })
const formatDateTime = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })

const methodFilterOptions: ComboboxOption<MethodFilter>[] = [
	{ value: 'all', label: 'All' },
	{ value: 'direct', label: 'Direct invite' },
	{ value: 'link', label: 'Shareable link' },
]
const selectedMethodFilterLabel = computed(
	() =>
		`Method: ${
			methodFilterOptions.find((option) => option.value === methodFilter.value)?.label ?? 'All'
		}`,
)
const columns: TableColumn<PlayerColumn>[] = [
	{ key: 'username', label: 'Username', enableSorting: true },
	{ key: 'lastPlayed', label: 'Last played', enableSorting: true },
	{ key: 'joined', label: 'Joined', enableSorting: true, defaultSortDirection: 'desc' },
	{ key: 'method', label: 'Method', enableSorting: true },
	{ key: 'actions', label: 'Actions', align: 'right' },
]

const filteredRows = computed(() => {
	const query = search.value.trim().toLowerCase()
	return props.rows.filter((row) => {
		if (methodFilter.value !== 'all' && row.method !== methodFilter.value) return false
		if (!query) return true
		return [row.username, methodLabel(row.method)].some((value) =>
			value.toLowerCase().includes(query),
		)
	})
})
const sortedRows = computed(() => [...filteredRows.value].sort(compareRows))

function methodLabel(method: ServerPlayerMethod) {
	return method === 'link' ? 'Shareable link' : 'Direct invite'
}

function formatPlayerDate(date: Date) {
	const today = new Date()
	if (
		date.getFullYear() === today.getFullYear() &&
		date.getMonth() === today.getMonth() &&
		date.getDate() === today.getDate()
	) {
		return 'Today'
	}
	return formatRelativeTime(date)
}

function compareRows(a: ServerPlayerRow, b: ServerPlayerRow) {
	let compared: number
	if (sortColumn.value === 'username') compared = a.username.localeCompare(b.username)
	else if (sortColumn.value === 'lastPlayed') {
		compared =
			(a.lastPlayedAt?.getTime() ?? Number.NEGATIVE_INFINITY) -
			(b.lastPlayedAt?.getTime() ?? Number.NEGATIVE_INFINITY)
	} else if (sortColumn.value === 'method') {
		compared = methodLabel(a.method).localeCompare(methodLabel(b.method))
	} else {
		compared =
			(a.pending ? Number.MAX_SAFE_INTEGER : (a.joinedAt?.getTime() ?? Number.NEGATIVE_INFINITY)) -
				(b.pending ? Number.MAX_SAFE_INTEGER : (b.joinedAt?.getTime() ?? Number.NEGATIVE_INFINITY))
	}
	return sortDirection.value === 'asc' ? compared : -compared
}

function handleSort(column: string, direction: SortDirection) {
	sortColumn.value = column
	sortDirection.value = direction
}
</script>
