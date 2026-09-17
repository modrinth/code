<template>
	<div class="flex min-w-0 flex-col gap-4">
		<div class="flex flex-col gap-2">
			<div class="flex items-center gap-2">
				<Input
					v-model="search"
					:icon="SearchIcon"
					:placeholder="`Search ${rows.length} users...`"
					wrapper-class="min-w-0 flex-1"
					size="medium"
					clearable
				/>
				<slot name="toolbar-actions" />
			</div>
			<div v-if="hasMultipleMethods" class="flex flex-wrap items-center gap-1.5">
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
		>
			<template #empty-state>
				<div class="flex h-64 items-center justify-center px-4 text-center text-secondary">
					{{
						formatMessage(rows.length === 0 ? messages.noUsersJoined : messages.noUsersMatchFilters)
					}}
				</div>
			</template>
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
				<span class="inline-flex min-w-0 max-w-full items-center gap-2">
					<UserPlusIcon v-if="row.method === 'direct'" class="size-5 shrink-0" aria-hidden="true" />
					<LinkIcon v-else class="size-5 shrink-0" aria-hidden="true" />
					<span class="min-w-0 truncate">{{ methodLabels[row.method] }}</span>
				</span>
			</template>
			<template #cell-actions="{ row }">
				<div v-if="canManage" class="flex items-center justify-end">
					<IconButton
						:disabled="disabled"
						v-tooltip="'Revoke access'"
						type="quiet"
						:label="`Revoke access for ${row.username}`"
						class="text-secondary hover:!filter-none hover:text-red focus-visible:!filter-none"
						@click="emit('remove', row)"
					>
						<XIcon aria-hidden="true"
					/></IconButton>
				</div>
			</template>
		</Table>
	</div>
</template>

<script setup lang="ts">
import { FilterIcon, LinkIcon, SearchIcon, UserPlusIcon, XIcon } from '@modrinth/assets'
import { computed, ref, toRef } from 'vue'

import AutoLink from '#ui/components/base/AutoLink.vue'
import Avatar from '#ui/components/base/Avatar.vue'
import { IconButton } from '#ui/components/base/buttons'
import Input from '#ui/components/base/inputs/Input.vue'
import Table, { type TableColumn } from '#ui/components/base/Table.vue'
import { useFormatDateTime, useRelativeTime } from '#ui/composables'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { truncatedTooltip } from '#ui/utils/truncate'

import { useInvitedPlayersTable } from './composables/use-invited-players-table'
import { invitedPlayerMethodLabels as methodLabels, type InvitedPlayerRow } from './types'

type InvitedPlayerColumn = 'username' | 'lastPlayed' | 'joined' | 'method' | 'actions'

const props = defineProps<{
	rows: InvitedPlayerRow[]
	canManage?: boolean
	disabled?: boolean
}>()
const emit = defineEmits<{
	remove: [row: InvitedPlayerRow]
}>()
const usernameRefs = ref<Record<string, HTMLElement | null>>({})
const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime({ style: 'narrow' })
const formatDateTime = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })
const {
	search,
	methodFilter,
	sortColumn,
	sortDirection,
	methodFilterOptions,
	hasMultipleMethods,
	sortedRows,
	toggleMethodFilter,
} = useInvitedPlayersTable(toRef(props, 'rows'), formatRelativeTime)

const columns = computed<TableColumn<InvitedPlayerColumn>[]>(() => {
	const result: TableColumn<InvitedPlayerColumn>[] = [
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
	if (props.canManage)
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

function filterClass(active: boolean) {
	return [
		'cursor-pointer rounded-full border border-solid px-3 py-1.5 text-base font-semibold leading-5 transition-all duration-100 active:scale-[0.97]',
		active
			? 'border-brand bg-brand-highlight text-brand'
			: 'border-surface-5 bg-surface-4 text-primary hover:bg-surface-5',
	]
}

const messages = defineMessages({
	noUsersJoined: {
		id: 'app.instance.share.members.empty',
		defaultMessage: 'No users have joined yet',
	},
	noUsersMatchFilters: {
		id: 'app.instance.share.members.no-filter-results',
		defaultMessage: 'No users match your filters.',
	},
})
function userProfileLink(username: string) {
	return !username || username.includes('@') ? undefined : `/user/${encodeURIComponent(username)}`
}
function setUsernameRef(id: string, element: unknown) {
	usernameRefs.value[id] = element instanceof HTMLElement ? element : null
}
</script>
