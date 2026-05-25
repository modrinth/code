<template>
	<div class="@container flex flex-col gap-4">
		<div class="flex min-w-0 flex-col items-start gap-3 @[640px]:flex-row @[640px]:items-center">
			<TimeFramePicker
				v-model:mode="timeframeMode"
				v-model:preset="timeframePreset"
				v-model:last-amount="timeframeLastAmount"
				v-model:last-unit="timeframeLastUnit"
				v-model:custom-start-date="timeframeCustomStartDate"
				v-model:custom-end-date="timeframeCustomEndDate"
				:class="timeframePickerClass"
				:trigger-class="timeframePickerTriggerClass"
			/>
			<template v-if="slots.filters">
				<div class="hidden h-8 w-[1px] shrink-0 bg-surface-5 @[640px]:ml-1 @[640px]:block"></div>
				<div class="flex min-w-0 flex-wrap items-center gap-2">
					<slot name="filters"></slot>
				</div>
			</template>
		</div>

		<div class="audit-log-content-frame relative overflow-hidden" :style="contentFrameStyle">
			<div ref="contentBody" class="min-w-0">
				<Table
					v-if="filteredEntries.length > 0"
					v-model:sort-column="sortColumn"
					v-model:sort-direction="sortDirection"
					class="audit-log-table hidden @[800px]:block @[800px]:text-sm @[1040px]:text-base"
					:columns="columns"
					:data="tableEntries"
					row-key="id"
					:row-transition-name="rowTransitionName"
				>
					<template #header-world="{ column }">
						<span class="inline-flex min-w-0 max-w-full items-center gap-1 font-semibold">
							<span class="min-w-0 truncate">{{ column.label }}</span>
							<Tooltip
								theme="dismissable-prompt"
								class="inline-flex shrink-0"
								:triggers="['hover', 'focus']"
								:popper-triggers="['hover', 'focus']"
								popper-class="v-popper--interactive"
								placement="top"
								:delay="{ show: 200, hide: 100 }"
								no-auto-focus
							>
								<button
									type="button"
									:aria-label="formatMessage(messages.instanceTooltipTitle)"
									class="inline-flex cursor-help items-center justify-center border-0 bg-transparent p-0 text-secondary transition-colors hover:text-contrast"
								>
									<UnknownIcon class="size-4" aria-hidden="true" />
								</button>
								<template #popper>
									<div class="grid !w-64 gap-1">
										<h3 class="m-0 whitespace-nowrap text-base w-full font-bold text-contrast">
											{{ formatMessage(messages.instanceTooltipTitle) }}
										</h3>
										<p
											class="m-0 text-wrap text-sm w-full font-medium leading-tight text-secondary"
										>
											{{ formatMessage(messages.instanceTooltipDescription) }}
										</p>
									</div>
								</template>
							</Tooltip>
						</span>
					</template>

					<template #cell-user="{ row: entry }">
						<AutoLink
							v-tooltip="actorName(entry)"
							:to="actorProfilePath(entry)"
							class="flex min-w-0 items-center gap-2 whitespace-nowrap"
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
						<AuditLogEventCell :event="entry.event" />
					</template>

					<template #cell-world="{ row: entry }">
						<span
							v-tooltip="entry.world?.name"
							class="block min-w-0 truncate whitespace-nowrap"
							:class="entry.world ? 'text-primary' : 'text-secondary'"
						>
							{{ entry.world?.name ?? '—' }}
						</span>
					</template>

					<template #cell-time="{ row: entry }">
						<span
							v-tooltip="formatDate(entry.timestamp)"
							class="inline-block max-w-full truncate whitespace-nowrap align-middle leading-5 @[1040px]:leading-6"
						>
							{{ formatCompactRelativeTime(entry.timestamp) }}
						</span>
					</template>
				</Table>

				<TransitionGroup
					v-if="filteredEntries.length > 0"
					name="audit-log-card"
					tag="div"
					class="flex flex-col gap-3 @[800px]:hidden"
				>
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
							<span
								v-if="showWorldColumn"
								v-tooltip="entry.world?.name"
								class="min-w-0 truncate"
							>
								{{ entry.world?.name ?? formatMessage(messages.serverScope) }}
							</span>
							<BulletDivider v-if="showWorldColumn" class="shrink-0" />
							<span v-tooltip="formatDate(entry.timestamp)" class="shrink-0">
								{{ formatRelativeTime(entry.timestamp) }}
							</span>
						</div>
					</div>
				</TransitionGroup>

				<div v-else class="overflow-hidden rounded-2xl border border-solid border-surface-5">
					<div
						class="hidden min-h-14 bg-surface-3 @[800px]:grid @[800px]:h-14 @[800px]:text-sm @[1040px]:text-base"
						:class="
							showWorldColumn
								? '@[800px]:grid-cols-[18%_52%_20%_10%]'
								: '@[800px]:grid-cols-[18%_72%_10%]'
						"
					>
						<div class="hidden items-center pl-4 pr-2 font-semibold text-secondary @[800px]:flex">
							{{ formatMessage(messages.userColumn) }}
						</div>
						<div class="hidden items-center px-2 font-semibold text-secondary @[800px]:flex">
							{{ formatMessage(messages.eventColumn) }}
						</div>
						<div
							v-if="showWorldColumn"
							class="hidden items-center px-2 font-semibold text-secondary @[800px]:flex"
						>
							<span class="inline-flex min-w-0 max-w-full items-center gap-1 font-semibold">
								<span class="min-w-0 truncate">{{ formatMessage(messages.worldColumn) }}</span>
								<Tooltip
									theme="dismissable-prompt"
									class="inline-flex shrink-0"
									:triggers="['hover', 'focus']"
									:popper-triggers="['hover', 'focus']"
									popper-class="v-popper--interactive"
									placement="top"
									:delay="{ show: 200, hide: 100 }"
									no-auto-focus
								>
									<button
										type="button"
										:aria-label="formatMessage(messages.instanceTooltipTitle)"
										class="inline-flex cursor-help items-center justify-center border-0 bg-transparent p-0 text-secondary transition-colors hover:text-contrast"
									>
										<UnknownIcon class="size-4" aria-hidden="true" />
									</button>
									<template #popper>
										<div class="grid !w-64 gap-1">
											<h3 class="m-0 whitespace-nowrap text-base font-bold text-contrast">
												{{ formatMessage(messages.instanceTooltipTitle) }}
											</h3>
											<p class="m-0 text-wrap text-sm font-medium leading-tight text-secondary">
												{{ formatMessage(messages.instanceTooltipDescription) }}
											</p>
										</div>
									</template>
								</Tooltip>
							</span>
						</div>
						<div
							class="hidden items-center justify-end pl-2 pr-4 font-semibold text-secondary @[800px]:flex"
						>
							{{ formatMessage(messages.timeColumn) }}
						</div>
					</div>
					<div
						class="border-0 border-solid border-surface-5 bg-surface-2 px-4 py-8 text-center text-secondary @[800px]:border-t"
					>
						{{ formatMessage(emptyStateMessage) }}
					</div>
				</div>
			</div>

			<Transition name="audit-log-loading-fade">
				<div
					v-if="loading"
					class="pointer-events-none absolute bottom-px left-px right-px top-0 z-20 animate-audit-log-bpulse rounded-[15px] bg-surface-3 @[800px]:top-[57px] @[800px]:rounded-t-none @[800px]:border-0 @[800px]:border-t @[800px]:border-solid @[800px]:border-surface-5"
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
import { IntercomBubbleIcon, UnknownIcon } from '@modrinth/assets'
import { Tooltip } from 'floating-vue'
import { computed, nextTick, onBeforeUnmount, onMounted, ref, useSlots, watch } from 'vue'

import { useFormatDateTime, useRelativeTime } from '../../../composables'
import { defineMessages, useVIntl } from '../../../composables/i18n'
import AutoLink from '../../base/AutoLink.vue'
import Avatar from '../../base/Avatar.vue'
import BulletDivider from '../../base/BulletDivider.vue'
import Table, { type SortDirection, type TableColumn } from '../../base/Table.vue'
import TimeFramePicker, {
	type TimeFrameLastUnit,
	type TimeFrameMode,
	type TimeFramePreset,
} from '../../base/TimeFramePicker.vue'
import AuditLogEventCell from './AuditLogEventCell.vue'
import type { ServerAuditLogEntry, ServerAuditLogFilters } from './types'

const props = defineProps<{
	entries: ServerAuditLogEntry[]
	hasActiveExternalFilters?: boolean
	hasMore?: boolean
	loading?: boolean
	loadingMore?: boolean
	showWorldColumn?: boolean
	suppressRowTransitions?: boolean
}>()

const emit = defineEmits<{
	'load-more': []
}>()

const query = defineModel<string>('query', { default: '' })
const timeframeMode = defineModel<TimeFrameMode>('timeframeMode', { default: 'preset' })
const timeframePreset = defineModel<TimeFramePreset>('timeframePreset', { default: 'all_time' })
const timeframeLastAmount = defineModel<number>('timeframeLastAmount', { default: 30 })
const timeframeLastUnit = defineModel<TimeFrameLastUnit>('timeframeLastUnit', { default: 'days' })
const timeframeCustomStartDate = defineModel<string>('timeframeCustomStartDate', { default: '' })
const timeframeCustomEndDate = defineModel<string>('timeframeCustomEndDate', { default: '' })
const sortDirection = defineModel<SortDirection>('sortDirection', { default: 'desc' })
const filters = defineModel<ServerAuditLogFilters>('filters', {
	default: () => ({
		userId: null,
		worldId: null,
	}),
})

const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()
const formatCompactRelativeTime = useRelativeTime({ numeric: 'always', style: 'narrow' })
const formatDate = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })
const slots = useSlots()
const sortColumn = ref<string | undefined>('time')
const suppressSortRowTransitions = ref(false)
const loadMoreSentinel = ref<HTMLElement | null>(null)
const contentBody = ref<HTMLElement | null>(null)
const contentHeight = ref<number | null>(null)
const showWorldColumn = computed(() => props.showWorldColumn !== false)
let loadMoreObserver: IntersectionObserver | null = null
let contentResizeObserver: ResizeObserver | null = null
let sortTransitionResetTimeout: ReturnType<typeof setTimeout> | null = null

const messages = defineMessages({
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
	instanceTooltipTitle: {
		id: 'servers.audit-log.column.world.tooltip-title',
		defaultMessage: 'Coming soon!',
	},
	instanceTooltipDescription: {
		id: 'servers.audit-log.column.world.tooltip-description',
		defaultMessage:
			'Server instances are contained environments with their own installed content and world files.',
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

const timeframePickerClass = computed(() =>
	slots.filters ? '!w-full @[640px]:!w-[225px] shrink-0' : '!w-full @[640px]:!w-[225px]',
)
const timeframePickerTriggerClass =
	'!h-10 !min-h-10 !w-full !rounded-[14px] !bg-surface-4 !py-2.5 !pl-4 !pr-3 !text-base shadow-[0px_1px_1px_rgba(0,0,0,0.3),0px_1px_1.5px_rgba(0,0,0,0.15)]'

onMounted(() => {
	updateLoadMoreObserver()
	updateContentHeightObserver()
})

onBeforeUnmount(() => {
	loadMoreObserver?.disconnect()
	contentResizeObserver?.disconnect()
	if (sortTransitionResetTimeout) {
		clearTimeout(sortTransitionResetTimeout)
	}
})

watch(
	() => [props.hasMore, props.loadingMore, loadMoreSentinel.value] as const,
	() => updateLoadMoreObserver(),
	{ flush: 'post' },
)

watch(
	sortDirection,
	(_direction, previousDirection) => {
		if (previousDirection === undefined) return
		suppressSortRowTransitions.value = true
		scheduleSortTransitionReset(1500)
	},
	{ flush: 'sync' },
)

watch(
	() => props.entries,
	() => {
		if (suppressSortRowTransitions.value) {
			scheduleSortTransitionReset(120)
		}
	},
	{ flush: 'post' },
)

type AuditLogTableColumn = 'user' | 'event' | 'world' | 'time'
type AuditLogTableRow = ServerAuditLogEntry & Record<string, unknown>

const columns = computed<TableColumn<AuditLogTableColumn>[]>(() => {
	const tableColumns: TableColumn<AuditLogTableColumn>[] = [
		{ key: 'user', label: formatMessage(messages.userColumn), width: '18%' },
		{
			key: 'event',
			label: formatMessage(messages.eventColumn),
			width: showWorldColumn.value ? '52%' : '72%',
		},
	]

	if (showWorldColumn.value) {
		tableColumns.push({
			key: 'world',
			label: formatMessage(messages.worldColumn),
			width: '20%',
		})
	}

	tableColumns.push({
		key: 'time',
		label: formatMessage(messages.timeColumn),
		align: 'right',
		enableSorting: true,
		width: '10%',
	})

	return tableColumns
})
const rowTransitionName = computed(() =>
	props.suppressRowTransitions || suppressSortRowTransitions.value ? undefined : 'audit-log-row',
)

const filteredEntries = computed(() => {
	const normalizedQuery = query.value.trim().toLowerCase()

	return props.entries
		.filter((entry) => {
			if (filters.value.userId && entry.actor.id !== filters.value.userId) return false
			if (
				showWorldColumn.value &&
				filters.value.worldId &&
				entry.world?.id !== filters.value.worldId
			) {
				return false
			}

			if (!normalizedQuery) return true

			return [
				entry.actor.username,
				showWorldColumn.value ? entry.world?.name : undefined,
				entry.event.searchText,
				entry.event.key,
			]
				.filter((value): value is string => typeof value === 'string' && value.length > 0)
				.some((value) => value.toLowerCase().includes(normalizedQuery))
		})
		.slice()
		.sort((a, b) => {
			const leftTimestamp = new Date(a.timestamp).getTime()
			const rightTimestamp = new Date(b.timestamp).getTime()
			return sortDirection.value === 'asc'
				? leftTimestamp - rightTimestamp
				: rightTimestamp - leftTimestamp
		})
})

const tableEntries = computed<AuditLogTableRow[]>(() => filteredEntries.value as AuditLogTableRow[])
const contentFrameStyle = computed(() =>
	contentHeight.value === null ? undefined : { height: `${contentHeight.value}px` },
)

watch(
	() => [filteredEntries.value.length, props.loading] as const,
	() => updateContentHeight(),
	{ flush: 'post' },
)

const hasActiveTimeframeFilter = computed(() => {
	if (timeframeMode.value === 'preset') {
		return timeframePreset.value !== 'all_time'
	}

	if (timeframeMode.value === 'last') {
		return true
	}

	return Boolean(timeframeCustomStartDate.value || timeframeCustomEndDate.value)
})

const hasActiveFilters = computed(
	() =>
		props.hasActiveExternalFilters ||
		query.value.trim().length > 0 ||
		hasActiveTimeframeFilter.value ||
		!!filters.value.userId ||
		(showWorldColumn.value && !!filters.value.worldId),
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

function updateContentHeightObserver() {
	contentResizeObserver?.disconnect()
	contentResizeObserver = null

	nextTick(() => {
		updateContentHeight()

		if (!contentBody.value || typeof ResizeObserver === 'undefined') {
			return
		}

		contentResizeObserver = new ResizeObserver((entries) => {
			const height =
				entries[0]?.contentRect.height ?? contentBody.value?.getBoundingClientRect().height ?? 0
			setContentHeight(height)
		})
		contentResizeObserver.observe(contentBody.value)
	})
}

function updateContentHeight() {
	nextTick(() => {
		if (!contentBody.value) return
		setContentHeight(contentBody.value.getBoundingClientRect().height)
	})
}

function setContentHeight(height: number) {
	contentHeight.value = Math.ceil(height)
}

function scheduleSortTransitionReset(delay: number) {
	if (sortTransitionResetTimeout) {
		clearTimeout(sortTransitionResetTimeout)
	}

	sortTransitionResetTimeout = setTimeout(() => {
		suppressSortRowTransitions.value = false
		sortTransitionResetTimeout = null
	}, delay)
}
</script>

<style>
@media (prefers-reduced-motion: no-preference) {
	.audit-log-content-frame {
		transition: height 220ms ease-in-out;
	}
}

@keyframes audit-log-bpulse {
	50% {
		filter: brightness(75%);
	}
}

.animate-audit-log-bpulse {
	animation: audit-log-bpulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

.audit-log-table :is(th, td) {
	height: 54px;
	padding-left: 0.5rem;
	padding-right: 0.5rem;
	vertical-align: middle;
}

.audit-log-table :is(th, td):first-child {
	padding-left: 1rem;
}

.audit-log-table :is(th, td):last-child {
	padding-right: 1rem;
}

.audit-log-table tbody tr {
	height: 54px;
	max-height: 54px;
}

.audit-log-table tbody td {
	height: 54px;
	max-height: 54px;
	padding-bottom: 0;
	padding-top: 0;
}

.audit-log-table-event {
	line-height: 1.375rem;
}

.audit-log-table-event-component > span {
	line-height: inherit;
}

@container (min-width: 1040px) {
	.audit-log-table :is(th, td) {
		padding-left: 0.75rem;
		padding-right: 0.75rem;
	}

	.audit-log-table :is(th, td):first-child {
		padding-left: 1rem;
	}

	.audit-log-table :is(th, td):last-child {
		padding-right: 1rem;
	}

	.audit-log-table-event {
		line-height: 1.375rem;
	}
}

.audit-log-loading-fade-enter-active,
.audit-log-loading-fade-leave-active {
	transition: opacity 250ms ease-in-out;
}

.audit-log-loading-fade-enter-from,
.audit-log-loading-fade-leave-to {
	opacity: 0;
}

.audit-log-row-enter-active,
.audit-log-row-leave-active,
.audit-log-row-move,
.audit-log-card-enter-active,
.audit-log-card-leave-active,
.audit-log-card-move {
	transition:
		opacity 180ms ease-in-out,
		transform 180ms ease-in-out;
}

.audit-log-row-enter-from,
.audit-log-card-enter-from {
	opacity: 0;
	transform: translateY(-8px);
}

.audit-log-row-leave-to,
.audit-log-card-leave-to {
	opacity: 0;
	transform: translateY(8px);
}
</style>
