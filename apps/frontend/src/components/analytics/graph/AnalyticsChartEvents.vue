<template>
	<div v-if="canRender" class="pointer-events-none absolute inset-0">
		<div
			v-for="group in eventGroups"
			:key="`${group.id}:guide`"
			aria-hidden="true"
			class="absolute z-0 border-0 border-l border-dashed border-secondary opacity-40"
			:style="getGuideStyle(group)"
		/>

		<button
			v-for="group in eventGroups"
			:key="group.id"
			type="button"
			class="pointer-events-auto absolute -top-[34px] z-20 inline-flex h-8 min-w-8 items-center justify-center gap-1 rounded-full bg-surface-3 px-1 text-secondary shadow-lg transition-colors hover:text-contrast focus-visible:border-brand focus-visible:text-contrast"
			:class="activeGroup?.id === group.id ? 'border-brand text-contrast' : ''"
			:style="getMarkerStyle(group)"
			:aria-label="getGroupAriaLabel(group)"
			@click.stop
			@mouseenter="showHoveredGroup(group.id)"
			@mouseleave="scheduleHoverClose"
			@focus="showHoveredGroup(group.id)"
			@blur="scheduleHoverClose"
			@keydown.escape.stop="clearActiveGroup"
		>
			<InfoIcon class="size-5" aria-hidden="true" />
			<span v-if="group.events.length > 1" class="text-xs font-semibold leading-none">
				{{ group.events.length }}
			</span>
		</button>

		<Transition name="analytics-event-tooltip-fade">
			<div
				v-if="activeGroup"
				ref="tooltipElement"
				class="analytics-event-tooltip pointer-events-auto absolute -top-7 left-0 z-30 max-h-[360px] w-[min(20rem,calc(100%-1rem))] overflow-y-auto rounded-xl border border-solid border-surface-5 bg-surface-3 py-2 text-sm shadow-xl"
				:style="tooltipStyle"
				@mouseenter="onTooltipMouseEnter"
				@mouseleave="onTooltipMouseLeave"
				@focusin="onTooltipMouseEnter"
				@focusout="onTooltipMouseLeave"
				@click.stop
			>
				<div class="flex flex-col gap-2.5">
					<div
						v-for="event in activeGroup.events"
						:key="getEventKey(event)"
						class="border-0 border-b border-solid border-surface-5 px-3 pb-2.5 last:border-b-0 last:pb-0"
					>
						<div class="font-semibold leading-snug text-contrast">
							{{ event.title }}
						</div>
						<a
							v-if="event.announcementUrl"
							:href="event.announcementUrl"
							target="_blank"
							rel="noopener noreferrer"
							class="mt-1.5 inline-flex items-center gap-1 text-sm font-medium text-primary underline !transition-all hover:text-contrast"
						>
							See announcement
							<ExternalIcon class="size-3.5" aria-hidden="true" />
						</a>
						<div class="mt-2.5 text-xs font-semibold text-primary">
							{{ formatEventRange(event) }}
						</div>
					</div>
				</div>
			</div>
		</Transition>
	</div>
</template>

<script setup lang="ts">
import { ExternalIcon, InfoIcon } from '@modrinth/assets'

import type { AnalyticsDashboardStat } from '~/providers/analytics/analytics'

import type { AnalyticsChartGeometryPayload } from './AnalyticsChart.client.vue'

export type AnalyticsChartEvent = {
	title: string
	announcementUrl?: string
	forMetricType?: 'view' | 'revenue' | 'downloads' | 'playtime'
	startDate: string
	endDate: string
}

type PositionedEvent = AnalyticsChartEvent & {
	startMs: number
	endMs: number
	x: number
}

type EventGroup = {
	id: string
	x: number
	events: PositionedEvent[]
}

const props = defineProps<{
	events: AnalyticsChartEvent[]
	activeStat: AnalyticsDashboardStat
	chartStart: Date | null
	chartEnd: Date | null
	geometry: AnalyticsChartGeometryPayload | null
	containerWidth: number
	containerHeight: number
}>()

const GROUP_DISTANCE_PX = 32
const MARKER_HEIGHT_PX = 28
const TOOLTIP_OFFSET_PX = 10
const EDGE_PADDING_PX = 8
const CLOSE_DELAY_MS = 120
const EVENT_RANGE_DATE_FORMATTER = new Intl.DateTimeFormat('en-US', {
	month: 'long',
	day: 'numeric',
	year: 'numeric',
})
const EVENT_RANGE_MONTH_DAY_FORMATTER = new Intl.DateTimeFormat('en-US', {
	month: 'long',
	day: 'numeric',
})

const hoveredGroupId = ref<string | null>(null)
const isTooltipHovered = ref(false)
const tooltipElement = ref<HTMLDivElement | null>(null)
const tooltipWidth = ref(0)
const tooltipHeight = ref(0)
let closeTimeout: ReturnType<typeof setTimeout> | null = null

const chartStartMs = computed(() => props.chartStart?.getTime() ?? null)
const chartEndMs = computed(() => props.chartEnd?.getTime() ?? null)
const containerWidth = computed(() => props.containerWidth || props.geometry?.width || 0)
const containerHeight = computed(() => props.containerHeight || props.geometry?.height || 0)
const canRender = computed(
	() =>
		props.geometry !== null &&
		containerWidth.value > 0 &&
		containerHeight.value > 0 &&
		chartStartMs.value !== null &&
		chartEndMs.value !== null &&
		chartEndMs.value > chartStartMs.value &&
		props.geometry.xPositions.length > 0 &&
		eventGroups.value.length > 0,
)

const visibleEvents = computed<PositionedEvent[]>(() => {
	const geometry = props.geometry
	const startMs = chartStartMs.value
	const endMs = chartEndMs.value
	if (!geometry || startMs === null || endMs === null || endMs <= startMs) return []

	return props.events
		.filter((event) => doesEventMatchActiveStat(event))
		.map((event) => {
			const eventStartMs = new Date(event.startDate).getTime()
			const eventEndMs = new Date(event.endDate).getTime()
			if (!Number.isFinite(eventStartMs) || !Number.isFinite(eventEndMs)) return null
			if (eventEndMs < eventStartMs) return null
			if (eventEndMs < startMs || eventStartMs > endMs) return null

			const clampedStartMs = Math.max(startMs, Math.min(endMs, eventStartMs))
			const x = getClosestBucketX(clampedStartMs, geometry, startMs, endMs)
			if (x === null) return null

			return {
				...event,
				startMs: eventStartMs,
				endMs: eventEndMs,
				x,
			}
		})
		.filter((event): event is PositionedEvent => Boolean(event))
		.sort((a, b) => a.x - b.x || a.startMs - b.startMs || a.title.localeCompare(b.title))
})

const eventGroups = computed<EventGroup[]>(() => {
	const groups: EventGroup[] = []

	for (const event of visibleEvents.value) {
		const previousGroup = groups[groups.length - 1]
		if (previousGroup && event.x - previousGroup.x <= GROUP_DISTANCE_PX) {
			previousGroup.events.push(event)
			previousGroup.x =
				previousGroup.events.reduce((sum, groupedEvent) => sum + groupedEvent.x, 0) /
				previousGroup.events.length
			continue
		}

		groups.push({
			id: getEventKey(event),
			x: event.x,
			events: [event],
		})
	}

	return groups.map((group) => ({
		...group,
		x: getClosestRenderedBucketX(group.x),
		id: group.events.map(getEventKey).join('|'),
	}))
})

const activeGroup = computed(
	() => eventGroups.value.find((group) => group.id === hoveredGroupId.value) ?? null,
)

const activeRange = computed(() => {
	const group = activeGroup.value
	const geometry = props.geometry
	const startMs = chartStartMs.value
	const endMs = chartEndMs.value
	if (!group || !geometry || startMs === null || endMs === null || endMs <= startMs) return null

	const rangedEvents = group.events.filter((event) => event.startMs !== event.endMs)
	if (rangedEvents.length === 0) return null

	const rangeStartMs = Math.max(startMs, Math.min(...rangedEvents.map((event) => event.startMs)))
	const rangeEndMs = Math.min(endMs, Math.max(...rangedEvents.map((event) => event.endMs)))
	if (rangeEndMs <= rangeStartMs) return null

	const left =
		geometry.left +
		((rangeStartMs - startMs) / (endMs - startMs)) * (geometry.right - geometry.left)
	const right =
		geometry.left + ((rangeEndMs - startMs) / (endMs - startMs)) * (geometry.right - geometry.left)

	return {
		left: Math.min(left, right),
		right: Math.max(left, right),
	}
})

const highlightStyle = computed(() => {
	const range = activeRange.value
	const geometry = props.geometry
	if (!range || !geometry) return null

	return {
		top: `${geometry.top}px`,
		height: `${geometry.bottom - geometry.top}px`,
		transform: `translate(${range.left}px, 0)`,
		width: `${Math.max(1, range.right - range.left)}px`,
	}
})

const markerTop = computed(() => {
	const geometry = props.geometry
	if (!geometry) return 0
	const preferredTop = geometry.top
	return Math.min(containerHeight.value - MARKER_HEIGHT_PX - EDGE_PADDING_PX, preferredTop)
})

const tooltipStyle = computed(() => {
	const group = activeGroup.value
	if (!group) return {}

	const desiredLeft = group.x - tooltipWidth.value / 2
	const maxLeft = Math.max(
		EDGE_PADDING_PX,
		containerWidth.value - tooltipWidth.value - EDGE_PADDING_PX,
	)
	const left = Math.min(maxLeft, Math.max(EDGE_PADDING_PX, desiredLeft))

	const desiredTop = markerTop.value - tooltipHeight.value - TOOLTIP_OFFSET_PX
	const maxTop = Math.max(
		EDGE_PADDING_PX,
		containerHeight.value - tooltipHeight.value - EDGE_PADDING_PX,
	)
	const top = Math.min(maxTop, desiredTop)

	return {
		transform: `translate3d(${left}px, ${top}px, 0)`,
	}
})

function doesEventMatchActiveStat(event: AnalyticsChartEvent) {
	if (!event.forMetricType) return true
	if (event.forMetricType === 'view') return props.activeStat === 'views'
	return event.forMetricType === props.activeStat
}

function getEventKey(event: AnalyticsChartEvent) {
	return `${event.title}:${event.startDate}:${event.endDate}:${event.announcementUrl ?? ''}:${
		event.forMetricType ?? ''
	}`
}

function getClosestBucketX(
	targetMs: number,
	geometry: AnalyticsChartGeometryPayload,
	startMs: number,
	endMs: number,
) {
	const xPositions = geometry.xPositions
	const bucketMs = xPositions.length > 0 ? (endMs - startMs) / xPositions.length : 0
	if (bucketMs <= 0) return null

	const rawIndex = (targetMs - startMs) / bucketMs - 0.5
	const bucketIndex = Math.min(xPositions.length - 1, Math.max(0, Math.round(rawIndex)))
	const x = xPositions[bucketIndex]
	return Number.isFinite(x) ? x : null
}

function getClosestRenderedBucketX(x: number) {
	const xPositions = props.geometry?.xPositions ?? []
	if (xPositions.length === 0) return x

	return xPositions.reduce((closestX, candidateX) =>
		Math.abs(candidateX - x) < Math.abs(closestX - x) ? candidateX : closestX,
	)
}

function getMarkerStyle(group: EventGroup) {
	return {
		transform: `translate(-50%, 0) translate(${group.x}px, ${markerTop.value}px)`,
	}
}

function getGuideStyle(group: EventGroup) {
	const geometry = props.geometry
	if (!geometry) return {}

	return {
		top: `${geometry.top}px`,
		height: `${geometry.bottom - geometry.top}px`,
		transform: `translate(${group.x}px, 0)`,
	}
}

function getGroupAriaLabel(group: EventGroup) {
	if (group.events.length === 1) {
		return group.events[0].title
	}

	return `${group.events.length} analytics events`
}

function clearCloseTimeout() {
	if (!closeTimeout) return
	clearTimeout(closeTimeout)
	closeTimeout = null
}

function showHoveredGroup(groupId: string) {
	clearCloseTimeout()
	hoveredGroupId.value = groupId
}

function scheduleHoverClose() {
	clearCloseTimeout()
	closeTimeout = setTimeout(() => {
		if (!isTooltipHovered.value) {
			hoveredGroupId.value = null
		}
		closeTimeout = null
	}, CLOSE_DELAY_MS)
}

function clearActiveGroup() {
	clearCloseTimeout()
	hoveredGroupId.value = null
	isTooltipHovered.value = false
}

function onTooltipMouseEnter() {
	clearCloseTimeout()
	isTooltipHovered.value = true
}

function onTooltipMouseLeave() {
	isTooltipHovered.value = false
	scheduleHoverClose()
}

function getDateInputValue(date: Date): string {
	const year = date.getFullYear()
	const month = String(date.getMonth() + 1).padStart(2, '0')
	const day = String(date.getDate()).padStart(2, '0')
	return `${year}-${month}-${day}`
}

function getEventDateInputValue(value: string): string | null {
	const dateInputValue = value.match(/^\d{4}-\d{2}-\d{2}/)?.[0]
	if (dateInputValue) return dateInputValue

	const parsedDate = new Date(value)
	if (Number.isNaN(parsedDate.getTime())) return null
	return getDateInputValue(parsedDate)
}

function getDateFromInputValue(value: string): Date | undefined {
	const date = new Date(`${value}T00:00:00`)
	if (Number.isNaN(date.getTime()) || getDateInputValue(date) !== value) {
		return undefined
	}

	return date
}

function formatEventRange(event: AnalyticsChartEvent) {
	const startDateValue = getEventDateInputValue(event.startDate)
	const endDateValue = getEventDateInputValue(event.endDate)
	if (!startDateValue || !endDateValue) {
		return `${event.startDate} - ${event.endDate}`
	}

	const startDate = getDateFromInputValue(startDateValue)
	const endDate = getDateFromInputValue(endDateValue)
	if (!startDate || !endDate) {
		return `${startDateValue} - ${endDateValue}`
	}

	const sameYear = startDate.getFullYear() === endDate.getFullYear()

	if (startDateValue === endDateValue) {
		return EVENT_RANGE_DATE_FORMATTER.format(startDate)
	}

	if (sameYear) {
		const startLabel = EVENT_RANGE_MONTH_DAY_FORMATTER.format(startDate)
		const endLabel = EVENT_RANGE_MONTH_DAY_FORMATTER.format(endDate)
		return `${startLabel} - ${endLabel}, ${startDate.getFullYear()}`
	}

	const startLabel = EVENT_RANGE_DATE_FORMATTER.format(startDate)
	const endLabel = EVENT_RANGE_DATE_FORMATTER.format(endDate)
	return `${startLabel} - ${endLabel}`
}

watch(
	() => [activeGroup.value, props.containerWidth, props.containerHeight],
	() => {
		nextTick(() => {
			if (!tooltipElement.value) return
			tooltipWidth.value = tooltipElement.value.offsetWidth
			tooltipHeight.value = tooltipElement.value.offsetHeight
		})
	},
	{ deep: true, immediate: true },
)

watch(eventGroups, (groups) => {
	const groupIds = new Set(groups.map((group) => group.id))
	if (hoveredGroupId.value && !groupIds.has(hoveredGroupId.value)) {
		hoveredGroupId.value = null
	}
})

onBeforeUnmount(() => {
	clearCloseTimeout()
})
</script>

<style scoped>
.analytics-event-tooltip {
	opacity: 1;
	transition:
		opacity 140ms ease-out,
		transform 180ms ease-out;
	will-change: opacity, transform;
}

.analytics-event-tooltip-fade-enter-from,
.analytics-event-tooltip-fade-leave-to {
	opacity: 0;
}

.analytics-event-tooltip-fade-enter-active,
.analytics-event-tooltip-fade-leave-active {
	transition:
		opacity 140ms ease-out,
		transform 180ms ease-out;
}
</style>
