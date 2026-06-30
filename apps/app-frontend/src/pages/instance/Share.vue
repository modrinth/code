<template>
	<div class="flex flex-col gap-4">
		<GrantAccessModal
			ref="grantAccessModal"
			:members="accessMembers"
			:friend-ids="[]"
			:search-users="searchInviteUsers"
			@grant="grantAccess"
		/>

		<template v-if="rows.length > 0">
			<div class="flex flex-col gap-4">
				<div class="flex items-center gap-2">
					<StyledInput
						v-model="memberSearch"
						:icon="SearchIcon"
						:placeholder="`Search ${rows.length} users...`"
						wrapper-class="min-w-0 flex-1"
						input-class="!h-10"
						clearable
					/>
					<ButtonStyled color="brand">
						<button
							class="flex !h-10 shrink-0 items-center gap-2"
							@click="grantAccessModal?.show($event)"
						>
							<UserPlusIcon aria-hidden="true" />
							Invite friends
						</button>
					</ButtonStyled>
				</div>

				<div class="flex flex-wrap items-center gap-1.5">
					<FilterIcon class="size-5 shrink-0 text-secondary" aria-hidden="true" />
					<button
						class="cursor-pointer rounded-full border border-solid px-3 py-1.5 text-base font-semibold leading-5 transition-all duration-100 active:scale-[0.97]"
						:class="
							methodFilter === 'all'
								? 'border-green bg-brand-highlight text-brand'
								: 'border-surface-5 bg-surface-4 text-primary hover:bg-surface-5'
						"
						:aria-pressed="methodFilter === 'all'"
						@click="methodFilter = 'all'"
					>
						All
					</button>
					<button
						v-for="option in methodFilterOptions"
						:key="option.id"
						class="cursor-pointer rounded-full border border-solid px-3 py-1.5 text-base font-semibold leading-5 transition-all duration-100 active:scale-[0.97]"
						:class="
							methodFilter === option.id
								? 'border-green bg-brand-highlight text-brand'
								: 'border-surface-5 bg-surface-4 text-primary hover:bg-surface-5'
						"
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
				@sort="handleTableSort"
			>
				<template #empty-state>
					<div class="flex h-64 items-center justify-center text-secondary">
						No users match your filters.
					</div>
				</template>

				<template #cell-username="{ row }">
					<div class="flex min-w-0 max-w-full items-center gap-2">
						<AutoLink
							:to="userProfileLink(row.username)"
							v-tooltip="truncatedTooltip(usernameRefs[row.id], row.username)"
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
						{{ formatCompactRelativeTime(row.lastPlayedAt) }}
					</span>
					<span v-else>Never</span>
				</template>

				<template #cell-joined="{ row }">
					<span
						v-if="row.pending"
						class="inline-flex h-7 items-center rounded-full border border-surface-5 border-solid bg-surface-4 px-2.5 py-1 text-sm font-semibold text-secondary"
					>
						Pending
					</span>
					<span v-else-if="row.joinedAt" v-tooltip="formatDateTime(row.joinedAt)">
						{{ formatCompactRelativeTime(row.joinedAt) }}
					</span>
				</template>

				<template #cell-method="{ row }">
					<span class="inline-flex min-w-0 items-center gap-2">
						<UserPlusIcon v-if="row.method === 'direct'" class="size-5 shrink-0" aria-hidden="true" />
						<LinkIcon v-else class="size-5 shrink-0" aria-hidden="true" />
						<span class="min-w-0 truncate">{{ compactMethodLabels[row.method] }}</span>
					</span>
				</template>

				<template #cell-actions="{ row }">
					<div class="flex items-center justify-end">
						<ButtonStyled v-if="row.pending" circular type="transparent">
							<button
								v-tooltip="'Revoke invite'"
								:aria-label="`Revoke invite for ${row.username}`"
								class="text-secondary hover:!filter-none hover:text-red focus-visible:!filter-none"
								@click="removeRow(row.id)"
							>
								<XIcon aria-hidden="true" />
							</button>
						</ButtonStyled>
						<ButtonStyled v-else circular type="transparent">
							<OverflowMenu
								:options="[
									{
										id: 'remove-user',
										action: () => removeRow(row.id),
										color: 'red',
									},
								]"
							>
								<MoreVerticalIcon aria-hidden="true" />
								<span class="sr-only">Actions for {{ row.username }}</span>
								<template #remove-user>
									<XIcon aria-hidden="true" />
									Revoke access
								</template>
							</OverflowMenu>
						</ButtonStyled>
					</div>
				</template>
			</Table>
		</template>

		<EmptyState v-else type="empty-inbox">
			<template #heading>No friends invited</template>
			<template #description>You can share an instance with your friends!</template>
			<template #actions>
				<ButtonStyled color="brand">
					<button class="!h-10" @click="grantAccessModal?.show($event)">
						<UserPlusIcon aria-hidden="true" />
						Invite friends
					</button>
				</ButtonStyled>
			</template>
		</EmptyState>
	</div>
</template>

<script setup lang="ts">
import {
	FilterIcon,
	LinkIcon,
	MoreVerticalIcon,
	SearchIcon,
	UserPlusIcon,
	XIcon,
} from '@modrinth/assets'
import {
	AutoLink,
	Avatar,
	ButtonStyled,
	EmptyState,
	GrantAccessModal,
	type GrantServerAccessPayload,
	OverflowMenu,
	StyledInput,
	Table,
	type SortDirection,
	type TableColumn,
	type ServerAccessInviteSuggestion,
	type ServerAccessMember,
	truncatedTooltip,
	useFormatDateTime,
	useRelativeTime,
} from '@modrinth/ui'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed, ref } from 'vue'

import { search_user } from '@/helpers/users.js'

type ShareMethod = 'direct' | 'link'
type MethodFilter = ShareMethod | 'all'
type ShareTableColumn = 'username' | 'lastPlayed' | 'joined' | 'method' | 'actions'

type ShareRow = {
	id: string
	username: string
	avatarUrl?: string
	lastPlayedAt: Date | null
	joinedAt: Date | null
	method: ShareMethod
	pending?: boolean
}

const HOUR_MS = 60 * 60 * 1000
const DAY_MS = 24 * HOUR_MS
const now = Date.now()

const grantAccessModal = ref<InstanceType<typeof GrantAccessModal> | null>(null)
const memberSearch = ref('')
const methodFilter = ref<MethodFilter>('all')
const sortColumn = ref<string | undefined>('joined')
const sortDirection = ref<SortDirection>('desc')
const usernameRefs = ref<Record<string, HTMLElement | null>>({})

const rows = ref<ShareRow[]>([
	{
		id: 'coolbot',
		username: 'Coolbot',
		lastPlayedAt: null,
		joinedAt: null,
		method: 'direct',
		pending: true,
	},
	{
		id: 'geometrically',
		username: 'Geometrically',
		lastPlayedAt: new Date(now - 50 * 60 * 1000),
		joinedAt: new Date(now - 2 * HOUR_MS),
		method: 'link',
	},
	{
		id: 'josh',
		username: 'Josh',
		lastPlayedAt: new Date(now - 1 * HOUR_MS),
		joinedAt: new Date(now - 3 * HOUR_MS),
		method: 'link',
	},
	{
		id: 'boris',
		username: 'Boris',
		lastPlayedAt: new Date(now - 4 * DAY_MS),
		joinedAt: new Date(now - 7 * DAY_MS),
		method: 'direct',
	},
	{
		id: 'prospector',
		username: 'Prospector',
		lastPlayedAt: new Date(now - 4 * DAY_MS),
		joinedAt: new Date(now - 30 * DAY_MS),
		method: 'direct',
	},
	{
		id: 'imb',
		username: 'IMB',
		lastPlayedAt: new Date(now - 14 * DAY_MS),
		joinedAt: new Date(now - 90 * DAY_MS),
		method: 'direct',
	},
])

const formatCompactRelativeTime = useRelativeTime({ style: 'narrow' })
const formatDateTime = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })

const methodLabels: Record<ShareMethod, string> = {
	direct: 'Direct invite',
	link: 'Share link',
}

const compactMethodLabels: Record<ShareMethod, string> = {
	direct: 'Direct invite',
	link: 'Share link',
}

const columns: TableColumn<ShareTableColumn>[] = [
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
	{
		key: 'actions',
		label: 'Actions',
		align: 'right',
		width: 'clamp(5.5rem, 7%, 7rem)',
		headerClass: 'whitespace-nowrap !pl-2 !pr-4',
		cellClass: 'whitespace-nowrap !pl-2 !pr-4',
	},
]

const methodFilterOptions = computed<Array<{ id: ShareMethod; label: string }>>(() => [
	{ id: 'direct', label: methodLabels.direct },
	{ id: 'link', label: methodLabels.link },
])

const accessMembers = computed<ServerAccessMember[]>(() =>
	rows.value.map((row) => ({
		id: row.id,
		user: {
			id: row.id,
			username: row.username,
			avatarUrl: row.avatarUrl,
		},
		role: 'viewer',
		joinedAt: row.joinedAt?.toISOString() ?? null,
		pending: row.pending,
	})),
)

const filteredRows = computed(() => {
	const normalizedSearch = memberSearch.value.trim().toLowerCase()

	return rows.value.filter((row) => {
		if (methodFilter.value !== 'all' && row.method !== methodFilter.value) return false
		if (!normalizedSearch) return true

		return [
			row.username,
			formattedLastPlayed(row),
			formattedJoined(row),
			methodLabels[row.method],
			compactMethodLabels[row.method],
		]
			.some((value) => value.toLowerCase().includes(normalizedSearch))
	})
})

const sortedRows = computed(() => [...filteredRows.value].sort(compareRows))

function compareRows(a: ShareRow, b: ShareRow) {
	const column = sortColumn.value
	const direction = sortDirection.value

	switch (column) {
		case 'username':
			return compareString(a.username, b.username, direction)
		case 'lastPlayed':
			return compareDate(a.lastPlayedAt, b.lastPlayedAt, direction)
		case 'method':
			return compareString(compactMethodLabels[a.method], compactMethodLabels[b.method], direction)
		default:
			return compareJoined(a, b, direction)
	}
}

function compareJoined(a: ShareRow, b: ShareRow, direction: SortDirection) {
	const compared = joinedSortValue(a) - joinedSortValue(b) || a.username.localeCompare(b.username)
	return direction === 'asc' ? compared : -compared
}

function joinedSortValue(row: ShareRow) {
	return row.pending ? Number.MAX_SAFE_INTEGER : row.joinedAt?.getTime() ?? Number.NEGATIVE_INFINITY
}

function compareDate(a: Date | null, b: Date | null, direction: SortDirection) {
	const compared = compareDateValue(a, b)
	return direction === 'asc' ? compared : -compared
}

function compareDateValue(a: Date | null, b: Date | null) {
	const aTime = a?.getTime() ?? Number.NEGATIVE_INFINITY
	const bTime = b?.getTime() ?? Number.NEGATIVE_INFINITY
	return aTime - bTime
}

function compareString(a: string, b: string, direction: SortDirection) {
	const compared = a.localeCompare(b)
	return direction === 'asc' ? compared : -compared
}

function handleTableSort(column: string, direction: SortDirection) {
	sortColumn.value = column
	sortDirection.value = direction
}

function toggleMethodFilter(filter: ShareMethod) {
	methodFilter.value = methodFilter.value === filter ? 'all' : filter
}

async function searchInviteUsers(query: string): Promise<ServerAccessInviteSuggestion[]> {
	const users = await search_user(query)

	return users.map((user) => ({
		id: user.id,
		username: user.username,
		avatarUrl: user.avatar_url || undefined,
	}))
}

function grantAccess(payload: GrantServerAccessPayload) {
	const user = payload.user
	const existingRow = rows.value.find(
		(row) =>
			row.id.toLowerCase() === user.id.toLowerCase() ||
			row.username.toLowerCase() === user.username.toLowerCase(),
	)
	if (existingRow) return

	rows.value = [
		{
			id: user.id,
			username: user.username,
			avatarUrl: user.avatarUrl,
			lastPlayedAt: null,
			joinedAt: null,
			method: 'direct',
			pending: true,
		},
		...rows.value,
	]
}

function removeRow(id: string) {
	rows.value = rows.value.filter((row) => row.id !== id)
}

function userProfileLink(username: string) {
	if (!username || username.includes('@')) return undefined
	return () => openUrl(`https://modrinth.com/user/${encodeURIComponent(username)}`)
}

function setUsernameRef(id: string, element: Element | null) {
	const nextElement = element instanceof HTMLElement ? element : null
	if (usernameRefs.value[id] === nextElement) return

	usernameRefs.value = {
		...usernameRefs.value,
		[id]: nextElement,
	}
}

function formattedLastPlayed(row: ShareRow) {
	return row.lastPlayedAt ? formatCompactRelativeTime(row.lastPlayedAt) : 'Never'
}

function formattedJoined(row: ShareRow) {
	return row.pending ? 'Pending' : row.joinedAt ? formatCompactRelativeTime(row.joinedAt) : ''
}
</script>
