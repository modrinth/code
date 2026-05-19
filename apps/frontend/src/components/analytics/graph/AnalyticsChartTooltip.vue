<template>
	<div
		v-show="visible"
		ref="tooltipElement"
		class="analytics-chart-tooltip absolute left-0 top-0 z-10 max-h-[360px] min-w-[14rem] overflow-y-auto overscroll-contain rounded-lg border border-solid border-surface-5 bg-surface-3 px-3 py-2 text-sm shadow-lg"
		:class="pinned ? '' : 'pointer-events-none'"
		:style="positionStyle"
		@click.stop
	>
		<div class="mb-1 flex items-center justify-between gap-2 font-medium text-contrast">
			<span>
				{{ rangeLabel }}
				<span v-if="durationLabel" class="text-xs font-normal text-secondary">
					({{ durationLabel }})
				</span>
			</span>
			<PinIcon
				v-if="pinned"
				v-tooltip="'Chart tooltip pinned'"
				class="pointer-events-none size-4 shrink-0 font-normal text-contrast"
				aria-label="Pinned"
			/>
		</div>
		<div v-if="!ratioMode" class="mb-1.5 flex items-center justify-between gap-4">
			<span class="font-medium text-primary">Total</span>
			<span class="font-semibold text-contrast">{{ formattedTotal }}</span>
		</div>
		<div class="flex flex-col gap-1">
			<div
				v-for="entry in entries"
				:key="entry.projectId"
				class="flex w-full items-center justify-between gap-4 text-primary"
			>
				<button
					type="button"
					class="inline-flex items-center gap-1.5 border-0 bg-transparent p-0 text-left focus-visible:!outline-none"
					:class="
						entry.toggleDisabled && !shiftKeyPressed
							? 'cursor-default'
							: entry.hidden
								? 'cursor-pointer text-secondary opacity-70'
								: 'cursor-pointer text-primary transition-all hover:brightness-125'
					"
					:aria-label="`${entry.hidden ? 'Show' : 'Hide'} ${entry.name} in graph`"
					@mouseenter="emit('entry-hover', entry.projectId)"
					@mouseleave="emit('entry-hover-clear', entry.projectId)"
					@focus="emit('entry-hover', entry.projectId)"
					@blur="emit('entry-hover-clear', entry.projectId)"
					@click="onEntryClick($event, entry)"
				>
					<span class="size-2 rounded-full" :style="{ backgroundColor: entry.color }" />
					<span
						v-tooltip="entry.projectName ?? ''"
						:class="{
							'line-through': entry.hidden,
							capitalize: capitalizeLabels,
						}"
					>
						{{ entry.name }}
					</span>
				</button>
				<span
					class="font-semibold"
					:class="entry.hidden ? 'text-secondary line-through opacity-70' : 'text-contrast'"
				>
					{{ entry.formattedValue }}
				</span>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { PinIcon } from '@modrinth/assets'

export type AnalyticsChartTooltipEntry = {
	projectId: string
	name: string
	projectName?: string
	color: string
	formattedValue: string
	hidden: boolean
	toggleDisabled: boolean
}

const props = defineProps<{
	visible: boolean
	x: number
	y: number
	start: Date | null
	end: Date | null
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

function onEntryClick(event: MouseEvent, entry: AnalyticsChartTooltipEntry) {
	if (entry.toggleDisabled && !event.shiftKey) return
	emit('entry-click', entry.projectId, event.shiftKey)
}

const ONE_DAY_MS = 24 * 60 * 60 * 1000
const ONE_HOUR_MS = 60 * 60 * 1000
const ONE_MINUTE_MS = 60 * 1000

function getDayPeriod(date: Date, options: Intl.DateTimeFormatOptions): string | null {
	return (
		new Intl.DateTimeFormat(undefined, options)
			.formatToParts(date)
			.find((part) => part.type === 'dayPeriod')?.value ?? null
	)
}

function formatDateWithoutDayPeriod(date: Date, options: Intl.DateTimeFormatOptions): string {
	const parts = new Intl.DateTimeFormat(undefined, options).formatToParts(date)
	const dayPeriodIndex = parts.findIndex((part) => part.type === 'dayPeriod')
	if (dayPeriodIndex === -1) return parts.map((part) => part.value).join('')

	const previousPart = parts[dayPeriodIndex - 1]
	const nextPart = parts[dayPeriodIndex + 1]

	parts.splice(dayPeriodIndex, 1)

	if (previousPart?.type === 'literal') {
		parts[dayPeriodIndex - 1] = {
			...previousPart,
			value: previousPart.value.replace(/\s+$/, ''),
		}
	} else if (nextPart?.type === 'literal') {
		parts[dayPeriodIndex] = {
			...nextPart,
			value: nextPart.value.replace(/^\s+/, ''),
		}
	}

	return parts.map((part) => part.value).join('')
}

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
	const showTrailingYear = !yearsDiffer && chartYearsDiffer
	const monthsDiffer = yearsDiffer || start.getMonth() !== end.getMonth()
	const daysDiffer = monthsDiffer || start.getDate() !== end.getDate()

	const timeOptions: Intl.DateTimeFormatOptions = includeTime
		? { hour: 'numeric', minute: '2-digit' }
		: {}

	const startOptions: Intl.DateTimeFormatOptions = {
		month: 'short',
		day: 'numeric',
		...(yearsDiffer ? { year: 'numeric' } : {}),
		...timeOptions,
	}

	let endOptions: Intl.DateTimeFormatOptions
	if (!daysDiffer && includeTime) {
		endOptions = { ...timeOptions }
	} else if (yearsDiffer) {
		endOptions = { month: 'short', day: 'numeric', year: 'numeric', ...timeOptions }
	} else if (monthsDiffer || includeTime) {
		endOptions = { month: 'short', day: 'numeric', ...timeOptions }
	} else {
		endOptions = { day: 'numeric' }
	}

	const startDayPeriod = includeTime ? getDayPeriod(start, timeOptions) : null
	const endDayPeriod = includeTime ? getDayPeriod(end, timeOptions) : null
	const showEndDayPeriodOnly = startDayPeriod !== null && startDayPeriod === endDayPeriod
	const startLabel = showEndDayPeriodOnly
		? formatDateWithoutDayPeriod(start, startOptions)
		: new Intl.DateTimeFormat(undefined, startOptions).format(start)
	const endLabel = new Intl.DateTimeFormat(undefined, endOptions).format(end)
	const range = `${startLabel}–${endLabel}`

	if (!showTrailingYear) return range

	const yearLabel = new Intl.DateTimeFormat(undefined, { year: 'numeric' }).format(end)
	return `${range}, ${yearLabel}`
}

function formatDurationLabel(start: Date, end: Date): string {
	const durationMs = end.getTime() - start.getTime()
	if (!Number.isFinite(durationMs) || durationMs <= 0) return ''

	if (durationMs >= ONE_DAY_MS) {
		const days = Math.round(durationMs / ONE_DAY_MS)
		return `${days} ${days === 1 ? 'day' : 'days'}`
	}
	if (durationMs >= ONE_HOUR_MS) {
		const hours = Math.round(durationMs / ONE_HOUR_MS)
		return `${hours} ${hours === 1 ? 'hour' : 'hours'}`
	}
	const minutes = Math.max(1, Math.round(durationMs / ONE_MINUTE_MS))
	return `${minutes} ${minutes === 1 ? 'minute' : 'minutes'}`
}

const rangeLabel = computed(() =>
	props.start && props.end
		? formatRangeLabel(props.start, props.end, props.chartStart, props.chartEnd)
		: '',
)

const durationLabel = computed(() =>
	props.start && props.end ? formatDurationLabel(props.start, props.end) : '',
)

const tooltipElement = ref<HTMLDivElement | null>(null)
const tooltipWidth = ref(0)
const tooltipHeight = ref(0)

const CURSOR_OFFSET = 12
const EDGE_PADDING = 8
const WHEEL_DELTA_LINE = 1
const WHEEL_DELTA_PAGE = 2
const WHEEL_LINE_HEIGHT = 16

watch(
	() => [props.visible, props.entries, rangeLabel.value, durationLabel.value, props.pinned],
	() => {
		nextTick(() => {
			if (!tooltipElement.value) return
			tooltipWidth.value = tooltipElement.value.offsetWidth
			tooltipHeight.value = tooltipElement.value.offsetHeight
		})
	},
	{ deep: true, immediate: true },
)

function getNormalizedWheelDeltaY(event: WheelEvent, element: HTMLElement) {
	if (event.deltaMode === WHEEL_DELTA_PAGE) return event.deltaY * element.clientHeight
	if (event.deltaMode === WHEEL_DELTA_LINE) return event.deltaY * WHEEL_LINE_HEIGHT
	return event.deltaY
}

function consumeWheel(event: WheelEvent): boolean {
	const element = tooltipElement.value
	if (!props.visible || !element) return false

	const maxScrollTop = element.scrollHeight - element.clientHeight
	if (maxScrollTop <= 0) return false

	const deltaY = getNormalizedWheelDeltaY(event, element)
	if (deltaY === 0) return false

	const scrollTop = element.scrollTop
	element.scrollTop = Math.min(maxScrollTop, Math.max(0, scrollTop + deltaY))
	event.preventDefault()
	return true
}

defineExpose({
	consumeWheel,
})

const positionStyle = computed(() => {
	const desiredLeft = props.x + CURSOR_OFFSET
	const maxLeft = Math.max(EDGE_PADDING, props.containerWidth - tooltipWidth.value - EDGE_PADDING)
	const clampedLeft =
		desiredLeft + tooltipWidth.value > props.containerWidth - EDGE_PADDING
			? Math.max(EDGE_PADDING, props.x - tooltipWidth.value - CURSOR_OFFSET)
			: Math.min(maxLeft, desiredLeft)

	const desiredTop = props.y - tooltipHeight.value / 2
	const maxTop = Math.max(EDGE_PADDING, props.containerHeight - tooltipHeight.value - EDGE_PADDING)
	const clampedTop = Math.min(maxTop, Math.max(EDGE_PADDING, desiredTop))

	return {
		transform: `translate3d(${clampedLeft}px, ${clampedTop}px, 0)`,
	}
})
</script>

<style scoped>
.analytics-chart-tooltip {
	transition: transform 750ms cubic-bezier(0.22, 1, 0.36, 1);
	will-change: transform;
}
</style>
