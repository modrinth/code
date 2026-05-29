import { computed, type ComputedRef, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'

export function useAnalyticsChartLayout(showEmptyChartState: ComputedRef<boolean>) {
	const graphSection = ref<HTMLElement | null>(null)
	const rememberedGraphSectionHeight = ref(0)
	const graphSectionStyle = computed(() =>
		showEmptyChartState.value && rememberedGraphSectionHeight.value > 0
			? { height: `${rememberedGraphSectionHeight.value}px` }
			: undefined,
	)
	let graphSectionResizeObserver: ResizeObserver | null = null

	function rememberGraphSectionHeight() {
		if (!graphSection.value) return

		const height = graphSection.value.getBoundingClientRect().height
		if (height > 0) {
			rememberedGraphSectionHeight.value = height
		}
	}

	onMounted(() => {
		if (graphSection.value && typeof ResizeObserver !== 'undefined') {
			graphSectionResizeObserver = new ResizeObserver(() => {
				if (showEmptyChartState.value) return
				rememberGraphSectionHeight()
			})
			graphSectionResizeObserver.observe(graphSection.value)
		}
	})

	onBeforeUnmount(() => {
		graphSectionResizeObserver?.disconnect()
		graphSectionResizeObserver = null
	})

	watch(showEmptyChartState, (showEmpty) => {
		if (showEmpty) {
			rememberGraphSectionHeight()
		} else {
			nextTick(rememberGraphSectionHeight)
		}
	})

	return {
		graphSection,
		graphSectionStyle,
		rememberGraphSectionHeight,
	}
}
