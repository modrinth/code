import type { Labrinth } from '@modrinth/api-client'
import { computed, type ComputedRef, onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue'

import type { AnalyticsGroupByPreset } from '~/providers/analytics/analytics'

import {
	ensureMinimumTimeRange,
	getDefaultAnalyticsGroupByForDurationMinutes,
} from '../../query-builder/timeframe.ts'
import type {
	AnalyticsChartHoverState,
	AnalyticsChartRangeBounds,
} from '../analytics-chart-types.ts'
import type { ChartDataset } from '../analytics-chart-utils.ts'
import { getSliceBucketRange } from '../analytics-chart-utils.ts'
import type {
	AnalyticsChartGeometryPayload,
	AnalyticsChartRangeSelectPayload,
} from '../AnalyticsChart.client.vue'
import type AnalyticsChartTooltip from './AnalyticsChartTooltip.vue'

export function useAnalyticsChartInteractions({
	isDataLoading,
	fetchRequest,
	sliceCount,
	chartLabels,
	allChartDatasets,
	chartRangeBounds,
	shouldShowPreviousPeriod,
	onRangeSelected,
}: {
	isDataLoading: ComputedRef<boolean>
	fetchRequest: ComputedRef<Labrinth.Analytics.v3.FetchRequest | null>
	sliceCount: ComputedRef<number>
	chartLabels: ComputedRef<string[]>
	allChartDatasets: ComputedRef<ChartDataset[]>
	chartRangeBounds: ComputedRef<AnalyticsChartRangeBounds | null>
	shouldShowPreviousPeriod: ComputedRef<boolean>
	onRangeSelected: (start: Date, end: Date, groupBy: AnalyticsGroupByPreset) => void
}) {
	const chartContainer = ref<HTMLElement | null>(null)
	const chartTooltip = ref<InstanceType<typeof AnalyticsChartTooltip> | null>(null)
	const chartGeometry = ref<AnalyticsChartGeometryPayload | null>(null)
	const containerSize = reactive({ width: 0, height: 0 })
	const hoverState = reactive<AnalyticsChartHoverState>({
		visible: false,
		x: 0,
		y: 0,
		sliceIndex: null,
	})
	const isHoverPinned = ref(false)
	const ignoreNextChartClick = ref(false)
	const isShiftKeyPressed = ref(false)
	let resizeObserver: ResizeObserver | null = null
	let clearIgnoredChartClickTimeout: ReturnType<typeof setTimeout> | null = null

	function setHoverState(payload: AnalyticsChartHoverState) {
		hoverState.visible = payload.visible
		hoverState.x = payload.x
		hoverState.y = payload.y
		hoverState.sliceIndex = payload.sliceIndex
	}

	function clearHoverState() {
		hoverState.visible = false
		hoverState.sliceIndex = null
	}

	function unpinHoverState() {
		isHoverPinned.value = false
		clearHoverState()
	}

	function updateShiftKeyState(event: KeyboardEvent) {
		isShiftKeyPressed.value = event.shiftKey
	}

	function clearShiftKeyState() {
		isShiftKeyPressed.value = false
	}

	function onDocumentClick(event: MouseEvent) {
		if (!isHoverPinned.value) return
		if (event.target instanceof Node && chartContainer.value?.contains(event.target)) return
		unpinHoverState()
	}

	function onChartHover(payload: AnalyticsChartHoverState) {
		if (isDataLoading.value) return
		if (isHoverPinned.value) return
		setHoverState(payload)
	}

	function ignoreUpcomingChartClick() {
		ignoreNextChartClick.value = true
		if (clearIgnoredChartClickTimeout) {
			clearTimeout(clearIgnoredChartClickTimeout)
		}
		clearIgnoredChartClickTimeout = setTimeout(() => {
			ignoreNextChartClick.value = false
			clearIgnoredChartClickTimeout = null
		}, 350)
	}

	function onPinnedDrag(payload: AnalyticsChartHoverState) {
		if (isDataLoading.value || !isHoverPinned.value) return
		ignoreUpcomingChartClick()
		setHoverState(payload)
	}

	function onTouchDragEnd() {
		ignoreUpcomingChartClick()
		if (!hoverState.visible || hoverState.sliceIndex === null) return
		isHoverPinned.value = true
	}

	function onChartGeometry(payload: AnalyticsChartGeometryPayload) {
		chartGeometry.value = payload
	}

	function getDefaultGroupByForRange(start: Date, end: Date) {
		const ensuredRange = ensureMinimumTimeRange(start, end)
		const durationMinutes = Math.max(
			1,
			Math.floor((ensuredRange.end.getTime() - ensuredRange.start.getTime()) / 60000),
		)

		return getDefaultAnalyticsGroupByForDurationMinutes(durationMinutes)
	}

	function onRangeSelect(payload: AnalyticsChartRangeSelectPayload) {
		if (isDataLoading.value) return

		const nextFetchRequest = fetchRequest.value
		if (!nextFetchRequest) return

		if (payload.startSliceIndex === payload.endSliceIndex) {
			ignoreUpcomingChartClick()
			return
		}

		const startSliceIndex = Math.min(payload.startSliceIndex, payload.endSliceIndex)
		const endSliceIndex = Math.max(payload.startSliceIndex, payload.endSliceIndex)
		const startBucketRange = getSliceBucketRange(
			nextFetchRequest.time_range,
			sliceCount.value,
			startSliceIndex,
		)
		const endBucketRange = getSliceBucketRange(
			nextFetchRequest.time_range,
			sliceCount.value,
			endSliceIndex,
		)
		const start = startBucketRange.start
		const end = endBucketRange.end

		if (
			!Number.isFinite(start.getTime()) ||
			!Number.isFinite(end.getTime()) ||
			end.getTime() <= start.getTime()
		) {
			return
		}

		ignoreUpcomingChartClick()
		unpinHoverState()
		onRangeSelected(start, end, getDefaultGroupByForRange(start, end))
	}

	function onChartClick() {
		if (isDataLoading.value) return
		if (ignoreNextChartClick.value) {
			ignoreNextChartClick.value = false
			return
		}

		if (!hoverState.visible || hoverState.sliceIndex === null) {
			if (isHoverPinned.value) {
				unpinHoverState()
			}
			return
		}

		if (isHoverPinned.value) {
			unpinHoverState()
			return
		}

		isHoverPinned.value = true
	}

	function onChartWheel(event: WheelEvent) {
		if (isAnalyticsEventTooltipTrigger(event.target)) return
		if (!hoverState.visible) return
		chartTooltip.value?.consumeWheel(event)
	}

	function isAnalyticsEventTooltipTrigger(target: EventTarget | null) {
		return (
			target instanceof Element && target.closest('[data-analytics-event-tooltip-trigger]') !== null
		)
	}

	const pinnedSliceIndex = computed(() => (isHoverPinned.value ? hoverState.sliceIndex : null))
	const showHoverGuide = computed(
		() =>
			!isDataLoading.value &&
			!isHoverPinned.value &&
			hoverState.visible &&
			hoverState.sliceIndex !== null,
	)
	const showPinnedGuide = computed(
		() =>
			!isDataLoading.value &&
			isHoverPinned.value &&
			hoverState.visible &&
			hoverState.sliceIndex !== null,
	)
	const hoverBucketRange = computed(() => {
		const nextFetchRequest = fetchRequest.value
		if (!nextFetchRequest || hoverState.sliceIndex === null) return null
		return getSliceBucketRange(nextFetchRequest.time_range, sliceCount.value, hoverState.sliceIndex)
	})
	const previousHoverBucketRange = computed(() => {
		if (!shouldShowPreviousPeriod.value) return null

		const bucketRange = hoverBucketRange.value
		const rangeBounds = chartRangeBounds.value
		if (!bucketRange || !rangeBounds) return null

		const periodMs = rangeBounds.end.getTime() - rangeBounds.start.getTime()
		if (!Number.isFinite(periodMs) || periodMs <= 0) return null

		return {
			start: new Date(bucketRange.start.getTime() - periodMs),
			end: new Date(bucketRange.end.getTime() - periodMs),
		}
	})

	onMounted(() => {
		if (chartContainer.value && typeof ResizeObserver !== 'undefined') {
			resizeObserver = new ResizeObserver((entries) => {
				const entry = entries[0]
				if (!entry) return
				containerSize.width = entry.contentRect.width
				containerSize.height = entry.contentRect.height
			})
			resizeObserver.observe(chartContainer.value)
		}

		window.addEventListener('keydown', updateShiftKeyState)
		window.addEventListener('keyup', updateShiftKeyState)
		window.addEventListener('blur', clearShiftKeyState)
		document.addEventListener('click', onDocumentClick, true)
	})

	onBeforeUnmount(() => {
		resizeObserver?.disconnect()
		resizeObserver = null
		window.removeEventListener('keydown', updateShiftKeyState)
		window.removeEventListener('keyup', updateShiftKeyState)
		window.removeEventListener('blur', clearShiftKeyState)
		document.removeEventListener('click', onDocumentClick, true)
		if (clearIgnoredChartClickTimeout) {
			clearTimeout(clearIgnoredChartClickTimeout)
			clearIgnoredChartClickTimeout = null
		}
	})

	watch([chartLabels, allChartDatasets], () => {
		isHoverPinned.value = false
		clearHoverState()
	})

	watch(isDataLoading, (loading) => {
		if (!loading) return
		isHoverPinned.value = false
		clearHoverState()
	})

	return {
		chartContainer,
		chartTooltip,
		chartGeometry,
		containerSize,
		hoverState,
		isHoverPinned,
		isShiftKeyPressed,
		setHoverState,
		clearHoverState,
		unpinHoverState,
		onChartHover,
		onPinnedDrag,
		onTouchDragEnd,
		onChartGeometry,
		onRangeSelect,
		onChartClick,
		onChartWheel,
		pinnedSliceIndex,
		showHoverGuide,
		showPinnedGuide,
		hoverBucketRange,
		previousHoverBucketRange,
	}
}
