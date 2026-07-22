<template>
	<div class="flex flex-col gap-4">
		<div class="flex flex-col gap-4">
			<div class="flex items-center gap-2">
				<StyledInput
					v-model="search"
					:icon="SearchIcon"
					:placeholder="`Search ${rows.length} users...`"
					wrapper-class="min-w-0 flex-1"
					input-class="!h-10"
					clearable
				/>
				<template v-if="!actionsLocked">
					<ButtonStyled color="brand">
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
					<ButtonStyled>
						<button
							class="flex !h-10 shrink-0 items-center gap-2"
							:disabled="pushUpdateDisabled"
							@click="emit('push-update', $event)"
						>
							<SpinnerIcon
								v-if="pushUpdatePending"
								class="animate-spin"
								aria-hidden="true"
							/>
							<UploadIcon v-else aria-hidden="true" />
							{{ formatMessage(messages.pushUpdate) }}
						</button>
					</ButtonStyled>
				</template>
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
					:key="option.id"
					:class="filterClass(methodFilter === option.id)"
					:aria-pressed="methodFilter === option.id"
					@click="toggleMethodFilter(option.id)"
				>
					{{ option.label }}
				</button>
			</div>
		</div>

		<Table
			v-model:sort-column="sortColumn"
			v-model:sort-direction="sortDirection"
			:columns="columns"
			:data="sortedRows"
			row-key="id"
			table-min-width="42rem"
			@sort="handleSort"
		>
			<template #empty-state
				><div class="flex h-64 items-center justify-center text-secondary">
					No users match your filters.
				</div></template
			>
			<template #cell-username="{ row }">
				<div class="flex min-w-0 max-w-full items-center gap-2">
					<AutoLink
						v-tooltip="truncatedTooltip(usernameRefs[row.id], row.username)"
						:to="userProfileLink(row.username)"
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
							>{{ row.username }}</span
						>
					</AutoLink>
				</div>
			</template>
			<template #cell-lastPlayed="{ row }">
				<span v-if="row.lastPlayedAt" v-tooltip="formatDateTime(row.lastPlayedAt)">{{
					formatRelativeTime(row.lastPlayedAt)
				}}</span>
				<span v-else>Never</span>
			</template>
			<template #cell-joined="{ row }">
				<span
					v-if="row.pending"
					class="inline-flex h-7 items-center rounded-full border border-surface-5 border-solid bg-surface-4 px-2.5 py-1 text-sm font-semibold text-secondary"
					>Pending</span
				>
				<span v-else-if="row.joinedAt" v-tooltip="formatDateTime(row.joinedAt)">{{
					formatRelativeTime(row.joinedAt)
				}}</span>
			</template>
			<template #cell-method="{ row }">
				<span class="inline-flex min-w-0 items-center gap-2">
					<UserPlusIcon v-if="row.method === 'direct'" class="size-5 shrink-0" aria-hidden="true" />
					<LinkIcon v-else class="size-5 shrink-0" aria-hidden="true" />
					<span class="min-w-0 truncate">{{ methodLabels[row.method] }}</span>
				</span>
			</template>
			<template #cell-actions="{ row }">
				<div v-if="!actionsLocked" class="flex items-center justify-end">
					<ButtonStyled circular type="transparent"
						><button
							v-tooltip="'Revoke access'"
							:aria-label="`Revoke access for ${row.username}`"
							class="text-secondary hover:!filter-none hover:text-red focus-visible:!filter-none"
							@click="emit('remove', row)"
						>
							<XIcon aria-hidden="true" /></button
					></ButtonStyled>
				</div>
			</template>
		</Table>
	</div>
</template>

<script setup lang="ts">
import {
	FilterIcon,
	LinkIcon,
	SearchIcon,
	SpinnerIcon,
	UploadIcon,
	UserPlusIcon,
	XIcon,
} from '@modrinth/assets'
import {
	AutoLink,
	Avatar,
	ButtonStyled,
	defineMessages,
	type SortDirection,
	StyledInput,
	Table,
	type TableColumn,
	truncatedTooltip,
	useFormatDateTime,
	useRelativeTime,
	useVIntl,
} from '@modrinth/ui'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed, ref } from 'vue'

import {
	type MethodFilter,
	methodLabels,
	type ShareMethod,
	type ShareRow,
	type ShareTableColumn,
} from './shared-instance-share-types'

const props = defineProps<{
	rows: ShareRow[]
	actionsLocked?: boolean
	invitePending?: boolean
	pushUpdateDisabled?: boolean
	pushUpdatePending?: boolean
}>()
const emit = defineEmits<{
	invite: [event: MouseEvent]
	remove: [row: ShareRow]
	'push-update': [event: MouseEvent]
}>()
const search = ref('')
const methodFilter = ref<MethodFilter>('all')
const sortColumn = ref<string | undefined>('joined')
const sortDirection = ref<SortDirection>('desc')
const usernameRefs = ref<Record<string, HTMLElement | null>>({})
const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime({ style: 'narrow' })
const formatDateTime = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })
const methodFilterOptions: Array<{ id: ShareMethod; label: string }> = [
	{ id: 'direct', label: methodLabels.direct },
	{ id: 'link', label: methodLabels.link },
]
const columns = computed<TableColumn<ShareTableColumn>[]>(() => {
	const result: TableColumn<ShareTableColumn>[] = [
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
	if (!props.actionsLocked)
		result.push({
			key: 'actions',
			label: 'Actions',
			align: 'right',
			width: 'clamp(5.5rem, 7%, 7rem)',
			headerClass: 'whitespace-nowrap !pl-2 !pr-4',
			cellClass: 'whitespace-nowrap !pl-2 !pr-4',
		})
	return result
})
const filteredRows = computed(() => {
	const query = search.value.trim().toLowerCase()
	return props.rows.filter((row) => {
		if (methodFilter.value !== 'all' && row.method !== methodFilter.value) return false
		if (!query) return true
		return [
			row.username,
			row.lastPlayedAt ? formatRelativeTime(row.lastPlayedAt) : 'Never',
			row.pending ? 'Pending' : row.joinedAt ? formatRelativeTime(row.joinedAt) : '',
			methodLabels[row.method],
		].some((value) => value.toLowerCase().includes(query))
	})
})
const sortedRows = computed(() => [...filteredRows.value].sort(compareRows))

function compareRows(a: ShareRow, b: ShareRow) {
	let compared: number
	if (sortColumn.value === 'username') compared = a.username.localeCompare(b.username)
	else if (sortColumn.value === 'lastPlayed')
		compared =
			(a.lastPlayedAt?.getTime() ?? Number.NEGATIVE_INFINITY) -
			(b.lastPlayedAt?.getTime() ?? Number.NEGATIVE_INFINITY)
	else if (sortColumn.value === 'method')
		compared = methodLabels[a.method].localeCompare(methodLabels[b.method])
	else
		compared =
			(a.pending ? Number.MAX_SAFE_INTEGER : (a.joinedAt?.getTime() ?? Number.NEGATIVE_INFINITY)) -
				(b.pending
					? Number.MAX_SAFE_INTEGER
					: (b.joinedAt?.getTime() ?? Number.NEGATIVE_INFINITY)) ||
			a.username.localeCompare(b.username)
	return sortDirection.value === 'asc' ? compared : -compared
}
function handleSort(column: string, direction: SortDirection) {
	sortColumn.value = column
	sortDirection.value = direction
}
function toggleMethodFilter(filter: ShareMethod) {
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

const messages = defineMessages({
	pushUpdate: {
		id: 'app.instance.admonitions.shared-instance.publish-button',
		defaultMessage: 'Push update',
	},
})
function userProfileLink(username: string) {
	return !username || username.includes('@')
		? undefined
		: () => openUrl(`https://modrinth.com/user/${encodeURIComponent(username)}`)
}
function setUsernameRef(id: string, element: Element | null) {
	usernameRefs.value[id] = element instanceof HTMLElement ? element : null
}
</script>
