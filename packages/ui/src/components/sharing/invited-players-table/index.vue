<template>
	<div class="flex min-w-0 flex-col gap-4">
		<div class="flex flex-col gap-2">
			<div class="flex flex-col gap-2 sm:flex-row sm:items-center">
				<Input
					v-model="search"
					:icon="SearchIcon"
					:placeholder="formatMessage(messages.searchUsers, { count: rows.length })"
					wrapper-class="w-full min-w-0 sm:flex-1"
					size="medium"
					clearable
				/>
				<div v-if="canManage" class="flex gap-2">
					<Button
						v-if="showPushUpdate"
						type="outlined"
						size="lg"
						class="min-w-0 flex-1 !border sm:flex-none"
						:disabled="pushUpdateDisabled || pushUpdatePending"
						@click="emit('push-update', $event)"
						@mouseenter="emit('push-update-hover')"
						@focus="emit('push-update-hover')"
					>
						<SpinnerIcon v-if="pushUpdatePending" class="animate-spin" aria-hidden="true" />
						<UploadIcon v-else aria-hidden="true" />
						{{ formatMessage(messages.pushUpdate) }}
					</Button>
					<Button
						type="colored"
						color="brand"
						size="lg"
						class="min-w-0 flex-1 sm:flex-none"
						:disabled="inviteDisabled || invitePending"
						@click="emit('invite', $event)"
					>
						<SpinnerIcon v-if="invitePending" class="animate-spin" aria-hidden="true" />
						<UserPlusIcon v-else aria-hidden="true" />
						{{ inviteLabel ?? formatMessage(messages.invitePlayers) }}
					</Button>
				</div>
			</div>
			<div v-if="hasMultipleMethods" class="flex flex-wrap items-center gap-1.5">
				<FilterIcon class="size-5 shrink-0 text-secondary" aria-hidden="true" />
				<button
					:class="filterClass(methodFilter === 'all')"
					:aria-pressed="methodFilter === 'all'"
					@click="methodFilter = 'all'"
				>
					{{ formatMessage(messages.all) }}
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

		<div
			v-if="sortedRows.length === 0"
			class="flex h-64 items-center justify-center gap-2 rounded-2xl border border-solid border-surface-4 bg-surface-2 px-4 text-center text-secondary sm:hidden"
		>
			<SpinnerIcon v-if="loading" class="size-6 animate-spin" aria-hidden="true" />
			{{ emptyStateMessage }}
		</div>
		<div v-else class="flex flex-col gap-3 sm:hidden">
			<div
				v-for="row in sortedRows"
				:key="row.id"
				class="flex min-w-0 flex-col gap-3 rounded-2xl border border-solid border-surface-4 bg-surface-2 p-4"
			>
				<div class="flex min-w-0 items-center justify-between gap-2">
					<AutoLink
						v-tooltip="row.username"
						:to="userProfileLink(row.username)"
						class="inline-flex min-w-0 items-center gap-2 text-primary hover:underline"
					>
						<Avatar
							:src="row.avatarUrl"
							:alt="formatMessage(messages.avatarAlt, { username: row.username })"
							:tint-by="row.username"
							size="24px"
							circle
							no-shadow
						/>
						<span class="min-w-0 truncate font-medium">{{ row.username }}</span>
					</AutoLink>
					<IconButton
						v-if="canManage"
						v-tooltip="formatMessage(messages.revokeAccess)"
						:disabled="disabled"
						type="quiet"
						:label="formatMessage(messages.revokeAccessFor, { username: row.username })"
						class="shrink-0 text-secondary hover:!filter-none hover:text-red focus-visible:!filter-none"
						@click="emit('remove', row)"
					>
						<XIcon aria-hidden="true" />
					</IconButton>
				</div>
				<div class="grid grid-cols-2 gap-3">
					<div class="flex min-w-0 flex-col gap-1">
						<span class="text-sm font-semibold text-secondary">
							{{ formatMessage(messages.joined) }}
						</span>
						<span
							v-if="row.pending"
							class="inline-flex h-7 w-fit items-center rounded-full border border-surface-5 border-solid bg-surface-4 px-2.5 py-1 text-sm font-semibold text-secondary"
							>{{ formatMessage(messages.pending) }}</span
						>
						<span v-else-if="row.joinedAt" v-tooltip="formatDateTime(row.joinedAt)">{{
							formatRelativeTime(row.joinedAt)
						}}</span>
					</div>
					<div class="flex min-w-0 flex-col gap-1">
						<span class="text-sm font-semibold text-secondary">
							{{ formatMessage(messages.lastPlayed) }}
						</span>
						<span v-if="row.lastPlayedAt" v-tooltip="formatDateTime(row.lastPlayedAt)">{{
							formatRelativeTime(row.lastPlayedAt)
						}}</span>
						<span v-else>{{ formatMessage(messages.never) }}</span>
					</div>
				</div>
			</div>
		</div>

		<Table
			v-model:sort-column="sortColumn"
			v-model:sort-direction="sortDirection"
			:columns="columns"
			:data="sortedRows"
			row-key="id"
			table-min-width="42rem"
			class="hidden sm:block"
		>
			<template #empty-state>
				<div class="flex h-64 items-center justify-center px-4 text-center text-secondary">
					<SpinnerIcon v-if="loading" class="size-6 animate-spin" aria-hidden="true" />
					{{ emptyStateMessage }}
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
							:alt="formatMessage(messages.avatarAlt, { username: row.username })"
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
			<template #cell-joined="{ row }">
				<span
					v-if="row.pending"
					class="inline-flex h-7 items-center rounded-full border border-surface-5 border-solid bg-surface-4 px-2.5 py-1 text-sm font-semibold text-secondary"
					>{{ formatMessage(messages.pending) }}</span
				>
				<span v-else-if="row.joinedAt" v-tooltip="formatDateTime(row.joinedAt)">{{
					formatRelativeTime(row.joinedAt)
				}}</span>
			</template>
			<template #cell-lastPlayed="{ row }">
				<span v-if="row.lastPlayedAt" v-tooltip="formatDateTime(row.lastPlayedAt)">{{
					formatRelativeTime(row.lastPlayedAt)
				}}</span>
				<span v-else>{{ formatMessage(messages.never) }}</span>
			</template>
			<template #cell-actions="{ row }">
				<div v-if="canManage" class="flex items-center justify-end">
					<IconButton
						v-tooltip="formatMessage(messages.revokeAccess)"
						:disabled="disabled"
						type="quiet"
						:label="formatMessage(messages.revokeAccessFor, { username: row.username })"
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
import {
	FilterIcon,
	SearchIcon,
	SpinnerIcon,
	UploadIcon,
	UserPlusIcon,
	XIcon,
} from '@modrinth/assets'
import { computed, ref, toRef } from 'vue'

import AutoLink from '#ui/components/base/AutoLink.vue'
import Avatar from '#ui/components/base/Avatar.vue'
import { Button, IconButton } from '#ui/components/base/buttons'
import Input from '#ui/components/base/inputs/Input.vue'
import Table, { type TableColumn } from '#ui/components/base/Table.vue'
import { useFormatDateTime, useRelativeTime } from '#ui/composables'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { truncatedTooltip } from '#ui/utils/truncate'

import { invitedPlayerMethodMessages, type InvitedPlayerRow } from './types'
import { useInvitedPlayersTable } from './use-invited-players-table'

type InvitedPlayerColumn = 'username' | 'lastPlayed' | 'joined' | 'actions'

const props = withDefaults(
	defineProps<{
		rows: InvitedPlayerRow[]
		loading?: boolean
		canManage?: boolean
		disabled?: boolean
		showPushUpdate?: boolean
		pushUpdateDisabled?: boolean
		pushUpdatePending?: boolean
		inviteLabel?: string
		inviteDisabled?: boolean
		invitePending?: boolean
	}>(),
	{
		showPushUpdate: true,
	},
)
const emit = defineEmits<{
	remove: [row: InvitedPlayerRow]
	'push-update': [event: MouseEvent]
	'push-update-hover': []
	invite: [event: MouseEvent]
}>()
const usernameRefs = ref<Record<string, HTMLElement | null>>({})
const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime({ style: 'narrow' })
const formatDateTime = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })
const messages = defineMessages({
	pushUpdate: {
		id: 'app.instance.admonitions.shared-instance.publish-button',
		defaultMessage: 'Push update',
	},
	invitePlayers: {
		id: 'servers.play.card.app.invite-button',
		defaultMessage: 'Invite players',
	},
	searchUsers: {
		id: 'servers.play.players.search-users',
		defaultMessage: 'Search {count} users...',
	},
	all: { id: 'servers.play.players.all', defaultMessage: 'All' },
	username: { id: 'servers.play.players.username', defaultMessage: 'Username' },
	joined: { id: 'servers.play.players.joined', defaultMessage: 'Joined' },
	lastPlayed: { id: 'servers.play.players.last-played', defaultMessage: 'Last played' },
	actions: { id: 'servers.play.players.actions', defaultMessage: 'Actions' },
	pending: { id: 'servers.play.players.pending', defaultMessage: 'Pending' },
	never: { id: 'servers.play.players.never', defaultMessage: 'Never' },
	avatarAlt: { id: 'servers.play.players.avatar-alt', defaultMessage: "{username}'s avatar" },
	revokeAccess: { id: 'servers.play.players.revoke-access', defaultMessage: 'Revoke access' },
	revokeAccessFor: {
		id: 'servers.play.players.revoke-access-for',
		defaultMessage: 'Revoke access for {username}',
	},
	loadingUsers: { id: 'servers.play.players.loading', defaultMessage: 'Loading users...' },
	noUsersJoined: {
		id: 'app.instance.share.members.empty',
		defaultMessage: 'No users have joined yet',
	},
	noUsersMatchFilters: {
		id: 'app.instance.share.members.no-filter-results',
		defaultMessage: 'No users match your filters.',
	},
})
const labels = computed(() => ({
	methods: {
		direct: formatMessage(invitedPlayerMethodMessages.direct),
		link: formatMessage(invitedPlayerMethodMessages.link),
	},
	never: formatMessage(messages.never),
	pending: formatMessage(messages.pending),
}))
const {
	search,
	methodFilter,
	sortColumn,
	sortDirection,
	methodFilterOptions,
	hasMultipleMethods,
	sortedRows,
	toggleMethodFilter,
} = useInvitedPlayersTable(toRef(props, 'rows'), formatRelativeTime, labels)

const emptyStateMessage = computed(() =>
	formatMessage(
		props.loading
			? messages.loadingUsers
			: props.rows.length === 0
				? messages.noUsersJoined
				: messages.noUsersMatchFilters,
	),
)

const columns = computed<TableColumn<InvitedPlayerColumn>[]>(() => {
	const result: TableColumn<InvitedPlayerColumn>[] = [
		{
			key: 'username',
			label: formatMessage(messages.username),
			width: 'clamp(14rem, 30%, 26rem)',
			enableSorting: true,
			headerClass: '!pr-3',
			cellClass: '!pr-3',
		},
		{
			key: 'joined',
			label: formatMessage(messages.joined),
			width: 'clamp(7rem, 14%, 12rem)',
			enableSorting: true,
			defaultSortDirection: 'desc',
			headerClass: 'whitespace-nowrap !px-2',
			cellClass: 'whitespace-nowrap !px-2',
		},
		{
			key: 'lastPlayed',
			label: formatMessage(messages.lastPlayed),
			width: 'clamp(7rem, 15%, 13rem)',
			enableSorting: true,
			headerClass: 'whitespace-nowrap !px-2',
			cellClass: 'whitespace-nowrap !px-2',
		},
	]
	if (props.canManage)
		result.push({
			key: 'actions',
			label: formatMessage(messages.actions),
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

function userProfileLink(username: string) {
	return !username || username.includes('@') ? undefined : `/user/${encodeURIComponent(username)}`
}
function setUsernameRef(id: string, element: unknown) {
	usernameRefs.value[id] = element instanceof HTMLElement ? element : null
}
</script>
