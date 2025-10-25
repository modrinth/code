import { nextTick, onUnmounted, type Ref, ref, watchEffect } from 'vue'

import { useDebugLogger } from './debug-logger'

export interface ScrollIndicatorOptions {
	watchContent?: boolean
	debounceMs?: number
	tolerance?: number
	debug?: boolean
}

export interface ScrollIndicator {
	showTopFade: Ref<boolean>
	showBottomFade: Ref<boolean>
	checkScrollState: () => void
	forceCheck: () => void
}

export function useScrollIndicator(
	containerRef: Ref<HTMLElement | null>,
	options: ScrollIndicatorOptions = {},
): ScrollIndicator {
	const { watchContent = true, debounceMs = 10, tolerance = 1, debug = false } = options

	const showTopFade = ref(false)
	const showBottomFade = ref(false)

	let resizeObserver: ResizeObserver | null = null
	let mutationObserver: MutationObserver | null = null
	let rafId: number | null = null
	let debounceTimer: number | null = null

	const log = useDebugLogger('ScrollIndicator')

	const checkScrollStateInternal = () => {
		const container = containerRef.value
		if (!container) {
			showTopFade.value = false
			showBottomFade.value = false
			if (debug) log('Container not found, hiding fades')
			return
		}

		if (rafId) {
			cancelAnimationFrame(rafId)
		}

		rafId = requestAnimationFrame(() => {
			const { scrollTop, scrollHeight, clientHeight } = container
			const isScrollable = scrollHeight > clientHeight + tolerance

			if (debug) {
				log('Checking scroll state', {
					scrollTop,
					scrollHeight,
					clientHeight,
					isScrollable,
				})
			}

			if (!isScrollable) {
				showTopFade.value = false
				showBottomFade.value = false
				if (debug) log('Content fits, no fades needed')
			} else {
				showTopFade.value = scrollTop > tolerance
				showBottomFade.value = scrollTop < scrollHeight - clientHeight - tolerance

				if (debug) {
					log('Fades updated', {
						showTop: showTopFade.value,
						showBottom: showBottomFade.value,
					})
				}
			}
		})
	}

	const checkScrollState = () => {
		if (debounceTimer) {
			clearTimeout(debounceTimer)
		}

		debounceTimer = window.setTimeout(() => {
			checkScrollStateInternal()
		}, debounceMs)
	}

	const forceCheck = () => {
		if (debounceTimer) {
			clearTimeout(debounceTimer)
			debounceTimer = null
		}
		checkScrollStateInternal()
	}

	watchEffect((onCleanup) => {
		const container = containerRef.value
		if (!container) {
			if (debug) log('No container, skipping setup')
			return
		}

		if (debug) log('Setting up observers for container', container)

		nextTick(() => {
			forceCheck()
		})

		resizeObserver = new ResizeObserver(() => {
			if (debug) log('ResizeObserver triggered')
			checkScrollState()
		})
		resizeObserver.observe(container)

		if (watchContent) {
			mutationObserver = new MutationObserver(() => {
				if (debug) log('MutationObserver triggered')
				checkScrollState()
			})

			mutationObserver.observe(container, {
				childList: true,
				subtree: true,
				characterData: true,
				attributes: false,
			})
		}

		const handleScroll = () => {
			if (debug) log('Scroll event triggered')
			checkScrollState()
		}
		container.addEventListener('scroll', handleScroll, { passive: true })

		const handleResize = () => {
			if (debug) log('Window resize triggered')
			checkScrollState()
		}
		window.addEventListener('resize', handleResize, { passive: true })

		onCleanup(() => {
			if (debug) log('Cleaning up observers and listeners')

			if (debounceTimer) {
				clearTimeout(debounceTimer)
				debounceTimer = null
			}

			if (rafId) {
				cancelAnimationFrame(rafId)
				rafId = null
			}

			resizeObserver?.disconnect()
			resizeObserver = null

			mutationObserver?.disconnect()
			mutationObserver = null

			container.removeEventListener('scroll', handleScroll)
			window.removeEventListener('resize', handleResize)
		})
	})

	onUnmounted(() => {
		if (debounceTimer) {
			clearTimeout(debounceTimer)
		}
		if (rafId) {
			cancelAnimationFrame(rafId)
		}
	})

	return {
		showTopFade,
		showBottomFade,
		checkScrollState,
		forceCheck,
	}
}
