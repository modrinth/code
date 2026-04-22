<template>
	<div
		v-show="visible"
		ref="tooltipElement"
		class="analytics-chart-tooltip pointer-events-none absolute left-0 top-0 z-10 min-w-[14rem] rounded-lg border border-solid border-surface-5 bg-surface-3 px-3 py-2 text-sm shadow-lg"
		:style="positionStyle"
	>
		<div class="mb-1 font-medium text-contrast">{{ rangeLabel }}</div>
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
	rangeLabel: string
	entries: AnalyticsChartTooltipEntry[]
	containerWidth: number
	containerHeight: number
}>()

const tooltipElement = ref<HTMLDivElement | null>(null)
const tooltipWidth = ref(0)
const tooltipHeight = ref(0)

const CURSOR_OFFSET = 12
const EDGE_PADDING = 8

watch(
	() => [props.visible, props.entries, props.rangeLabel],
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
