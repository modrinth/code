import { computed, onUnmounted, ref, watch, type ComputedRef, type Ref } from 'vue'

const LOADING_INDICATOR_DELAY_MS = 200
const LOADING_INDICATOR_MIN_MS = 250

type MaybeReadonlyRef<T> = Ref<T> | ComputedRef<T>

export function useSkinPreviewLoading(isReady: MaybeReadonlyRef<boolean>) {
	const showLoading = ref(false)
	const isPreviewVisible = computed(() => isReady.value && !showLoading.value)
	let loadingIndicatorDelayTimer: number | null = null
	let loadingIndicatorMinTimer: number | null = null
	let loadingIndicatorShownAt = 0

	function clearLoadingIndicatorDelayTimer() {
		if (loadingIndicatorDelayTimer !== null) {
			clearTimeout(loadingIndicatorDelayTimer)
			loadingIndicatorDelayTimer = null
		}
	}

	function clearLoadingIndicatorMinTimer() {
		if (loadingIndicatorMinTimer !== null) {
			clearTimeout(loadingIndicatorMinTimer)
			loadingIndicatorMinTimer = null
		}
	}

	function hideLoadingIndicatorAfterMinimum() {
		const visibleFor = Date.now() - loadingIndicatorShownAt
		const remaining = LOADING_INDICATOR_MIN_MS - visibleFor

		if (remaining <= 0) {
			showLoading.value = false
			return
		}

		loadingIndicatorMinTimer = window.setTimeout(() => {
			showLoading.value = false
			loadingIndicatorMinTimer = null
		}, remaining)
	}

	watch(
		() => isReady.value,
		(ready) => {
			clearLoadingIndicatorDelayTimer()

			if (ready) {
				if (showLoading.value) {
					clearLoadingIndicatorMinTimer()
					hideLoadingIndicatorAfterMinimum()
				}

				return
			}

			clearLoadingIndicatorMinTimer()

			if (showLoading.value || typeof window === 'undefined') {
				return
			}

			loadingIndicatorDelayTimer = window.setTimeout(() => {
				loadingIndicatorDelayTimer = null

				if (isReady.value) {
					return
				}

				showLoading.value = true
				loadingIndicatorShownAt = Date.now()
			}, LOADING_INDICATOR_DELAY_MS)
		},
		{ immediate: true },
	)

	onUnmounted(() => {
		clearLoadingIndicatorDelayTimer()
		clearLoadingIndicatorMinTimer()
	})

	return {
		isPreviewVisible,
		showLoading,
	}
}
