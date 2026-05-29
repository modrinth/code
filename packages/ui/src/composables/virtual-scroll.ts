import type { Ref } from 'vue'
import { computed, ref, watch, watchEffect } from 'vue'

export interface ScrollViewportOptions {
	onScroll?: () => void
	onResize?: () => void
}

export interface VirtualScrollOptions {
	itemHeight: number
	bufferSize?: number
	initialItemCount?: number
	enabled?: Ref<boolean>
	onNearEnd?: () => void
	nearEndThreshold?: number
}

export function findScrollableAncestor(element: HTMLElement | null): HTMLElement | Window {
	if (!element) return window

	let current: HTMLElement | null = element.parentElement
	while (current) {
		const { overflowY } = getComputedStyle(current)
		if (overflowY === 'auto' || overflowY === 'scroll') {
			return current
		}
		current = current.parentElement
	}
	return window
}

export function getScrollTop(container: HTMLElement | Window): number {
	return container instanceof Window ? window.scrollY : container.scrollTop
}

export function getViewportHeight(container: HTMLElement | Window): number {
	return container instanceof Window ? window.innerHeight : container.clientHeight
}

export function useScrollViewport(options: ScrollViewportOptions = {}) {
	const listContainer = ref<HTMLElement | null>(null)
	const scrollContainer = ref<HTMLElement | Window | null>(null)
	const scrollTop = ref(0)
	const viewportHeight = ref(0)
	const containerOffset = ref(0)
	const relativeScrollTop = computed(() => Math.max(0, scrollTop.value - containerOffset.value))

	function updateContainerOffset() {
		const listEl = listContainer.value
		const container = scrollContainer.value
		if (!listEl || !container) return

		if (container instanceof Window) {
			containerOffset.value = listEl.getBoundingClientRect().top + window.scrollY
		} else {
			const listRect = listEl.getBoundingClientRect()
			const containerRect = container.getBoundingClientRect()
			containerOffset.value = listRect.top - containerRect.top + container.scrollTop
		}
	}

	function syncScrollState() {
		if (!scrollContainer.value) return
		scrollTop.value = getScrollTop(scrollContainer.value)
		viewportHeight.value = getViewportHeight(scrollContainer.value)
		updateContainerOffset()
	}

	function handleScroll() {
		if (scrollContainer.value) {
			scrollTop.value = getScrollTop(scrollContainer.value)
			updateContainerOffset()
		}

		options.onScroll?.()
	}

	function handleResize() {
		syncScrollState()
		options.onResize?.()
	}

	watchEffect((onCleanup) => {
		if (typeof window === 'undefined') return

		const listEl = listContainer.value
		if (!listEl) return

		const container = findScrollableAncestor(listEl)
		scrollContainer.value = container
		syncScrollState()

		container.addEventListener('scroll', handleScroll, { passive: true })
		window.addEventListener('resize', handleResize, { passive: true })

		let resizeObserver: ResizeObserver | undefined
		if (!(container instanceof Window)) {
			resizeObserver = new ResizeObserver(() => {
				syncScrollState()
			})
			resizeObserver.observe(container)
		}

		onCleanup(() => {
			container.removeEventListener('scroll', handleScroll)
			window.removeEventListener('resize', handleResize)
			resizeObserver?.disconnect()
		})
	})

	return {
		containerOffset,
		listContainer,
		relativeScrollTop,
		scrollContainer,
		scrollTop,
		syncScrollState,
		updateContainerOffset,
		viewportHeight,
	}
}

export function useVirtualScroll<T>(items: Ref<T[]>, options: VirtualScrollOptions) {
	const {
		itemHeight,
		bufferSize = 5,
		initialItemCount = 20,
		enabled,
		onNearEnd,
		nearEndThreshold = 0.2,
	} = options

	const { listContainer, relativeScrollTop, scrollContainer, syncScrollState, viewportHeight } =
		useScrollViewport({
			onScroll: checkNearEnd,
		})

	const totalHeight = computed(() => items.value.length * itemHeight)

	const visibleRange = computed(() => {
		if (enabled && !enabled.value) {
			return { start: 0, end: items.value.length }
		}

		if (!listContainer.value || !scrollContainer.value) {
			return { start: 0, end: Math.min(items.value.length, initialItemCount) }
		}

		const start = Math.floor(relativeScrollTop.value / itemHeight)
		const visibleCount = Math.ceil(viewportHeight.value / itemHeight)
		const rangeSize = visibleCount + bufferSize * 2

		const rangeStart = Math.min(
			Math.max(0, start - bufferSize),
			Math.max(0, items.value.length - rangeSize),
		)
		const rangeEnd = Math.min(items.value.length, rangeStart + rangeSize)

		return {
			start: rangeStart,
			end: rangeEnd,
		}
	})

	const visibleTop = computed(() =>
		enabled && !enabled.value ? 0 : visibleRange.value.start * itemHeight,
	)

	const visibleItems = computed(() =>
		items.value.slice(visibleRange.value.start, visibleRange.value.end),
	)

	function checkNearEnd() {
		if (!onNearEnd || !listContainer.value || !viewportHeight.value) return

		const containerBottom = listContainer.value.getBoundingClientRect().bottom
		const remainingScroll = containerBottom - viewportHeight.value

		if (remainingScroll < viewportHeight.value * nearEndThreshold) {
			onNearEnd()
		}
	}

	watch(items, () => {
		syncScrollState()
	})

	return {
		listContainer,
		totalHeight,
		visibleRange,
		visibleTop,
		visibleItems,
	}
}
