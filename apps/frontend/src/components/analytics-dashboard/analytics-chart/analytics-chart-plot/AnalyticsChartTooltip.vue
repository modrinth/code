<template>
	<div
		v-show="visible"
		ref="tooltipElement"
		class="analytics-chart-tooltip absolute left-0 top-0 z-10 flex max-h-[356px] flex-col overflow-hidden rounded-lg border border-solid border-surface-5 bg-surface-3 py-2 text-sm shadow-lg"
		:class="pinned ? '' : 'pointer-events-none'"
		:style="positionStyle"
		@wheel.stop
		@click.stop
	>
		<div
			class="mb-1.5 flex shrink-0 items-start justify-between gap-2 border-0 border-b border-solid border-surface-5 px-3 pb-1.5 font-medium text-contrast"
		>
			<div class="flex min-w-0 flex-col gap-0.5">
				<span class="min-w-0 truncate">
					{{ rangeLabel }}
					<span v-if="durationLabel" class="text-xs font-normal text-secondary">
						({{ durationLabel }})
					</span>
				</span>
				<span v-if="previousRangeLabel" class="min-w-0 space-x-1 truncate text-xs text-primary">
					<span class="font-medium">{{ previousRangeLabel }}</span>
					<span class="font-normal text-secondary">
						{{ formatMessage(analyticsChartMessages.previousPeriodShort) }}
					</span>
				</span>
			</div>
			<PinIcon
				v-if="pinned"
				v-tooltip="formatMessage(analyticsChartMessages.tooltipPinned)"
				class="pointer-events-none size-4 shrink-0 font-normal text-contrast"
				:aria-label="formatMessage(analyticsChartMessages.pinned)"
			/>
		</div>
		<Transition
			enter-active-class="transition-all duration-200 ease-out"
			enter-from-class="opacity-0 max-h-0"
			enter-to-class="opacity-100 max-h-6"
			leave-active-class="transition-all duration-200 ease-in"
			leave-from-class="opacity-100 max-h-6"
			leave-to-class="opacity-0 max-h-0"
		>
			<div
				v-if="showEntriesTopFade"
				class="analytics-chart-tooltip-entries-fade-top pointer-events-none absolute left-0 right-0 z-10 -mt-1 h-6 bg-gradient-to-b from-surface-3 to-transparent"
			/>
		</Transition>

		<div
			ref="entriesElement"
			class="analytics-chart-tooltip-entries flex min-h-0 flex-col overflow-y-auto overscroll-contain px-3"
			@scroll="checkEntriesScrollState"
			@touchstart="onEntriesTouchStart"
			@touchmove="onEntriesTouchMove"
			@touchend="clearEntriesTouchScroll"
			@touchcancel="clearEntriesTouchScroll"
		>
			<div v-if="!ratioMode" class="flex shrink-0 items-center justify-between gap-4">
				<span class="font-medium text-primary">
					{{ formatMessage(analyticsChartMessages.total) }}
				</span>
				<span class="font-semibold text-contrast">{{ formattedTotal }}</span>
			</div>
			<div
				v-for="entry in entries"
				:key="entry.projectId"
				class="flex w-full min-w-0 items-center justify-between gap-4 text-primary"
			>
				<button
					type="button"
					class="inline-flex min-w-0 items-center gap-1.5 border-0 bg-transparent p-0 py-0.5 text-left focus-visible:!outline-none"
					:class="
						entry.toggleDisabled && !shiftKeyPressed
							? 'cursor-default'
							: entry.hidden
								? 'cursor-pointer text-secondary opacity-70'
								: 'cursor-pointer text-primary transition-all hover:brightness-125'
					"
					:aria-label="getEntryAriaLabel(entry)"
					@mouseenter="emit('entry-hover', entry.projectId)"
					@mouseleave="emit('entry-hover-clear', entry.projectId)"
					@focus="emit('entry-hover', entry.projectId)"
					@blur="emit('entry-hover-clear', entry.projectId)"
					@click="onEntryClick($event, entry)"
				>
					<span
						:class="
							entry.isPreviousPeriod
								? 'h-0 w-2 rounded-none border-0 border-t-2 border-dashed bg-transparent'
								: 'size-2 rounded-full'
						"
						class="shrink-0"
						:style="
							entry.isPreviousPeriod
								? { borderColor: entry.color }
								: { backgroundColor: entry.color }
						"
					/>
					<span
						v-tooltip="entry.tooltip ?? entry.projectName ?? ''"
						class="min-w-0 truncate"
						:class="{
							'line-through': entry.hidden,
							capitalize: capitalizeLabels,
						}"
					>
						{{ entry.name }}
					</span>
				</button>
				<span
					:class="[
						'shrink-0',
						entry.isPreviousPeriod ? 'font-medium text-secondary' : 'font-semibold',
						entry.hidden ? 'text-primary line-through opacity-70' : 'text-contrast',
					]"
				>
					{{ entry.formattedValue }}
				</span>
			</div>
		</div>

		<Transition
			enter-active-class="transition-all duration-200 ease-out"
			enter-from-class="opacity-0 max-h-0"
			enter-to-class="opacity-100 max-h-6"
			leave-active-class="transition-all duration-200 ease-in"
			leave-from-class="opacity-100 max-h-6"
			leave-to-class="opacity-0 max-h-0"
		>
			<div
				v-if="showEntriesBottomFade"
				class="analytics-chart-tooltip-entries-fade-bottom pointer-events-none absolute left-0 right-0 z-10 -mb-1 h-6 bg-gradient-to-t from-surface-3 to-transparent"
			/>
		</Transition>
	</div>
</template>

<script setup lang="ts">
import { PinIcon } from '@modrinth/assets'
import { useScrollIndicator, useVIntl } from '@modrinth/ui'

import { analyticsChartMessages } from '../../analytics-messages'

export type AnalyticsChartTooltipEntry = {
	projectId: string
	name: string
	projectName?: string
	tooltip?: string
	color: string
	formattedValue: string
	hidden: boolean
	toggleDisabled: boolean
	isPreviousPeriod?: boolean
}

const props = defineProps<{
	visible: boolean
	x: number
	y: number
	start: Date | null
	end: Date | null
	previousStart: Date | null
	previousEnd: Date | null
	chartStart: Date | null
	chartEnd: Date | null
	formattedTotal: string
	entries: AnalyticsChartTooltipEntry[]
	containerWidth: number
	containerHeight: number
	pinned: boolean
	ratioMode: boolean
	capitalizeLabels: boolean
	shiftKeyPressed: boolean
}>()

const emit = defineEmits<{
	'entry-click': [projectId: string, shiftKey: boolean]
	'entry-hover': [projectId: string]
	'entry-hover-clear': [projectId: string]
}>()

const { formatMessage } = useVIntl()

function onEntryClick(event: MouseEvent, entry: AnalyticsChartTooltipEntry) {
	if (entry.toggleDisabled && !event.shiftKey) return
	emit('entry-click', entry.projectId, event.shiftKey)
}

function getEntryAriaLabel(entry: AnalyticsChartTooltipEntry) {
	return formatMessage(
		entry.hidden
			? analyticsChartMessages.showEntryInGraph
			: analyticsChartMessages.hideEntryInGraph,
		{ name: entry.name },
	)
}

const ONE_DAY_MS = 24 * 60 * 60 * 1000
const ONE_HOUR_MS = 60 * 60 * 1000
const ONE_MINUTE_MS = 60 * 1000
const DATE_LOCALE = 'en-US'

function formatRangeLabel(
	start: Date,
	end: Date,
	chartStart: Date | null,
	chartEnd: Date | null,
): string {
	const includeTime = end.getTime() - start.getTime() < ONE_DAY_MS
	const yearsDiffer = start.getFullYear() !== end.getFullYear()
	const chartYearsDiffer =
		chartStart !== null && chartEnd !== null && chartStart.getFullYear() !== chartEnd.getFullYear()
	const rangeYearDiffersFromChart =
		chartStart !== null && start.getFullYear() !== chartStart.getFullYear()
	const showTrailingYear = !yearsDiffer && (chartYearsDiffer || rangeYearDiffersFromChart)
	const monthsDiffer = yearsDiffer || start.getMonth() !== end.getMonth()

	const timeOptions: Intl.DateTimeFormatOptions = includeTime
		? { hour: 'numeric', minute: '2-digit', hour12: true }
		: {}

	const startOptions: Intl.DateTimeFormatOptions = {
		month: 'short',
		day: 'numeric',
		...(yearsDiffer ? { year: 'numeric' } : {}),
		...timeOptions,
	}

	if (includeTime) {
		const startLabel = new Intl.DateTimeFormat(DATE_LOCALE, startOptions).format(start)
		const endLabel = new Intl.DateTimeFormat(DATE_LOCALE, timeOptions).format(end)
		const range = `${startLabel}–${endLabel}`

		if (!showTrailingYear) return range

		const yearLabel = new Intl.DateTimeFormat(DATE_LOCALE, { year: 'numeric' }).format(end)
		return `${range}, ${yearLabel}`
	}

	let endOptions: Intl.DateTimeFormatOptions
	if (yearsDiffer) {
		endOptions = { month: 'short', day: 'numeric', year: 'numeric' }
	} else if (monthsDiffer) {
		endOptions = { month: 'short', day: 'numeric' }
	} else {
		endOptions = { day: 'numeric' }
	}

	const startLabel = new Intl.DateTimeFormat(DATE_LOCALE, startOptions).format(start)
	const endLabel = new Intl.DateTimeFormat(DATE_LOCALE, endOptions).format(end)
	const range = `${startLabel}–${endLabel}`

	if (!showTrailingYear) return range

	const yearLabel = new Intl.DateTimeFormat(DATE_LOCALE, { year: 'numeric' }).format(end)
	return `${range}, ${yearLabel}`
}

function formatDurationLabel(start: Date, end: Date): string {
	const durationMs = end.getTime() - start.getTime()
	if (!Number.isFinite(durationMs) || durationMs <= 0) return ''

	if (durationMs >= ONE_DAY_MS) {
		const days = Math.round(durationMs / ONE_DAY_MS)
		return formatMessage(analyticsChartMessages.durationDays, { count: days })
	}
	if (durationMs >= ONE_HOUR_MS) {
		const hours = Math.round(durationMs / ONE_HOUR_MS)
		return formatMessage(analyticsChartMessages.durationHours, { count: hours })
	}
	const minutes = Math.max(1, Math.round(durationMs / ONE_MINUTE_MS))
	return formatMessage(analyticsChartMessages.durationMinutes, { count: minutes })
}

const rangeLabel = computed(() =>
	props.start && props.end
		? formatRangeLabel(props.start, props.end, props.chartStart, props.chartEnd)
		: '',
)

const durationLabel = computed(() =>
	props.start && props.end ? formatDurationLabel(props.start, props.end) : '',
)
const previousRangeLabel = computed(() =>
	props.previousStart && props.previousEnd
		? formatRangeLabel(props.previousStart, props.previousEnd, props.chartStart, props.chartEnd)
		: '',
)

const tooltipElement = ref<HTMLDivElement | null>(null)
const entriesElement = ref<HTMLDivElement | null>(null)
const {
	showTopFade: showEntriesTopFade,
	showBottomFade: showEntriesBottomFade,
	checkScrollState: checkEntriesScrollState,
	forceCheck: forceCheckEntriesScrollState,
} = useScrollIndicator(entriesElement)
const tooltipWidth = ref(0)
const tooltipHeight = ref(0)
const entriesTopOffset = ref(0)
const entriesBottomOffset = ref(0)
const tooltipOffsetParentLeft = ref(0)
const viewportWidth = ref(0)
let entriesTouchStartY = 0
let entriesTouchStartScrollTop = 0

const CURSOR_OFFSET = 12
const EDGE_PADDING = 8
const TOOLTIP_MAX_WIDTH = 26 * 16
const WHEEL_DELTA_LINE = 1
const WHEEL_DELTA_PAGE = 2
const WHEEL_LINE_HEIGHT = 16

function getTooltipFallbackWidth() {
	const availableWidth = (viewportWidth.value || props.containerWidth) - EDGE_PADDING * 2
	if (availableWidth <= 0) return TOOLTIP_MAX_WIDTH
	return Math.min(TOOLTIP_MAX_WIDTH, availableWidth)
}

function updateTooltipMeasurements() {
	nextTick(() => {
		const element = tooltipElement.value
		if (!element) return

		tooltipWidth.value = element.offsetWidth
		tooltipHeight.value = element.offsetHeight

		const entries = entriesElement.value
		if (entries) {
			entriesTopOffset.value = entries.offsetTop
			entriesBottomOffset.value = Math.max(
				0,
				element.offsetHeight - entries.offsetTop - entries.offsetHeight,
			)
		}

		const offsetParent =
			element.offsetParent instanceof HTMLElement ? element.offsetParent : element.parentElement
		tooltipOffsetParentLeft.value = offsetParent?.getBoundingClientRect().left ?? 0
		viewportWidth.value =
			document.documentElement.clientWidth || window.innerWidth || props.containerWidth
		forceCheckEntriesScrollState()
	})
}

watch(
	() => [
		props.visible,
		props.entries,
		rangeLabel.value,
		durationLabel.value,
		previousRangeLabel.value,
		props.pinned,
		props.containerWidth,
		props.containerHeight,
	],
	updateTooltipMeasurements,
	{ deep: true, immediate: true },
)

onMounted(() => {
	updateTooltipMeasurements()
	window.addEventListener('resize', updateTooltipMeasurements)
})

onBeforeUnmount(() => {
	window.removeEventListener('resize', updateTooltipMeasurements)
})

function getNormalizedWheelDeltaY(event: WheelEvent, element: HTMLElement) {
	if (event.deltaMode === WHEEL_DELTA_PAGE) return event.deltaY * element.clientHeight
	if (event.deltaMode === WHEEL_DELTA_LINE) return event.deltaY * WHEEL_LINE_HEIGHT
	return event.deltaY
}

function getMaxScrollTop(element: HTMLElement) {
	return Math.max(0, element.scrollHeight - element.clientHeight)
}

function consumeWheel(event: WheelEvent): boolean {
	const element = entriesElement.value
	if (!props.visible || !element) return false

	const maxScrollTop = getMaxScrollTop(element)
	if (maxScrollTop <= 0) return false

	const deltaY = getNormalizedWheelDeltaY(event, element)
	if (deltaY === 0) return false

	const scrollTop = element.scrollTop
	element.scrollTop = Math.min(maxScrollTop, Math.max(0, scrollTop + deltaY))
	event.preventDefault()
	return true
}

function onEntriesTouchStart(event: TouchEvent) {
	const element = entriesElement.value
	const touch = event.touches[0]
	if (!element || !touch) return

	entriesTouchStartY = touch.clientY
	entriesTouchStartScrollTop = element.scrollTop
}

function onEntriesTouchMove(event: TouchEvent) {
	const element = entriesElement.value
	const touch = event.touches[0]
	if (!props.visible || !element || !touch) return

	const maxScrollTop = getMaxScrollTop(element)
	if (maxScrollTop <= 0) return

	const nextScrollTop = Math.min(
		maxScrollTop,
		Math.max(0, entriesTouchStartScrollTop + entriesTouchStartY - touch.clientY),
	)
	element.scrollTop = nextScrollTop
	event.preventDefault()
	event.stopPropagation()
}

function clearEntriesTouchScroll() {
	entriesTouchStartY = 0
	entriesTouchStartScrollTop = 0
}

defineExpose({
	consumeWheel,
})

const positionStyle = computed(() => {
	const tooltipMaxWidth = getTooltipFallbackWidth()
	const tooltipWidthForPosition = tooltipWidth.value || tooltipMaxWidth
	const desiredLeft = props.x + CURSOR_OFFSET
	const viewportRight = viewportWidth.value || tooltipOffsetParentLeft.value + props.containerWidth
	const desiredViewportRight = tooltipOffsetParentLeft.value + desiredLeft + tooltipWidthForPosition
	const shouldPlaceLeft =
		props.x <= props.containerWidth / 4 || desiredViewportRight > viewportRight - EDGE_PADDING
	const candidateLeft = shouldPlaceLeft
		? props.x - tooltipWidthForPosition - CURSOR_OFFSET
		: desiredLeft
	const minLeft = EDGE_PADDING - tooltipOffsetParentLeft.value
	const maxLeft = Math.max(
		minLeft,
		viewportRight - tooltipOffsetParentLeft.value - tooltipWidthForPosition - EDGE_PADDING,
	)
	const clampedLeft = Math.min(maxLeft, Math.max(minLeft, candidateLeft))

	const desiredTop = props.y - tooltipHeight.value / 2
	const maxTop = Math.max(EDGE_PADDING, props.containerHeight - tooltipHeight.value - EDGE_PADDING)
	const clampedTop = Math.min(maxTop, Math.max(EDGE_PADDING, desiredTop))

	return {
		'--analytics-chart-tooltip-max-width': `${tooltipMaxWidth}px`,
		'--analytics-chart-tooltip-entries-top': `${entriesTopOffset.value}px`,
		'--analytics-chart-tooltip-entries-bottom': `${entriesBottomOffset.value}px`,
		transform: `translate3d(${clampedLeft}px, ${clampedTop}px, 0)`,
	}
})
</script>

<style scoped>
.analytics-chart-tooltip {
	min-width: min(14rem, var(--analytics-chart-tooltip-max-width, calc(100vw - 1rem)));
	max-width: var(--analytics-chart-tooltip-max-width, min(26rem, calc(100vw - 1rem)));
	transition: transform 750ms cubic-bezier(0.22, 1, 0.36, 1);
	will-change: transform;
}

.analytics-chart-tooltip-entries {
	-webkit-overflow-scrolling: touch;
	overscroll-behavior: contain;
	touch-action: pan-y;
}

.analytics-chart-tooltip-entries-fade-top {
	top: var(--analytics-chart-tooltip-entries-top, 0rem);
}

.analytics-chart-tooltip-entries-fade-bottom {
	bottom: var(--analytics-chart-tooltip-entries-bottom, 0rem);
}

@media (pointer: coarse) {
	.analytics-chart-tooltip {
		pointer-events: auto;
	}
}
</style>
