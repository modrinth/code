<template>
	<div v-if="canRender" ref="chartElement" class="pointer-events-none absolute inset-0">
		<div
			v-for="group in eventGroups"
			:key="`${group.id}:guide`"
			aria-hidden="true"
			class="absolute left-0 z-0 border-0 border-l border-dashed opacity-40"
			:class="activeGroup?.id === group.id ? 'border-contrast' : 'border-secondary'"
			:style="getGuideStyle(group)"
		/>
		<Transition name="analytics-event-range-highlight-fade">
			<div
				v-if="rangeHighlight"
				aria-hidden="true"
				class="pointer-events-none absolute left-0 z-10 rounded-sm border border-l-0 border-dashed border-secondary bg-highlight-blue opacity-40"
				:style="rangeHighlight"
			/>
		</Transition>

		<button
			v-for="group in eventGroups"
			:key="group.id"
			type="button"
			class="pointer-events-auto absolute -top-[26px] left-0 z-20 inline-flex h-5 min-w-5 cursor-default items-center justify-center gap-1 rounded-full bg-surface-3 px-1 shadow-lg transition-colors focus-visible:border-brand focus-visible:text-contrast"
			:class="activeGroup?.id === group.id ? 'border-brand text-contrast' : 'text-secondary'"
			:style="getMarkerStyle(group)"
			:aria-label="getGroupAriaLabel(group)"
			@click.stop
			@mouseenter="scheduleHoveredGroupOpen(group.id)"
			@mouseleave="scheduleHoverClose"
			@focus="showHoveredGroup(group.id)"
			@blur="scheduleHoverClose"
			@keydown.escape.stop="clearActiveGroup"
		>
			<TagCategoryFlagIcon
				v-if="group.markerIcon === 'flag'"
				class="relative top-px size-5"
				aria-hidden="true"
			/>
			<InfoIcon v-else class="size-5" aria-hidden="true" />
			<span v-if="group.events.length > 1" class="text-xs font-semibold leading-none">
				{{ group.events.length }}
			</span>
		</button>
	</div>
	<Teleport to="body">
		<Transition name="analytics-event-tooltip-fade">
			<div
				v-if="activeGroup"
				ref="tooltipElement"
				class="analytics-event-tooltip pointer-events-auto fixed left-0 top-0 z-[100] max-h-[360px] w-[min(12rem,calc(100vw-1rem))] overflow-y-auto rounded-xl border border-solid border-surface-5 bg-surface-3 py-2 text-sm shadow-xl"
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
							v-if="event.announcement_url"
							:href="event.announcement_url"
							target="_blank"
							rel="noopener noreferrer"
							class="mt-1.5 inline-flex items-center gap-1 text-sm font-medium text-primary underline !transition-all hover:text-contrast"
						>
							See announcement
							<ExternalIcon class="size-3.5" aria-hidden="true" />
						</a>
						<div class="mt-1 text-xs font-semibold text-primary">
							{{ event.subtitle ?? formatEventRange(event) }}
						</div>
					</div>
				</div>
			</div>
		</Transition>
	</Teleport>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ExternalIcon, InfoIcon, TagCategoryFlagIcon } from '@modrinth/assets'

import type {
	AnalyticsDashboardStat,
	AnalyticsGroupByPreset,
} from '~/providers/analytics/analytics'

import type { AnalyticsChartGeometryPayload } from './AnalyticsChart.client.vue'
import { isTimeRelevantForGroupBy } from './utils'

type AnalyticsChartEventMarkerIcon = 'info' | 'flag'

export type AnalyticsChartEvent = {
	title: string
	starts: string
	ends: string
	subtitle?: string
	announcement_url?: string | null
	for_metric_kind?: Labrinth.Analytics.v3.AnalyticsEventMetricKind[] | null
	markerIcon?: AnalyticsChartEventMarkerIcon
	groupKey?: string
}

type PositionedEvent = AnalyticsChartEvent & {
	startMs: number
	endMs: number
	x: number
	endX: number
	markerIcon: AnalyticsChartEventMarkerIcon
	groupKey: string
}

type EventGroup = {
	id: string
	x: number
	markerOffsetX: number
	markerIcon: AnalyticsChartEventMarkerIcon
	groupKey: string
	events: PositionedEvent[]
}

const props = defineProps<{
	events: AnalyticsChartEvent[]
	activeStat: AnalyticsDashboardStat
	groupBy: AnalyticsGroupByPreset
	chartStart: Date | null
	chartEnd: Date | null
	geometry: AnalyticsChartGeometryPayload | null
	containerWidth: number
	containerHeight: number
	markerIcon?: AnalyticsChartEventMarkerIcon
	markerOffsetY?: number
}>()

const GROUP_DISTANCE_PX = 32
const GROUP_MARKER_GAP_PX = 6
const MARKER_ICON_WIDTH_PX = 20
const MARKER_HORIZONTAL_PADDING_PX = 8
const MARKER_COUNT_GAP_PX = 4
const MARKER_COUNT_DIGIT_WIDTH_PX = 7
const MARKER_HEIGHT_PX = 28
const TOOLTIP_OFFSET_PX = 8
const EDGE_PADDING_PX = 8
const OPEN_DELAY_MS = 300
const CLOSE_DELAY_MS = 120
const EVENT_RANGE_DATE_TIME_FORMATTER = new Intl.DateTimeFormat(undefined, {
	month: 'long',
	day: 'numeric',
	year: 'numeric',
	hour: 'numeric',
	minute: '2-digit',
})
const EVENT_RANGE_DATE_FORMATTER = new Intl.DateTimeFormat(undefined, {
	month: 'long',
	day: 'numeric',
	year: 'numeric',
})
const EVENT_RANGE_MONTH_DAY_TIME_FORMATTER = new Intl.DateTimeFormat(undefined, {
	month: 'long',
	day: 'numeric',
	hour: 'numeric',
	minute: '2-digit',
})
const EVENT_RANGE_TIME_FORMATTER = new Intl.DateTimeFormat(undefined, {
	hour: 'numeric',
	minute: '2-digit',
})

const hoveredGroupId = ref<string | null>(null)
const isTooltipHovered = ref(false)
const chartElement = ref<HTMLDivElement | null>(null)
const tooltipElement = ref<HTMLDivElement | null>(null)
const tooltipWidth = ref(0)
const tooltipHeight = ref(0)
const chartRect = reactive({
	left: 0,
	top: 0,
})
let closeTimeout: ReturnType<typeof setTimeout> | null = null
let openTimeout: ReturnType<typeof setTimeout> | null = null

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
			const eventStartMs = new Date(event.starts).getTime()
			const eventEndMs = new Date(event.ends).getTime()
			if (!Number.isFinite(eventStartMs) || !Number.isFinite(eventEndMs)) return null
			if (eventEndMs < eventStartMs) return null
			if (eventEndMs < startMs || eventStartMs > endMs) return null

			const x = getDateBucketX(event.starts, eventStartMs, geometry, startMs, endMs)
			const endX = getDateBucketX(event.ends, eventEndMs, geometry, startMs, endMs)
			if (x === null || endX === null) return null

			return {
				...event,
				startMs: eventStartMs,
				endMs: eventEndMs,
				x,
				endX,
				markerIcon: getEventMarkerIcon(event),
				groupKey: getEventGroupKey(event),
			}
		})
		.filter((event): event is PositionedEvent => Boolean(event))
		.sort((a, b) => a.x - b.x || a.startMs - b.startMs || a.title.localeCompare(b.title))
})

const eventGroups = computed<EventGroup[]>(() => {
	const groups = mergeNearbyEventGroups(buildInitialEventGroups(visibleEvents.value))
	const resolvedGroups = groups.map((group) => ({
		...group,
		id: `${group.groupKey}:${group.events.map(getEventKey).join('|')}`,
	}))

	return applyCollisionOffsets(resolvedGroups)
})

function buildInitialEventGroups(events: PositionedEvent[]): EventGroup[] {
	const groups: EventGroup[] = []
	const previousGroupsByKey = new Map<string, EventGroup>()

	for (const event of events) {
		const previousGroup = previousGroupsByKey.get(event.groupKey)
		const group = createEventGroup(event)
		if (previousGroup && shouldMergeEventGroups(previousGroup, group)) {
			mergeEventGroup(previousGroup, group)
			continue
		}

		groups.push(group)
		previousGroupsByKey.set(event.groupKey, group)
	}

	return groups
}

function mergeNearbyEventGroups(groups: EventGroup[]): EventGroup[] {
	let nextGroups = groups
	let didMerge = true

	while (didMerge) {
		const result = mergeNearbyEventGroupsOnce(nextGroups)
		nextGroups = result.groups
		didMerge = result.didMerge
	}

	return nextGroups
}

function mergeNearbyEventGroupsOnce(groups: EventGroup[]) {
	const mergedGroups: EventGroup[] = []
	const previousGroupsByKey = new Map<string, EventGroup>()
	let didMerge = false

	for (const group of groups) {
		const nextGroup = cloneEventGroup(group)
		const previousGroup = previousGroupsByKey.get(nextGroup.groupKey)
		if (previousGroup && shouldMergeEventGroups(previousGroup, nextGroup)) {
			mergeEventGroup(previousGroup, nextGroup)
			didMerge = true
			continue
		}

		mergedGroups.push(nextGroup)
		previousGroupsByKey.set(nextGroup.groupKey, nextGroup)
	}

	return {
		groups: mergedGroups,
		didMerge,
	}
}

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

	const left = Math.min(...rangedEvents.map((event) => event.x))
	const right = Math.max(...rangedEvents.map((event) => event.endX))

	return {
		left: Math.min(left, right),
		right: Math.max(left, right),
	}
})

const rangeHighlight = computed(() => {
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
	const availableHeight =
		containerHeight.value -
		MARKER_HEIGHT_PX -
		EDGE_PADDING_PX -
		Math.max(props.markerOffsetY ?? 0, 0)
	return Math.min(availableHeight, preferredTop)
})
const markerOffsetTop = computed(() => markerTop.value + (props.markerOffsetY ?? 0))

const tooltipStyle = computed(() => {
	const group = activeGroup.value
	if (!group) return {}

	const viewportWidth = typeof window === 'undefined' ? containerWidth.value : window.innerWidth
	const viewportHeight = typeof window === 'undefined' ? containerHeight.value : window.innerHeight
	const desiredLeft = chartRect.left + group.x + group.markerOffsetX - tooltipWidth.value / 2
	const maxLeft = Math.max(EDGE_PADDING_PX, viewportWidth - tooltipWidth.value - EDGE_PADDING_PX)
	const left = Math.min(maxLeft, Math.max(EDGE_PADDING_PX, desiredLeft))

	const desiredTop =
		chartRect.top +
		markerOffsetTop.value -
		tooltipHeight.value -
		TOOLTIP_OFFSET_PX -
		MARKER_HEIGHT_PX
	const maxTop = Math.max(EDGE_PADDING_PX, viewportHeight - tooltipHeight.value - EDGE_PADDING_PX)
	const top = Math.min(maxTop, Math.max(EDGE_PADDING_PX, desiredTop))

	return {
		transform: `translate3d(${left}px, ${top}px, 0)`,
	}
})

function updateChartRect() {
	if (!chartElement.value) return
	const rect = chartElement.value.getBoundingClientRect()
	chartRect.left = rect.left
	chartRect.top = rect.top
}

function doesEventMatchActiveStat(event: AnalyticsChartEvent) {
	if (!event.for_metric_kind?.length) return true

	return event.for_metric_kind.some((metricKind) => {
		return metricKind === props.activeStat
	})
}

function getEventKey(event: AnalyticsChartEvent) {
	return `${event.title}:${event.starts}:${event.ends}:${event.announcement_url ?? ''}:${
		event.for_metric_kind?.join(',') ?? ''
	}:${event.subtitle ?? ''}`
}

function getEventMarkerIcon(event: AnalyticsChartEvent): AnalyticsChartEventMarkerIcon {
	return event.markerIcon ?? props.markerIcon ?? 'info'
}

function getEventGroupKey(event: AnalyticsChartEvent): string {
	return event.groupKey ?? getEventMarkerIcon(event)
}

function createEventGroup(event: PositionedEvent): EventGroup {
	return {
		id: getEventKey(event),
		x: event.x,
		markerOffsetX: 0,
		markerIcon: event.markerIcon,
		groupKey: event.groupKey,
		events: [event],
	}
}

function cloneEventGroup(group: EventGroup): EventGroup {
	return {
		...group,
		events: [...group.events],
	}
}

function shouldMergeEventGroups(left: EventGroup, right: EventGroup) {
	if (left.groupKey !== right.groupKey) return false
	return (
		right.x - left.x <= GROUP_DISTANCE_PX || getGroupMarkerGap(left, right) <= GROUP_MARKER_GAP_PX
	)
}

function mergeEventGroup(target: EventGroup, source: EventGroup) {
	target.events.push(...source.events)
	target.events.sort((left, right) => left.x - right.x || left.startMs - right.startMs)
	target.x = target.events.reduce((sum, event) => sum + event.x, 0) / target.events.length
}

function getGroupMarkerGap(left: EventGroup, right: EventGroup) {
	return getGroupLeftEdge(right) - getGroupRightEdge(left)
}

function getGroupLeftEdge(group: EventGroup) {
	return group.x - getEstimatedMarkerWidth(group) / 2
}

function getGroupRightEdge(group: EventGroup) {
	return group.x + getEstimatedMarkerWidth(group) / 2
}

function applyCollisionOffsets(groups: EventGroup[]): EventGroup[] {
	const sortedGroups = [...groups].sort(
		(left, right) =>
			left.x - right.x ||
			getMarkerIconOrder(left.markerIcon) - getMarkerIconOrder(right.markerIcon) ||
			left.groupKey.localeCompare(right.groupKey),
	)
	const offsetByGroupId = new Map<string, number>()
	let cluster: EventGroup[] = []

	function commitCluster() {
		if (cluster.length === 0) return
		const layout = getCollisionClusterLayout(cluster)
		let cursor = layout.left
		layout.groups.forEach((group, index) => {
			const markerWidth = layout.markerWidths[index]
			const targetX = cursor + markerWidth / 2
			offsetByGroupId.set(group.id, targetX - group.x)
			cursor += markerWidth + GROUP_MARKER_GAP_PX
		})
		cluster = []
	}

	for (const group of sortedGroups) {
		const groupLeftEdge = getGroupLeftEdge(group)
		const clusterRightEdge =
			cluster.length > 0 ? getCollisionClusterLayout(cluster).right : Number.NEGATIVE_INFINITY
		if (cluster.length > 0 && groupLeftEdge - clusterRightEdge > GROUP_MARKER_GAP_PX) {
			commitCluster()
		}
		cluster.push(group)
	}
	commitCluster()

	return groups.map((group) => ({
		...group,
		markerOffsetX: offsetByGroupId.get(group.id) ?? 0,
	}))
}

function getMarkerIconOrder(markerIcon: AnalyticsChartEventMarkerIcon) {
	return markerIcon === 'info' ? 0 : 1
}

function getCollisionClusterLayout(groups: EventGroup[]) {
	const sortedGroups = [...groups].sort(
		(left, right) =>
			getMarkerIconOrder(left.markerIcon) - getMarkerIconOrder(right.markerIcon) ||
			left.x - right.x ||
			left.groupKey.localeCompare(right.groupKey),
	)
	const markerWidths = sortedGroups.map(getEstimatedMarkerWidth)
	const totalWidth =
		markerWidths.reduce((sum, width) => sum + width, 0) +
		Math.max(0, sortedGroups.length - 1) * GROUP_MARKER_GAP_PX
	const originalLeft = Math.min(...groups.map(getGroupLeftEdge))
	const originalRight = Math.max(...groups.map(getGroupRightEdge))
	const center = (originalLeft + originalRight) / 2
	const left = center - totalWidth / 2
	return {
		groups: sortedGroups,
		markerWidths,
		left,
		right: left + totalWidth,
	}
}

function getEstimatedMarkerWidth(group: EventGroup) {
	const countWidth =
		group.events.length > 1
			? MARKER_COUNT_GAP_PX + String(group.events.length).length * MARKER_COUNT_DIGIT_WIDTH_PX
			: 0
	return MARKER_ICON_WIDTH_PX + MARKER_HORIZONTAL_PADDING_PX + countWidth
}

function getDateBucketX(
	value: string,
	fallbackMs: number,
	geometry: AnalyticsChartGeometryPayload,
	startMs: number,
	endMs: number,
) {
	if (isTimeRelevantForGroupBy(props.groupBy)) {
		const clampedMs = Math.max(startMs, Math.min(endMs, fallbackMs))
		return getTimeAxisX(clampedMs, geometry, startMs, endMs)
	}

	const dateInputValue = getEventDateInputValue(value)
	const xPositions = geometry.xPositions
	const bucketMs = xPositions.length > 0 ? (endMs - startMs) / xPositions.length : 0
	if (!dateInputValue || bucketMs <= 0) {
		const clampedMs = Math.max(startMs, Math.min(endMs, fallbackMs))
		return getTimeAxisX(clampedMs, geometry, startMs, endMs)
	}

	for (let index = 0; index < xPositions.length; index++) {
		const bucketDate = getBucketDateForEventSnap(index, bucketMs, startMs)
		if (getDateInputValue(bucketDate) === dateInputValue) {
			const x = xPositions[index]
			return Number.isFinite(x) ? x : null
		}
	}

	const clampedMs = Math.max(startMs, Math.min(endMs, fallbackMs))
	return getTimeAxisX(clampedMs, geometry, startMs, endMs)
}

function getBucketDateForEventSnap(index: number, bucketMs: number, startMs: number): Date {
	const bucketOffset = isTimeRelevantForGroupBy(props.groupBy) ? index : index + 1
	return new Date(startMs + bucketOffset * bucketMs)
}

function getTimeAxisX(
	targetMs: number,
	geometry: AnalyticsChartGeometryPayload,
	startMs: number,
	endMs: number,
) {
	const xPositions = geometry.xPositions
	const bucketMs = xPositions.length > 0 ? (endMs - startMs) / xPositions.length : 0
	if (xPositions.length === 0 || bucketMs <= 0) return null
	if (xPositions.length === 1) return xPositions[0]

	const firstX = xPositions[0]
	const lastX = xPositions[xPositions.length - 1]
	if (!Number.isFinite(firstX) || !Number.isFinite(lastX)) return null

	const firstBucketEndMs = startMs + bucketMs
	const clampedTargetMs = Math.min(endMs, Math.max(firstBucketEndMs, targetMs))
	const progress = (clampedTargetMs - firstBucketEndMs) / (endMs - firstBucketEndMs)

	return firstX + progress * (lastX - firstX)
}

function getMarkerStyle(group: EventGroup) {
	return {
		transform: `translate(-50%, 0) translate(${
			group.x + group.markerOffsetX
		}px, ${markerOffsetTop.value}px)`,
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

function clearOpenTimeout() {
	if (!openTimeout) return
	clearTimeout(openTimeout)
	openTimeout = null
}

function showHoveredGroup(groupId: string) {
	clearOpenTimeout()
	clearCloseTimeout()
	updateChartRect()
	hoveredGroupId.value = groupId
}

function scheduleHoveredGroupOpen(groupId: string) {
	clearOpenTimeout()
	clearCloseTimeout()
	if (hoveredGroupId.value === groupId) {
		updateChartRect()
		return
	}

	hoveredGroupId.value = null
	isTooltipHovered.value = false
	openTimeout = setTimeout(() => {
		showHoveredGroup(groupId)
		openTimeout = null
	}, OPEN_DELAY_MS)
}

function scheduleHoverClose() {
	clearOpenTimeout()
	clearCloseTimeout()
	closeTimeout = setTimeout(() => {
		if (!isTooltipHovered.value) {
			hoveredGroupId.value = null
		}
		closeTimeout = null
	}, CLOSE_DELAY_MS)
}

function clearActiveGroup() {
	clearOpenTimeout()
	clearCloseTimeout()
	hoveredGroupId.value = null
	isTooltipHovered.value = false
}

function onTooltipMouseEnter() {
	clearOpenTimeout()
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
	const parsedDate = new Date(value)
	if (Number.isNaN(parsedDate.getTime())) return null
	return getDateInputValue(parsedDate)
}

function formatEventRange(event: AnalyticsChartEvent) {
	const startDate = new Date(event.starts)
	const endDate = new Date(event.ends)
	if (Number.isNaN(startDate.getTime()) || Number.isNaN(endDate.getTime())) {
		return `${event.starts} - ${event.ends}`
	}

	const startDateValue = getDateInputValue(startDate)
	const endDateValue = getDateInputValue(endDate)

	const sameYear = startDate.getFullYear() === endDate.getFullYear()

	if (startDate.getTime() === endDate.getTime()) {
		return EVENT_RANGE_DATE_TIME_FORMATTER.format(startDate)
	}

	if (startDateValue === endDateValue) {
		return `${EVENT_RANGE_DATE_FORMATTER.format(startDate)}, ${EVENT_RANGE_TIME_FORMATTER.format(
			startDate,
		)} - ${EVENT_RANGE_TIME_FORMATTER.format(endDate)}`
	}

	if (sameYear) {
		const startLabel = EVENT_RANGE_MONTH_DAY_TIME_FORMATTER.format(startDate)
		const endLabel = EVENT_RANGE_MONTH_DAY_TIME_FORMATTER.format(endDate)
		return `${startLabel} - ${endLabel}, ${startDate.getFullYear()}`
	}

	const startLabel = EVENT_RANGE_DATE_TIME_FORMATTER.format(startDate)
	const endLabel = EVENT_RANGE_DATE_TIME_FORMATTER.format(endDate)
	return `${startLabel} - ${endLabel}`
}

watch(
	() => [activeGroup.value, props.containerWidth, props.containerHeight],
	() => {
		nextTick(() => {
			updateChartRect()
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

onMounted(() => {
	updateChartRect()
	window.addEventListener('resize', updateChartRect)
	window.addEventListener('scroll', updateChartRect, true)
})

onBeforeUnmount(() => {
	clearOpenTimeout()
	clearCloseTimeout()
	window.removeEventListener('resize', updateChartRect)
	window.removeEventListener('scroll', updateChartRect, true)
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

.analytics-event-range-highlight-fade-enter-active,
.analytics-event-range-highlight-fade-leave-active {
	transition: opacity 140ms ease-out;
}

.analytics-event-range-highlight-fade-enter-from,
.analytics-event-range-highlight-fade-leave-to {
	opacity: 0;
}
</style>
