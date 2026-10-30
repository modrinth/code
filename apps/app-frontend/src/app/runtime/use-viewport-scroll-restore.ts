import { onScopeDispose } from 'vue'
import { useRouter } from 'vue-router'

const RESTORE_TIMEOUT_MS = 2000
const CANCEL_EVENTS = ['wheel', 'touchstart', 'mousedown', 'keydown'] as const

function getViewport() {
	return document.querySelector<HTMLElement>('.app-viewport')
}

/**
 * Manages `.app-viewport` scrolling on navigation. Pages scroll inside the viewport rather than the
 * window, so vue-router's saved positions are always zero.
 *
 * - Routes with `meta.saveScroll` get their offset restored on back/forward navigation.
 * - Routes with `meta.manualScroll` handle their own scroll position.
 * - Every other navigation scrolls to the top.
 *
 * The route `<Suspense>` keeps showing the previous page until the next one resolves, so scrolling
 * waits for `onRouteResolved`. Navigations that reuse the current page component never suspend, so
 * those scroll on the next frame instead.
 */
export function useViewportScrollRestore() {
	const router = useRouter()
	const positions = new Map<string, number>()
	let isHistoryNavigation = false
	let routePending = false
	let pendingTop: number | null = null
	let activeScroll = 0

	const cancelScroll = () => {
		activeScroll++
	}

	/**
	 * A window `popstate` listener added here runs after vue-router's own, by which point the
	 * navigation has already finished. History listeners run inside vue-router's handler, before it.
	 */
	const removeHistoryListener = router.options.history.listen(() => {
		isHistoryNavigation = true
	})

	const removeBeforeEach = router.beforeEach((to, from) => {
		if (to.path === from.path) return
		cancelScroll()
		pendingTop = null
		if (from.meta.saveScroll) positions.set(from.fullPath, getViewport()?.scrollTop ?? 0)
	})

	const removeAfterEach = router.afterEach((to, from, failure) => {
		const historyNavigation = isHistoryNavigation
		isHistoryNavigation = false
		if (failure || to.path === from.path || to.meta.manualScroll) return
		pendingTop = historyNavigation && to.meta.saveScroll ? (positions.get(to.fullPath) ?? 0) : 0
		requestAnimationFrame(() => {
			if (!routePending) flushPendingScroll()
		})
	})

	function onRoutePending() {
		routePending = true
	}

	function onRouteResolved() {
		routePending = false
		flushPendingScroll()
	}

	function flushPendingScroll() {
		if (pendingTop === null) return
		const top = pendingTop
		pendingTop = null
		restoreScroll(top)
	}

	/**
	 * Retries each frame until the offset lands, since content can keep loading after the page
	 * renders. Stops on timeout, user interaction, or another navigation.
	 */
	function restoreScroll(top: number) {
		const scrollId = ++activeScroll
		const deadline = performance.now() + RESTORE_TIMEOUT_MS
		CANCEL_EVENTS.forEach((event) =>
			window.addEventListener(event, cancelScroll, { capture: true, passive: true }),
		)

		const finish = () =>
			CANCEL_EVENTS.forEach((event) => window.removeEventListener(event, cancelScroll, true))

		const attempt = () => {
			if (scrollId !== activeScroll || performance.now() > deadline) return finish()
			const viewport = getViewport()
			if (viewport) {
				viewport.scrollTo(0, top)
				if (viewport.scrollTop >= top - 1) return finish()
			}
			requestAnimationFrame(attempt)
		}
		requestAnimationFrame(attempt)
	}

	onScopeDispose(() => {
		removeHistoryListener()
		removeBeforeEach()
		removeAfterEach()
		cancelScroll()
		pendingTop = null
	})

	return { onRoutePending, onRouteResolved }
}
