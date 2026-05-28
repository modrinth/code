<template>
	<div
		v-if="canRender"
		ref="chartElement"
		class="pointer-events-none absolute left-0 top-0"
		:style="chartLayerStyle"
	>
		<div
			v-for="group in eventGroups"
			:key="`${group.id}:guide`"
			aria-hidden="true"
			class="absolute left-0 border-0 border-l border-dashed transition-all"
			:class="
				activeGroup?.id === group.id
					? isModrinthEventGroup(group)
						? 'border-blue opacity-80'
						: 'border-contrast opacity-60'
					: isModrinthEventGroup(group)
						? 'border-blue opacity-50'
						: 'border-secondary opacity-40'
			"
			:style="getGuideStyle(group)"
		/>
		<Transition name="analytics-event-range-highlight-fade">
			<div
				v-if="rangeHighlight"
				aria-hidden="true"
				class="pointer-events-none absolute left-0 rounded-sm border border-l-0 border-dashed border-blue bg-highlight-blue opacity-40"
				:style="rangeHighlight"
			/>
		</Transition>

		<button
			v-for="group in eventGroups"
			:key="group.id"
			type="button"
			class="pointer-events-auto absolute left-0 top-0 inline-flex h-5 min-w-5 cursor-default items-center justify-center gap-1 rounded-full bg-surface-3 px-1 transition-colors focus-visible:border-brand focus-visible:text-contrast"
			:class="
				activeGroup?.id === group.id
					? isModrinthEventGroup(group)
						? 'border-blue text-contrast'
						: 'border-brand text-contrast'
					: 'text-secondary'
			"
			:style="getMarkerStyle(group)"
			:aria-label="getGroupAriaLabel(group)"
			@pointerdown.stop="handleGroupPointerDown(group.id)"
			@click.stop="handleGroupClick(group.id)"
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
			<InfoIcon v-else class="size-5 text-blue" aria-hidden="true" />
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
				class="analytics-event-tooltip pointer-events-auto fixed left-0 top-0 flex max-h-[340px] w-[12rem] flex-col overflow-hidden rounded-xl border border-solid border-surface-5 bg-surface-3 text-sm shadow-xl"
				:style="tooltipStyle"
				@mouseenter="onTooltipMouseEnter"
				@mouseleave="onTooltipMouseLeave"
				@focusin="onTooltipMouseEnter"
				@focusout="onTooltipMouseLeave"
				@click.stop
			>
				<div class="relative flex min-h-0 flex-1 flex-col">
					<Transition
						enter-active-class="transition-all duration-200 ease-out"
						enter-from-class="opacity-0 max-h-0"
						enter-to-class="opacity-100 max-h-6"
						leave-active-class="transition-all duration-200 ease-in"
						leave-from-class="opacity-100 max-h-6"
						leave-to-class="opacity-0 max-h-0"
					>
						<div
							v-if="showTooltipTopFade"
							class="pointer-events-none absolute left-0 right-0 top-0 -mt-1 h-6 bg-gradient-to-b from-bg-raised to-transparent"
						/>
					</Transition>

					<div
						ref="tooltipScrollElement"
						class="overflow-y-auto overscroll-contain py-2"
						@scroll="checkTooltipScrollState"
					>
						<div class="flex flex-col gap-2.5">
							<div
								v-for="event in activeGroup.events"
								:key="getEventKey(event)"
								class="border-0 border-b border-solid border-surface-5 px-3 pb-2.5 last:border-b-0 last:pb-0"
							>
								<div
									:class="
										event.projectName
											? 'font-medium leading-snug text-primary'
											: 'font-medium leading-snug text-contrast'
									"
								>
									<template v-if="event.projectName">
										<span class="text-contrast">{{ event.projectName }}: </span>
										<span>{{ event.title }}</span>
									</template>
									<template v-else>
										{{ event.title }}
									</template>
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
								<div class="mt-1 text-xs font-medium text-primary">
									{{ event.subtitle ?? formatEventRange(event) }}
								</div>
							</div>
						</div>
					</div>

					<Transition
						enter-active-class="transition-all duration-200 ease-out"
						enter-from-class="opacity-0 max-h-0"
						enter-to-class="opacity-100 max-h-8"
						leave-active-class="transition-all duration-200 ease-in"
						leave-from-class="opacity-100 max-h-8"
						leave-to-class="opacity-0 max-h-0"
					>
						<div
							v-if="showTooltipBottomFade"
							class="pointer-events-none absolute bottom-0 left-0 right-0 -mb-1 h-8 bg-gradient-to-t from-bg-raised to-transparent"
						/>
					</Transition>
				</div>
			</div>
		</Transition>
	</Teleport>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ExternalIcon, InfoIcon, TagCategoryFlagIcon } from '@modrinth/assets'
import { useScrollIndicator } from '@modrinth/ui'

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
	projectId?: string
	projectName?: string
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
	xSum: number
	markerOffsetX: number
	markerIcon: AnalyticsChartEventMarkerIcon
	groupKey: string
	events: PositionedEvent[]
}

type CollisionClusterLayout = {
	groups: EventGroup[]
	markerWidths: number[]
	left: number
	right: number
}

const props = defineProps<{
	events: AnalyticsChartEvent[]
	activeStat: AnalyticsDashboardStat
	groupBy: AnalyticsGroupByPreset
	chartStart: Date | null
	chartEnd: Date | null
	geometry: AnalyticsChartGeometryPayload | null
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
const MARKER_TOP_OFFSET_PX = -26
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
const tooltipScrollElement = ref<HTMLDivElement | null>(null)
const tooltipWidth = ref(0)
const tooltipHeight = ref(0)
const chartRect = reactive({
	left: 0,
	top: 0,
})
let closeTimeout: ReturnType<typeof setTimeout> | null = null
let openTimeout: ReturnType<typeof setTimeout> | null = null
let pointerDownGroupId: string | null = null
let wasPointerDownGroupActive = false
const {
	showTopFade: showTooltipTopFade,
	showBottomFade: showTooltipBottomFade,
	checkScrollState: checkTooltipScrollState,
	forceCheck: forceCheckTooltipScrollState,
} = useScrollIndicator(tooltipScrollElement)

const chartStartMs = computed(() => props.chartStart?.getTime() ?? null)
const chartEndMs = computed(() => props.chartEnd?.getTime() ?? null)
const chartWidth = computed(() => props.geometry?.width ?? 0)
const chartHeight = computed(() => props.geometry?.height ?? 0)
const chartLayerStyle = computed(() => ({
	width: `${chartWidth.value}px`,
	height: `${chartHeight.value}px`,
}))
const canRender = computed(
	() =>
		props.geometry !== null &&
		chartWidth.value > 0 &&
		chartHeight.value > 0 &&
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

	const bucketXByDate = getBucketXByDate(geometry, startMs, endMs)

	return props.events
		.filter((event) => doesEventMatchActiveStat(event))
		.map((event) => {
			const eventStartMs = new Date(event.starts).getTime()
			const eventEndMs = new Date(event.ends).getTime()
			if (!Number.isFinite(eventStartMs) || !Number.isFinite(eventEndMs)) return null
			if (eventEndMs < eventStartMs) return null
			if (eventEndMs < startMs || eventStartMs > endMs) return null

			const x = getDateBucketX(event.starts, eventStartMs, geometry, startMs, endMs, bucketXByDate)
			const endX = getDateBucketX(event.ends, eventEndMs, geometry, startMs, endMs, bucketXByDate)
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
		const previousGroup = previousGroupsByKey.get(group.groupKey)
		if (previousGroup && shouldMergeEventGroups(previousGroup, group)) {
			mergeEventGroup(previousGroup, group)
			didMerge = true
			continue
		}

		mergedGroups.push(group)
		previousGroupsByKey.set(group.groupKey, group)
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
	const preferredTop = geometry.top - MARKER_HEIGHT_PX
	const availableHeight =
		chartHeight.value - MARKER_HEIGHT_PX - EDGE_PADDING_PX - Math.max(props.markerOffsetY ?? 0, 0)
	const maxTop = Math.max(EDGE_PADDING_PX, availableHeight)
	return clamp(preferredTop, EDGE_PADDING_PX, maxTop)
})
const markerOffsetTop = computed(() => {
	const maxTop = Math.max(EDGE_PADDING_PX, chartHeight.value - MARKER_HEIGHT_PX - EDGE_PADDING_PX)
	return clamp(markerTop.value + (props.markerOffsetY ?? 0), EDGE_PADDING_PX, maxTop)
})

const tooltipStyle = computed(() => {
	const group = activeGroup.value
	if (!group) return {}

	const maxTooltipWidth = Math.max(0, chartWidth.value - EDGE_PADDING_PX * 2)
	const resolvedTooltipWidth = Math.min(tooltipWidth.value, maxTooltipWidth)
	const resolvedTooltipHeight = tooltipHeight.value
	const markerX = getClampedMarkerCenterX(group)
	const markerViewportLeft = chartRect.left + markerX
	const markerViewportTop = chartRect.top + MARKER_TOP_OFFSET_PX + markerOffsetTop.value
	const desiredLeft = markerViewportLeft - resolvedTooltipWidth / 2
	const maxLeft = Math.max(
		chartRect.left + EDGE_PADDING_PX,
		chartRect.left + chartWidth.value - resolvedTooltipWidth - EDGE_PADDING_PX,
	)
	const left = clamp(desiredLeft, chartRect.left + EDGE_PADDING_PX, maxLeft)

	const desiredTop = markerViewportTop - resolvedTooltipHeight - TOOLTIP_OFFSET_PX
	const viewportHeight = typeof window === 'undefined' ? resolvedTooltipHeight : window.innerHeight
	const maxTop = Math.max(EDGE_PADDING_PX, viewportHeight - resolvedTooltipHeight - EDGE_PADDING_PX)
	const top = clamp(desiredTop, EDGE_PADDING_PX, maxTop)

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
	}:${event.subtitle ?? ''}:${event.projectId ?? ''}:${event.projectName ?? ''}`
}

function getEventMarkerIcon(event: AnalyticsChartEvent): AnalyticsChartEventMarkerIcon {
	return event.markerIcon ?? props.markerIcon ?? 'info'
}

function getEventGroupKey(event: AnalyticsChartEvent): string {
	return event.groupKey ?? getEventMarkerIcon(event)
}

function isModrinthEventGroup(group: EventGroup) {
	return group.groupKey === 'modrinth'
}

function createEventGroup(event: PositionedEvent): EventGroup {
	return {
		id: getEventKey(event),
		x: event.x,
		xSum: event.x,
		markerOffsetX: 0,
		markerIcon: event.markerIcon,
		groupKey: event.groupKey,
		events: [event],
	}
}

function shouldMergeEventGroups(left: EventGroup, right: EventGroup) {
	if (left.groupKey !== right.groupKey) return false
	return (
		right.x - left.x <= GROUP_DISTANCE_PX || getGroupMarkerGap(left, right) <= GROUP_MARKER_GAP_PX
	)
}

function mergeEventGroup(target: EventGroup, source: EventGroup) {
	target.events = mergeSortedGroupEvents(target.events, source.events)
	target.xSum += source.xSum
	target.x = target.xSum / target.events.length
}

// Both event arrays are already sorted, so this does the merge step from merge sort.
function mergeSortedGroupEvents(leftEvents: PositionedEvent[], rightEvents: PositionedEvent[]) {
	const mergedEvents: PositionedEvent[] = []
	let leftIndex = 0
	let rightIndex = 0

	while (leftIndex < leftEvents.length && rightIndex < rightEvents.length) {
		const left = leftEvents[leftIndex]
		const right = rightEvents[rightIndex]
		if (compareGroupEvents(left, right) <= 0) {
			mergedEvents.push(left)
			leftIndex++
			continue
		}

		mergedEvents.push(right)
		rightIndex++
	}

	while (leftIndex < leftEvents.length) {
		mergedEvents.push(leftEvents[leftIndex])
		leftIndex++
	}

	while (rightIndex < rightEvents.length) {
		mergedEvents.push(rightEvents[rightIndex])
		rightIndex++
	}

	return mergedEvents
}

function compareGroupEvents(left: PositionedEvent, right: PositionedEvent) {
	return left.x - right.x || left.startMs - right.startMs
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

function clamp(value: number, min: number, max: number) {
	return Math.min(max, Math.max(min, value))
}

function applyCollisionOffsets(groups: EventGroup[]): EventGroup[] {
	const sortedGroups = [...groups].sort(
		(left, right) =>
			left.x - right.x ||
			getMarkerIconOrder(left.markerIcon) - getMarkerIconOrder(right.markerIcon) ||
			left.groupKey.localeCompare(right.groupKey),
	)
	const offsetByGroupId = new Map<string, number>()
	const clusterLayouts = getStableCollisionClusterLayouts(sortedGroups.map((group) => [group]))

	for (const layout of clusterLayouts) {
		let cursor = layout.left
		layout.groups.forEach((group, index) => {
			const markerWidth = layout.markerWidths[index]
			const targetX = cursor + markerWidth / 2
			offsetByGroupId.set(group.id, targetX - group.x)
			cursor += markerWidth + GROUP_MARKER_GAP_PX
		})
	}

	return groups.map((group) => ({
		...group,
		markerOffsetX: offsetByGroupId.get(group.id) ?? 0,
	}))
}

function getMarkerIconOrder(markerIcon: AnalyticsChartEventMarkerIcon) {
	return markerIcon === 'info' ? 0 : 1
}

function getStableCollisionClusterLayouts(clusters: EventGroup[][]) {
	let layouts = clusters.map(getCollisionClusterLayout).sort(compareCollisionClusterLayouts)

	while (true) {
		const mergedLayouts: CollisionClusterLayout[] = []
		let didMerge = false

		for (const layout of layouts) {
			const previousLayout = mergedLayouts[mergedLayouts.length - 1]
			if (!previousLayout) {
				mergedLayouts.push(layout)
				continue
			}

			if (layout.left - previousLayout.right <= GROUP_MARKER_GAP_PX) {
				mergedLayouts[mergedLayouts.length - 1] = getCollisionClusterLayout([
					...previousLayout.groups,
					...layout.groups,
				])
				didMerge = true
				continue
			}

			mergedLayouts.push(layout)
		}

		if (!didMerge) return mergedLayouts

		layouts = mergedLayouts.sort(compareCollisionClusterLayouts)
	}
}

function compareCollisionClusterLayouts(
	left: CollisionClusterLayout,
	right: CollisionClusterLayout,
) {
	return left.left - right.left || left.right - right.right
}

function getCollisionClusterLayout(groups: EventGroup[]): CollisionClusterLayout {
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
	const preferredLeft = center - totalWidth / 2
	const maxLeft = Math.max(EDGE_PADDING_PX, chartWidth.value - totalWidth - EDGE_PADDING_PX)
	const left = clamp(preferredLeft, EDGE_PADDING_PX, maxLeft)
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

function getClampedMarkerCenterX(group: EventGroup) {
	const markerWidth = getEstimatedMarkerWidth(group)
	const minX = markerWidth / 2 + EDGE_PADDING_PX
	const maxX = Math.max(minX, chartWidth.value - markerWidth / 2 - EDGE_PADDING_PX)
	return clamp(group.x + group.markerOffsetX, minX, maxX)
}

function getBucketXByDate(geometry: AnalyticsChartGeometryPayload, startMs: number, endMs: number) {
	if (isTimeRelevantForGroupBy(props.groupBy)) return null

	const xPositions = geometry.xPositions
	const bucketMs = xPositions.length > 0 ? (endMs - startMs) / xPositions.length : 0
	if (bucketMs <= 0) return null

	const bucketXByDate = new Map<string, number>()
	for (let index = 0; index < xPositions.length; index++) {
		const x = xPositions[index]
		if (!Number.isFinite(x)) continue

		const bucketDate = getBucketDateForEventSnap(index, bucketMs, startMs)
		const dateValue = getDateInputValue(bucketDate)
		if (!bucketXByDate.has(dateValue)) {
			bucketXByDate.set(dateValue, x)
		}
	}

	return bucketXByDate
}

function getDateBucketX(
	value: string,
	fallbackMs: number,
	geometry: AnalyticsChartGeometryPayload,
	startMs: number,
	endMs: number,
	bucketXByDate: Map<string, number> | null,
) {
	if (isTimeRelevantForGroupBy(props.groupBy)) {
		const clampedMs = Math.max(startMs, Math.min(endMs, fallbackMs))
		return getTimeAxisX(clampedMs, geometry, startMs, endMs)
	}

	const dateInputValue = getEventDateInputValue(value)
	if (dateInputValue && bucketXByDate) {
		const x = bucketXByDate.get(dateInputValue)
		if (x !== undefined) return x
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
		transform: `translate(-50%, 0) translate(${getClampedMarkerCenterX(
			group,
		)}px, ${MARKER_TOP_OFFSET_PX + markerOffsetTop.value}px)`,
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

function handleGroupPointerDown(groupId: string) {
	pointerDownGroupId = groupId
	wasPointerDownGroupActive = hoveredGroupId.value === groupId
	showHoveredGroup(groupId)
}

function handleGroupClick(groupId: string) {
	if (pointerDownGroupId === groupId && wasPointerDownGroupActive) {
		clearActiveGroup()
	}

	pointerDownGroupId = null
	wasPointerDownGroupActive = false
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
	() => [activeGroup.value, chartWidth.value, chartHeight.value],
	() => {
		nextTick(() => {
			updateChartRect()
			if (!tooltipElement.value) return
			tooltipWidth.value = tooltipElement.value.offsetWidth
			tooltipHeight.value = tooltipElement.value.offsetHeight
			forceCheckTooltipScrollState()
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
