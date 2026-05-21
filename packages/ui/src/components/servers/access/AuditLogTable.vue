<template>
	<div class="@container flex flex-col gap-4">
		<div class="flex min-w-0 flex-col items-start gap-3 @[640px]:flex-row @[640px]:items-center">
			<DatePicker
				v-model="dateRange"
				mode="range"
				:placeholder="formatMessage(messages.dateRangePlaceholder)"
				:wrapper-class="dateRangePickerWrapperClass"
				:input-class="dateRangePickerInputClass"
				date-format="Y-m-d"
				alt-format="M j, Y"
				position="below left"
			/>
			<template v-if="slots.filters">
				<div class="hidden h-8 w-[1px] shrink-0 bg-surface-5 @[640px]:ml-4 @[640px]:block"></div>
				<div class="flex min-w-0 flex-wrap items-center gap-2">
					<slot name="filters"></slot>
				</div>
			</template>
		</div>

		<div class="relative">
			<Table
				v-if="filteredEntries.length > 0"
				class="hidden @[640px]:block"
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
							:alt="formatMessage(messages.userAvatarAlt, { username: actorName(entry) })"
							:tint-by="entry.actor.username"
							size="22px"
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

			<div v-if="filteredEntries.length > 0" class="flex flex-col gap-3 @[640px]:hidden">
				<div
					v-for="entry in filteredEntries"
					:key="entry.id"
					class="flex min-w-0 flex-col gap-3 rounded-2xl border border-solid border-surface-5 bg-surface-2 p-4"
				>
					<AutoLink
						v-tooltip="actorName(entry)"
						:to="actorProfilePath(entry)"
						class="inline-flex min-w-0 items-center gap-2 self-start"
						:class="actorProfilePath(entry) ? 'text-primary hover:underline' : 'text-primary'"
					>
						<Avatar
							:src="actorAvatarSrc(entry)"
							:alt="formatMessage(messages.userAvatarAlt, { username: actorName(entry) })"
							:tint-by="entry.actor.username"
							size="24px"
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
					<div class="min-w-0">
						<component :is="entry.event.component" v-bind="entry.event.props" />
					</div>
					<div class="flex min-w-0 items-center gap-1 text-sm text-secondary">
						<span v-tooltip="entry.world?.name" class="min-w-0 truncate">
							{{ entry.world?.name ?? formatMessage(messages.serverScope) }}
						</span>
						<BulletDivider class="shrink-0" />
						<span v-tooltip="formatDate(entry.timestamp)" class="shrink-0">
							{{ formatRelativeTime(entry.timestamp) }}
						</span>
					</div>
				</div>
			</div>

			<div v-else class="overflow-hidden rounded-2xl border border-solid border-surface-5">
				<div
					class="hidden min-h-14 bg-surface-3 @[640px]:grid @[640px]:h-14 @[640px]:grid-cols-[22%_48%_18%_12%]"
				>
					<div class="hidden items-center pl-4 font-semibold text-secondary @[640px]:flex">
						{{ formatMessage(messages.userColumn) }}
					</div>
					<div class="hidden items-center font-semibold text-secondary @[640px]:flex">
						{{ formatMessage(messages.eventColumn) }}
					</div>
					<div class="hidden items-center font-semibold text-secondary @[640px]:flex">
						{{ formatMessage(messages.worldColumn) }}
					</div>
					<div
						class="hidden items-center justify-end pr-4 font-semibold text-secondary @[640px]:flex"
					>
						{{ formatMessage(messages.timeColumn) }}
					</div>
				</div>
				<div
					class="border-0 border-solid border-surface-5 bg-surface-2 px-4 py-8 text-center text-secondary @[640px]:border-t"
				>
					{{ formatMessage(emptyStateMessage) }}
				</div>
			</div>

			<Transition name="audit-log-loading-fade">
				<div
					v-if="loading"
					class="pointer-events-none absolute inset-0 z-20 animate-audit-log-bpulse rounded-2xl bg-surface-3"
					aria-hidden="true"
				/>
			</Transition>
		</div>

		<div
			v-if="loadingMore"
			class="h-8 animate-audit-log-bpulse rounded-xl bg-surface-3"
			aria-hidden="true"
		></div>
		<div v-if="hasMore" ref="loadMoreSentinel" class="h-px"></div>
	</div>
</template>

<script setup lang="ts">
import { IntercomBubbleIcon } from '@modrinth/assets'
import { computed, nextTick, onBeforeUnmount, onMounted, ref, useSlots, watch } from 'vue'

import { useFormatDateTime, useRelativeTime } from '../../../composables'
import { defineMessages, useVIntl } from '../../../composables/i18n'
import AutoLink from '../../base/AutoLink.vue'
import Avatar from '../../base/Avatar.vue'
import BulletDivider from '../../base/BulletDivider.vue'
import DatePicker from '../../base/DatePicker.vue'
import Table, { type TableColumn } from '../../base/Table.vue'
import type { ServerAuditLogEntry, ServerAuditLogFilters } from './types'

const props = defineProps<{
	entries: ServerAuditLogEntry[]
	hasActiveExternalFilters?: boolean
	hasMore?: boolean
	loading?: boolean
	loadingMore?: boolean
}>()

const emit = defineEmits<{
	'load-more': []
}>()

const query = defineModel<string>('query', { default: '' })
const dateRange = defineModel<string[]>('dateRange', { default: () => [] })
const filters = defineModel<ServerAuditLogFilters>('filters', {
	default: () => ({
		userId: null,
		worldId: null,
	}),
})

const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()
const formatDate = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })
const slots = useSlots()
const loadMoreSentinel = ref<HTMLElement | null>(null)
let loadMoreObserver: IntersectionObserver | null = null

const messages = defineMessages({
	dateRangePlaceholder: {
		id: 'servers.audit-log.date-range.placeholder',
		defaultMessage: 'Select date range...',
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
	serverScope: {
		id: 'servers.audit-log.scope.server',
		defaultMessage: 'Server',
	},
})

const dateRangePickerWrapperClass = computed(() =>
	slots.filters ? '!w-full @[640px]:!w-[285px] shrink-0' : '!w-full @[640px]:!w-[285px]',
)
const dateRangePickerInputClass = computed(
	() =>
		'!h-10 !w-full !rounded-[14px] !bg-surface-4 !py-2.5 !pl-10 !pr-4 !text-base shadow-[0px_1px_1px_rgba(0,0,0,0.3),0px_1px_1.5px_rgba(0,0,0,0.15)]',
)

onMounted(() => {
	updateLoadMoreObserver()
})

onBeforeUnmount(() => {
	loadMoreObserver?.disconnect()
})

watch(
	() => [props.hasMore, props.loadingMore, loadMoreSentinel.value] as const,
	() => updateLoadMoreObserver(),
	{ flush: 'post' },
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
		props.hasActiveExternalFilters ||
		query.value.trim().length > 0 ||
		dateRange.value.length > 0 ||
		!!filters.value.userId ||
		!!filters.value.worldId,
)

const emptyStateMessage = computed(() =>
	props.entries.length === 0 && !hasActiveFilters.value
		? messages.noActivityEmptyState
		: messages.emptyState,
)

function actorName(entry: ServerAuditLogEntry): string {
	if (entry.actor.id !== 'support') return entry.actor.username
	return entry.actor.username === 'support'
		? formatMessage(messages.supportActor)
		: entry.actor.username
}

function actorAvatarSrc(entry: ServerAuditLogEntry): string | undefined {
	return entry.actor.id === 'support' ? IntercomBubbleIcon : (entry.actor.avatarUrl ?? undefined)
}

function actorProfilePath(entry: ServerAuditLogEntry): string | undefined {
	return entry.actor.profilePath
}

function updateLoadMoreObserver() {
	loadMoreObserver?.disconnect()
	loadMoreObserver = null

	if (!props.hasMore || typeof IntersectionObserver === 'undefined') {
		return
	}

	nextTick(() => {
		if (!loadMoreSentinel.value || !props.hasMore) {
			return
		}

		loadMoreObserver?.disconnect()
		loadMoreObserver = new IntersectionObserver(
			(entries) => {
				if (entries.some((entry) => entry.isIntersecting) && !props.loadingMore) {
					emit('load-more')
				}
			},
			{ rootMargin: '400px 0px' },
		)
		loadMoreObserver.observe(loadMoreSentinel.value)
	})
}
</script>

<style>
@keyframes audit-log-bpulse {
	50% {
		filter: brightness(75%);
	}
}

.animate-audit-log-bpulse {
	animation: audit-log-bpulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

.audit-log-loading-fade-enter-active,
.audit-log-loading-fade-leave-active {
	transition: opacity 250ms ease-in-out;
}

.audit-log-loading-fade-enter-from,
.audit-log-loading-fade-leave-to {
	opacity: 0;
}
</style>
