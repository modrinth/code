<template>
	<div
		v-show="visible"
		ref="tooltipElement"
		class="analytics-chart-tooltip absolute left-0 top-0 z-10 max-h-[360px] min-w-[14rem] overflow-y-auto rounded-lg border border-solid border-surface-5 bg-surface-3 px-3 py-2 text-sm shadow-lg"
		:class="pinned ? '' : 'pointer-events-none'"
		:style="positionStyle"
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
				class="pointer-events-auto size-4 shrink-0 font-normal text-contrast"
				aria-label="Pinned"
			/>
		</div>
		<div class="mb-1.5 flex items-center justify-between gap-4">
			<span class="font-medium text-primary">Total</span>
			<span class="font-semibold text-contrast">{{ formattedTotal }}</span>
		</div>
		<div class="flex flex-col gap-1">
			<div
				v-for="entry in entries"
				:key="entry.projectId"
				class="flex items-center justify-between gap-4"
			>
				<div class="inline-flex items-center gap-1.5 text-primary">
					<span class="size-2 rounded-full" :style="{ backgroundColor: entry.color }" />
					<span>{{ entry.name }}</span>
				</div>
				<span class="font-semibold text-contrast">{{ entry.formattedValue }}</span>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { PinIcon } from '@modrinth/assets'

export type AnalyticsChartTooltipEntry = {
	projectId: string
	name: string
	color: string
	formattedValue: string
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
}>()

const ONE_DAY_MS = 24 * 60 * 60 * 1000
const ONE_HOUR_MS = 60 * 60 * 1000
const ONE_MINUTE_MS = 60 * 1000

function formatRangeLabel(
	start: Date,
	end: Date,
	chartStart: Date | null,
	chartEnd: Date | null,
): string {
	const includeTime = end.getTime() - start.getTime() < ONE_DAY_MS
	const yearsDiffer = start.getFullYear() !== end.getFullYear()
	const chartYearsDiffer =
		chartStart !== null &&
		chartEnd !== null &&
		chartStart.getFullYear() !== chartEnd.getFullYear()
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

	const startLabel = new Intl.DateTimeFormat(undefined, startOptions).format(start)
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
