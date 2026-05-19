<template>
	<div class="flex flex-col gap-4">
		<div class="flex items-center">
			<Combobox
				v-model="selectedTimeRange"
				class="!w-full sm:!w-[245px]"
				:options="timeRangeOptions"
				:display-value="selectedTimeRangeLabel"
				trigger-class="!h-10 !w-full sm:!w-[245px] !rounded-[14px] !bg-surface-4 !px-4 !py-2.5 !text-base shadow-[0px_1px_1px_rgba(0,0,0,0.3),0px_1px_1.5px_rgba(0,0,0,0.15)]"
				dropdown-min-width="245px"
			/>
		</div>

		<Table
			v-if="filteredEntries.length > 0"
			class="hidden sm:block"
			:columns="columns"
			:data="tableEntries"
			row-key="id"
		>
			<template #cell-user="{ row: entry }">
				<AutoLink
					:to="actorProfilePath(entry)"
					class="flex min-w-0 items-center gap-2"
					:class="actorProfilePath(entry) ? 'text-primary hover:underline' : ''"
				>
					<Avatar
						:src="
							entry.actor.id === 'support'
								? IntercomBubbleIcon
								: (entry.actor.avatarUrl ?? undefined)
						"
						:alt="formatMessage(messages.userAvatarAlt, { username: entry.actor.username })"
						:tint-by="entry.actor.username"
						size="22px"
						circle
						no-shadow
					/>
					<span
						class="min-w-0 truncate font-medium"
						:class="entry.actor.id === 'support' ? 'text-blue' : ''"
					>
						{{
							entry.actor.id === 'support'
								? formatMessage(messages.supportActor)
								: entry.actor.username
						}}
					</span>
				</AutoLink>
			</template>

			<template #cell-event="{ row: entry }">
				<component :is="entry.event.component" v-bind="entry.event.props" />
			</template>

			<template #cell-world="{ row: entry }">
				<span
					v-tooltip="entry.world?.name"
					class="truncate"
					:class="entry.world ? 'text-primary' : 'text-secondary'"
				>
					{{ entry.world?.name ?? '—' }}
				</span>
			</template>

			<template #cell-time="{ row: entry }">
				<span v-tooltip="formatDate(entry.timestamp)">
					{{ formatRelativeTime(entry.timestamp) }}
				</span>
			</template>
		</Table>

		<div
			v-if="filteredEntries.length > 0"
			class="overflow-hidden rounded-2xl border border-solid border-surface-5 sm:hidden"
		>
			<div class="grid min-h-14 grid-cols-[minmax(0,1fr)_5rem] bg-surface-3">
				<div class="flex items-center pl-4 font-semibold text-secondary">
					{{ formatMessage(messages.eventColumn) }}
				</div>
				<div class="flex items-center justify-end pr-4 font-semibold text-secondary">
					{{ formatMessage(messages.timeColumn) }}
				</div>
			</div>
			<div
				v-for="(entry, index) in filteredEntries"
				:key="entry.id"
				class="grid min-h-[5.5rem] grid-cols-[minmax(0,1fr)_5rem] items-start border-0 border-t border-solid border-surface-5"
				:class="index % 2 === 0 ? 'bg-surface-2' : 'bg-surface-1.5'"
			>
				<div class="min-w-0 py-3 pl-4 pr-2">
					<div class="mb-2 flex min-w-0 items-center gap-2 text-sm">
						<AutoLink
							v-tooltip="actorName(entry)"
							:to="actorProfilePath(entry)"
							class="inline-flex min-w-0 items-center gap-2"
							:class="actorProfilePath(entry) ? 'text-primary hover:underline' : ''"
						>
							<Avatar
								:src="actorAvatarSrc(entry)"
								:alt="formatMessage(messages.userAvatarAlt, { username: actorName(entry) })"
								:tint-by="entry.actor.username"
								size="20px"
								circle
								no-shadow
							/>
							<span
								class="min-w-0 truncate font-medium"
								:class="entry.actor.id === 'support' ? 'text-blue' : ''"
							>
								{{ actorName(entry) }}
							</span>
						</AutoLink>
						<span class="text-secondary">·</span>
						<span
							v-tooltip="entry.world?.name"
							class="min-w-0 truncate text-secondary"
						>
							{{ entry.world?.name ?? 'Server' }}
						</span>
					</div>
					<component :is="entry.event.component" v-bind="entry.event.props" />
				</div>
				<div class="min-w-0 py-3 pr-4 text-right text-secondary">
					<span v-tooltip="formatDate(entry.timestamp)" class="inline-block max-w-full truncate">
						{{ formatRelativeTime(entry.timestamp) }}
					</span>
				</div>
			</div>
		</div>

		<div v-else class="overflow-hidden rounded-2xl border border-solid border-surface-5">
			<div
				class="grid min-h-14 grid-cols-[minmax(0,1fr)_5rem] bg-surface-3 sm:h-14 sm:grid-cols-[22%_48%_18%_12%]"
			>
				<div class="flex items-center pl-4 font-semibold text-secondary sm:hidden">
					{{ formatMessage(messages.eventColumn) }}
				</div>
				<div class="hidden items-center pl-4 font-semibold text-secondary sm:flex">
					{{ formatMessage(messages.userColumn) }}
				</div>
				<div class="hidden items-center font-semibold text-secondary sm:flex">
					{{ formatMessage(messages.eventColumn) }}
				</div>
				<div class="hidden items-center font-semibold text-secondary sm:flex">
					{{ formatMessage(messages.worldColumn) }}
				</div>
				<div class="flex items-center justify-end pr-4 font-semibold text-secondary">
					{{ formatMessage(messages.timeColumn) }}
				</div>
			</div>
			<div
				class="border-0 border-t border-solid border-surface-5 bg-surface-2 px-4 py-8 text-center text-secondary"
			>
				{{ formatMessage(emptyStateMessage) }}
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { IntercomBubbleIcon } from '@modrinth/assets'
import { computed, ref } from 'vue'

import { useFormatDateTime, useRelativeTime } from '../../../composables'
import { defineMessages, useVIntl } from '../../../composables/i18n'
import AutoLink from '../../base/AutoLink.vue'
import Avatar from '../../base/Avatar.vue'
import Combobox, { type ComboboxOption } from '../../base/Combobox.vue'
import Table, { type TableColumn } from '../../base/Table.vue'
import type { ServerAuditLogEntry, ServerAuditLogFilters } from './types'

const props = defineProps<{
	entries: ServerAuditLogEntry[]
}>()

const query = defineModel<string>('query', { default: '' })
const filters = defineModel<ServerAuditLogFilters>('filters', {
	default: () => ({
		userId: null,
		worldId: null,
	}),
})

const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()
const formatDate = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })

type AuditTimeRange =
	| 'previous_30_minutes'
	| 'previous_hour'
	| 'previous_12_hours'
	| 'previous_24_hours'
	| 'today'
	| 'yesterday'
	| 'this_week'
	| 'last_week'
	| 'previous_7_days'
	| 'this_month'
	| 'last_month'
	| 'last_30_days'
	| 'this_quarter'
	| 'last_quarter'
	| 'this_year'
	| 'last_year'
	| 'previous_year'
	| 'previous_two_years'
	| 'all_time'

const selectedTimeRange = ref<AuditTimeRange>('last_30_days')

const messages = defineMessages({
	previous30Minutes: {
		id: 'servers.audit-log.time-range.previous-30-minutes',
		defaultMessage: 'Previous 30 minutes',
	},
	previousHour: {
		id: 'servers.audit-log.time-range.previous-hour',
		defaultMessage: 'Previous hour',
	},
	previous12Hours: {
		id: 'servers.audit-log.time-range.previous-12-hours',
		defaultMessage: 'Previous 12 hours',
	},
	previous24Hours: {
		id: 'servers.audit-log.time-range.previous-24-hours',
		defaultMessage: 'Previous 24 hours',
	},
	today: {
		id: 'servers.audit-log.time-range.today',
		defaultMessage: 'Today',
	},
	yesterday: {
		id: 'servers.audit-log.time-range.yesterday',
		defaultMessage: 'Yesterday',
	},
	thisWeek: {
		id: 'servers.audit-log.time-range.this-week',
		defaultMessage: 'This week',
	},
	lastWeek: {
		id: 'servers.audit-log.time-range.last-week',
		defaultMessage: 'Last week',
	},
	previous7Days: {
		id: 'servers.audit-log.time-range.previous-7-days',
		defaultMessage: 'Previous 7 days',
	},
	thisMonth: {
		id: 'servers.audit-log.time-range.this-month',
		defaultMessage: 'This month',
	},
	lastMonth: {
		id: 'servers.audit-log.time-range.last-month',
		defaultMessage: 'Last month',
	},
	last30Days: {
		id: 'servers.audit-log.time-range.last-30-days',
		defaultMessage: 'Last 30 days',
	},
	thisQuarter: {
		id: 'servers.audit-log.time-range.this-quarter',
		defaultMessage: 'This quarter',
	},
	lastQuarter: {
		id: 'servers.audit-log.time-range.last-quarter',
		defaultMessage: 'Last quarter',
	},
	thisYear: {
		id: 'servers.audit-log.time-range.this-year',
		defaultMessage: 'This year',
	},
	lastYear: {
		id: 'servers.audit-log.time-range.last-year',
		defaultMessage: 'Last year',
	},
	previousYear: {
		id: 'servers.audit-log.time-range.previous-year',
		defaultMessage: 'Previous year',
	},
	previousTwoYears: {
		id: 'servers.audit-log.time-range.previous-two-years',
		defaultMessage: 'Previous two years',
	},
	allTime: {
		id: 'servers.audit-log.time-range.all-time',
		defaultMessage: 'All Time',
	},
	supportActor: {
		id: 'servers.audit-log.actor.support',
		defaultMessage: 'Support',
	},
	userColumn: {
		id: 'servers.audit-log.column.user',
		defaultMessage: 'User',
	},
	worldColumn: {
		id: 'servers.audit-log.column.world',
		defaultMessage: 'Instance',
	},
	eventColumn: {
		id: 'servers.audit-log.column.event',
		defaultMessage: 'Event',
	},
	timeColumn: {
		id: 'servers.audit-log.column.time',
		defaultMessage: 'Time',
	},
	emptyState: {
		id: 'servers.audit-log.empty',
		defaultMessage: 'No activity matches your filters.',
	},
	noActivityEmptyState: {
		id: 'servers.audit-log.empty.no-activity',
		defaultMessage: 'Perform an action on your server and you will see it here!',
	},
	userAvatarAlt: {
		id: 'servers.audit-log.user-avatar-alt',
		defaultMessage: "{username}'s avatar",
	},
})

const timeRangeOptions = computed<ComboboxOption<AuditTimeRange>[]>(() => [
	{ value: 'previous_30_minutes', label: formatMessage(messages.previous30Minutes) },
	{ value: 'previous_hour', label: formatMessage(messages.previousHour) },
	{ value: 'previous_12_hours', label: formatMessage(messages.previous12Hours) },
	{ value: 'previous_24_hours', label: formatMessage(messages.previous24Hours) },
	{ value: 'today', label: formatMessage(messages.today) },
	{ value: 'yesterday', label: formatMessage(messages.yesterday) },
	{ value: 'this_week', label: formatMessage(messages.thisWeek) },
	{ value: 'last_week', label: formatMessage(messages.lastWeek) },
	{ value: 'previous_7_days', label: formatMessage(messages.previous7Days) },
	{ value: 'this_month', label: formatMessage(messages.thisMonth) },
	{ value: 'last_month', label: formatMessage(messages.lastMonth) },
	{ value: 'last_30_days', label: formatMessage(messages.last30Days) },
	{ value: 'this_quarter', label: formatMessage(messages.thisQuarter) },
	{ value: 'last_quarter', label: formatMessage(messages.lastQuarter) },
	{ value: 'this_year', label: formatMessage(messages.thisYear) },
	{ value: 'last_year', label: formatMessage(messages.lastYear) },
	{ value: 'previous_year', label: formatMessage(messages.previousYear) },
	{ value: 'previous_two_years', label: formatMessage(messages.previousTwoYears) },
	{ value: 'all_time', label: formatMessage(messages.allTime) },
])

const selectedTimeRangeLabel = computed(
	() =>
		timeRangeOptions.value.find((option) => option.value === selectedTimeRange.value)?.label ??
		formatMessage(messages.last30Days),
)

type AuditLogTableColumn = 'user' | 'event' | 'world' | 'time'
type AuditLogTableRow = ServerAuditLogEntry & Record<string, unknown>

const columns = computed<TableColumn<AuditLogTableColumn>[]>(() => [
	{ key: 'user', label: formatMessage(messages.userColumn), width: '22%' },
	{ key: 'event', label: formatMessage(messages.eventColumn), width: '48%' },
	{ key: 'world', label: formatMessage(messages.worldColumn), width: '18%' },
	{ key: 'time', label: formatMessage(messages.timeColumn), align: 'right', width: '12%' },
])

const filteredEntries = computed(() => {
	const normalizedQuery = query.value.trim().toLowerCase()

	return props.entries
		.filter((entry) => {
			if (filters.value.userId && entry.actor.id !== filters.value.userId) return false
			if (filters.value.worldId && entry.world?.id !== filters.value.worldId) return false

			if (!normalizedQuery) return true

			return [entry.actor.username, entry.world?.name, entry.event.searchText, entry.event.key]
				.filter((value): value is string => typeof value === 'string' && value.length > 0)
				.some((value) => value.toLowerCase().includes(normalizedQuery))
		})
		.slice()
		.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())
})

const tableEntries = computed<AuditLogTableRow[]>(() => filteredEntries.value as AuditLogTableRow[])

const hasActiveFilters = computed(
	() =>
		query.value.trim().length > 0 ||
		!!filters.value.userId ||
		!!filters.value.worldId,
)

const emptyStateMessage = computed(() =>
	props.entries.length === 0 && !hasActiveFilters.value
		? messages.noActivityEmptyState
		: messages.emptyState,
)

function actorName(entry: ServerAuditLogEntry): string {
	return entry.actor.id === 'support' ? formatMessage(messages.supportActor) : entry.actor.username
}

function actorAvatarSrc(entry: ServerAuditLogEntry): string | undefined {
	return entry.actor.id === 'support' ? IntercomBubbleIcon : (entry.actor.avatarUrl ?? undefined)
}

function actorProfilePath(entry: ServerAuditLogEntry): string | undefined {
	return entry.actor.profilePath
}
</script>
