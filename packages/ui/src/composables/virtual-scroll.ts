import type { Ref } from 'vue'
import { computed, ref, watchEffect } from 'vue'

export interface VirtualScrollOptions {
	itemHeight: number
	bufferSize?: number
	enabled?: Ref<boolean>
	onNearEnd?: () => void
	nearEndThreshold?: number
}

export function useVirtualScroll<T>(items: Ref<T[]>, options: VirtualScrollOptions) {
	const { itemHeight, bufferSize = 5, enabled, onNearEnd, nearEndThreshold = 0.2 } = options

	const listContainer = ref<HTMLElement | null>(null)
	const scrollContainer = ref<HTMLElement | Window | null>(null)
	const scrollTop = ref(0)
	const viewportHeight = ref(0)

	const totalHeight = computed(() => items.value.length * itemHeight)

	function findScrollableAncestor(element: HTMLElement | null): HTMLElement | Window {
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

	function getScrollTop(container: HTMLElement | Window): number {
		return container instanceof Window ? window.scrollY : container.scrollTop
	}

	function getViewportHeight(container: HTMLElement | Window): number {
		return container instanceof Window ? window.innerHeight : container.clientHeight
	}

	function getContainerOffset(listEl: HTMLElement, container: HTMLElement | Window): number {
		if (container instanceof Window) {
			return listEl.getBoundingClientRect().top + window.scrollY
		}
		const listRect = listEl.getBoundingClientRect()
		const containerRect = container.getBoundingClientRect()
		return listRect.top - containerRect.top + container.scrollTop
	}

	const visibleRange = computed(() => {
		if (enabled && !enabled.value) {
			return { start: 0, end: items.value.length }
		}

		if (!listContainer.value || !scrollContainer.value) return { start: 0, end: 0 }

		const containerOffset = getContainerOffset(listContainer.value, scrollContainer.value)
		const relativeScrollTop = Math.max(0, scrollTop.value - containerOffset)

		const start = Math.floor(relativeScrollTop / itemHeight)
		const visibleCount = Math.ceil(viewportHeight.value / itemHeight)

		return {
			start: Math.max(0, start - bufferSize),
			end: Math.min(items.value.length, start + visibleCount + bufferSize * 2),
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

	function handleScroll() {
		if (scrollContainer.value) {
			scrollTop.value = getScrollTop(scrollContainer.value)
		}
		checkNearEnd()
	}

	function handleResize() {
		if (scrollContainer.value) {
			viewportHeight.value = getViewportHeight(scrollContainer.value)
		}
	}

	watchEffect((onCleanup) => {
		if (typeof window === 'undefined') return

		const listEl = listContainer.value
		if (!listEl) return

		const container = findScrollableAncestor(listEl)
		scrollContainer.value = container
		viewportHeight.value = getViewportHeight(container)
		scrollTop.value = getScrollTop(container)

		container.addEventListener('scroll', handleScroll, { passive: true })
		window.addEventListener('resize', handleResize, { passive: true })

		onCleanup(() => {
			container.removeEventListener('scroll', handleScroll)
			window.removeEventListener('resize', handleResize)
		})
	})

	return {
		listContainer,
		totalHeight,
		visibleRange,
		visibleTop,
		visibleItems,
	}
}
