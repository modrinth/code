<template>
	<div class="flex flex-col gap-4">
		<div v-if="variant === 'share'" class="flex flex-col gap-4">
			<div class="flex items-center gap-2">
				<StyledInput
					v-model="search"
					:icon="SearchIcon"
					:placeholder="`Search ${rows.length} users...`"
					wrapper-class="min-w-0 flex-1"
					input-class="!h-10"
					clearable
				/>
				<ButtonStyled v-if="!actionsLocked" color="brand">
					<button
						class="flex !h-10 shrink-0 items-center gap-2"
						:disabled="invitePending"
						@click="emit('invite', $event)"
					>
						<SpinnerIcon v-if="invitePending" class="animate-spin" aria-hidden="true" />
						<UserPlusIcon v-else aria-hidden="true" />
						Invite friends
					</button>
				</ButtonStyled>
			</div>
			<div class="flex flex-wrap items-center gap-1.5">
				<FilterIcon class="size-5 shrink-0 text-secondary" aria-hidden="true" />
				<button
					:class="filterClass(methodFilter === 'all')"
					:aria-pressed="methodFilter === 'all'"
					@click="methodFilter = 'all'"
				>
					All
				</button>
				<button
					v-for="option in methodFilterOptions"
					:key="option.value"
					:class="filterClass(methodFilter === option.value)"
					:aria-pressed="methodFilter === option.value"
					@click="toggleMethodFilter(option.value)"
				>
					{{ option.label }}
				</button>
			</div>
		</div>

		<div v-else class="flex flex-col gap-2 md:flex-row">
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
				:options="methodFilterOptionsWithAll"
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
			:table-min-width="variant === 'server' ? '50rem' : '42rem'"
			@sort="handleSort"
		>
			<template #empty-state>
				<div class="flex h-64 items-center justify-center px-4 text-center text-secondary">
					No users match your filters.
				</div>
			</template>
			<template #cell-username="{ row }">
				<div class="flex min-w-0 max-w-full items-center gap-2">
					<AutoLink
						v-tooltip="truncatedTooltip(usernameRefs[row.id], row.username)"
						:to="userProfileLink?.(row.username)"
						class="inline-flex max-w-full min-w-0 items-center gap-2 text-primary hover:underline"
					>
						<Avatar
							:src="row.avatarUrl"
							:alt="`${row.username}'s avatar`"
							:tint-by="row.username"
							size="24px"
							circle
							no-shadow
						/>
						<span
							:ref="(element) => setUsernameRef(row.id, element)"
							class="min-w-0 truncate font-medium"
						>
							{{ row.username }}
						</span>
					</AutoLink>
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
					<UserPlusIcon v-if="row.method === 'direct'" class="size-5 shrink-0" aria-hidden="true" />
					<LinkIcon v-else class="size-5 shrink-0" aria-hidden="true" />
					<span class="min-w-0 truncate">{{ methodLabel(row.method) }}</span>
				</span>
			</template>
			<template #cell-actions="{ row }">
				<div v-if="!actionsLocked" class="flex items-center justify-end">
					<ButtonStyled circular type="transparent">
						<button
							v-tooltip="actionLabel(row)"
							:aria-label="`${actionLabel(row)} for ${row.username}`"
							class="text-secondary hover:!filter-none focus-visible:!filter-none"
							:class="variant === 'share' || row.pending ? 'hover:text-red' : ''"
							@click="handleAction(row)"
						>
							<XIcon v-if="variant === 'share' || row.pending" aria-hidden="true" />
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
	SpinnerIcon,
	UserPlusIcon,
	XIcon,
} from '@modrinth/assets'
import { computed, ref } from 'vue'

import AutoLink from '#ui/components/base/AutoLink.vue'
import Avatar from '#ui/components/base/Avatar.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import Combobox, { type ComboboxOption } from '#ui/components/base/Combobox.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'
import Table, { type SortDirection, type TableColumn } from '#ui/components/base/Table.vue'
import { useFormatDateTime, useRelativeTime } from '#ui/composables'
import { truncatedTooltip } from '#ui/utils'

import {
	type SharedPlayersTableColumn,
	type SharedPlayersTableMethod,
	type SharedPlayersTableMethodFilter,
	sharedPlayersTableMethodLabels,
	type SharedPlayersTableRow,
	type SharedPlayersTableUserProfileLink,
} from './shared-players-table-types'

const props = withDefaults(
	defineProps<{
		rows: SharedPlayersTableRow[]
		actionsLocked?: boolean
		invitePending?: boolean
		variant?: 'share' | 'server'
		userProfileLink?: (username: string) => SharedPlayersTableUserProfileLink
	}>(),
	{
		actionsLocked: false,
		invitePending: false,
		variant: 'share',
		userProfileLink: undefined,
	},
)

const emit = defineEmits<{
	invite: [event: MouseEvent]
	remove: [row: SharedPlayersTableRow]
	openActions: [row: SharedPlayersTableRow]
}>()

const search = ref('')
const methodFilter = ref<SharedPlayersTableMethodFilter>('all')
const sortColumn = ref<string | undefined>('joined')
const sortDirection = ref<SortDirection>('desc')
const usernameRefs = ref<Record<string, HTMLElement | null>>({})
const formatRelativeTime = useRelativeTime({ style: 'narrow' })
const formatDateTime = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })

const methodFilterOptions = computed<ComboboxOption<SharedPlayersTableMethod>[]>(() => [
	{ value: 'direct', label: sharedPlayersTableMethodLabels.direct },
	{ value: 'link', label: methodLabel('link') },
])

const methodFilterOptionsWithAll = computed<ComboboxOption<SharedPlayersTableMethodFilter>[]>(
	() => [{ value: 'all', label: 'All' }, ...methodFilterOptions.value],
)

const selectedMethodFilterLabel = computed(
	() =>
		`Method: ${
			methodFilterOptionsWithAll.value.find((option) => option.value === methodFilter.value)
				?.label ?? 'All'
		}`,
)

const columns = computed<TableColumn<SharedPlayersTableColumn>[]>(() => {
	const result: TableColumn<SharedPlayersTableColumn>[] = [
		{
			key: 'username',
			label: 'Username',
			width: 'clamp(14rem, 30%, 26rem)',
			enableSorting: true,
			headerClass: '!pr-3',
			cellClass: '!pr-3',
		},
		{
			key: 'lastPlayed',
			label: 'Last played',
			width: 'clamp(7rem, 15%, 13rem)',
			enableSorting: true,
			headerClass: 'whitespace-nowrap !px-2',
			cellClass: 'whitespace-nowrap !px-2',
		},
		{
			key: 'joined',
			label: 'Joined',
			width: 'clamp(7rem, 14%, 12rem)',
			enableSorting: true,
			defaultSortDirection: 'desc',
			headerClass: 'whitespace-nowrap !px-2',
			cellClass: 'whitespace-nowrap !px-2',
		},
		{
			key: 'method',
			label: 'Method',
			enableSorting: true,
			headerClass: 'whitespace-nowrap !px-2',
			cellClass: 'whitespace-nowrap !px-2',
		},
	]

	if (!props.actionsLocked) {
		result.push({
			key: 'actions',
			label: 'Actions',
			align: 'right',
			width: 'clamp(5.5rem, 7%, 7rem)',
			headerClass: 'whitespace-nowrap !pl-2 !pr-4',
			cellClass: 'whitespace-nowrap !pl-2 !pr-4',
		})
	}

	return result
})

const filteredRows = computed(() => {
	const query = search.value.trim().toLowerCase()
	return props.rows.filter((row) => {
		if (methodFilter.value !== 'all' && row.method !== methodFilter.value) return false
		if (!query) return true

		return [
			row.username,
			row.lastPlayedAt ? formatPlayerDate(row.lastPlayedAt) : 'Never',
			row.pending ? 'Pending' : row.joinedAt ? formatPlayerDate(row.joinedAt) : '',
			methodLabel(row.method),
		].some((value) => value.toLowerCase().includes(query))
	})
})

const sortedRows = computed(() => [...filteredRows.value].sort(compareRows))

function methodLabel(method: SharedPlayersTableMethod) {
	if (method === 'link' && props.variant === 'server') return 'Shareable link'
	return sharedPlayersTableMethodLabels[method]
}

function formatPlayerDate(date: Date) {
	if (props.variant === 'server') {
		const today = new Date()
		if (
			date.getFullYear() === today.getFullYear() &&
			date.getMonth() === today.getMonth() &&
			date.getDate() === today.getDate()
		) {
			return 'Today'
		}
	}

	return formatRelativeTime(date)
}

function compareRows(a: SharedPlayersTableRow, b: SharedPlayersTableRow) {
	let compared: number
	if (sortColumn.value === 'username') {
		compared = a.username.localeCompare(b.username)
	} else if (sortColumn.value === 'lastPlayed') {
		compared =
			(a.lastPlayedAt?.getTime() ?? Number.NEGATIVE_INFINITY) -
			(b.lastPlayedAt?.getTime() ?? Number.NEGATIVE_INFINITY)
	} else if (sortColumn.value === 'method') {
		compared = methodLabel(a.method).localeCompare(methodLabel(b.method))
	} else {
		compared =
			(a.pending ? Number.MAX_SAFE_INTEGER : (a.joinedAt?.getTime() ?? Number.NEGATIVE_INFINITY)) -
				(b.pending
					? Number.MAX_SAFE_INTEGER
					: (b.joinedAt?.getTime() ?? Number.NEGATIVE_INFINITY)) ||
			a.username.localeCompare(b.username)
	}

	return sortDirection.value === 'asc' ? compared : -compared
}

function handleSort(column: string, direction: SortDirection) {
	sortColumn.value = column
	sortDirection.value = direction
}

function toggleMethodFilter(filter: SharedPlayersTableMethod) {
	methodFilter.value = methodFilter.value === filter ? 'all' : filter
}

function filterClass(active: boolean) {
	return [
		'cursor-pointer rounded-full border border-solid px-3 py-1.5 text-base font-semibold leading-5 transition-all duration-100 active:scale-[0.97]',
		active
			? 'border-green bg-brand-highlight text-brand'
			: 'border-surface-5 bg-surface-4 text-primary hover:bg-surface-5',
	]
}

function actionLabel(row: SharedPlayersTableRow) {
	if (props.variant === 'share') return 'Revoke access'
	return row.pending ? 'Revoke invite' : 'More actions'
}

function handleAction(row: SharedPlayersTableRow) {
	if (props.variant === 'share' || row.pending) emit('remove', row)
	else emit('openActions', row)
}

function setUsernameRef(id: string, element: Element | null) {
	usernameRefs.value[id] = element instanceof HTMLElement ? element : null
}
</script>
