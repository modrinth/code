import { useEventListener } from '@vueuse/core'
import type { Ref } from 'vue'

const REVEAL_SCROLL_DISTANCE = 800
const REVEAL_GESTURE_GAP_MS = 400
const BOTTOM_TOLERANCE_PX = 4

/**
 * When `true`, the default layout collapses the site footer behind a "Show footer" control.
 * Pages with tall, scroll-heavy content (e.g. the hosting files tab) enable it while mounted.
 */
export function useCollapsibleFooter() {
	return useState('collapsible-footer', () => false)
}

/**
 * Calls `onReveal` once the user keeps scrolling down past the bottom of the page with intent,
 * i.e. roughly `REVEAL_SCROLL_DISTANCE` pixels of continuous downward wheel movement.
 */
export function useFooterRevealIntent(enabled: Ref<boolean>, onReveal: () => void) {
	let accumulated = 0
	let lastWheelAt = 0

	useEventListener(
		'wheel',
		(event: WheelEvent) => {
			if (!enabled.value) return

			const atBottom =
				window.innerHeight + window.scrollY >=
				document.documentElement.scrollHeight - BOTTOM_TOLERANCE_PX
			const now = performance.now()

			if (event.deltaY <= 0 || !atBottom || now - lastWheelAt > REVEAL_GESTURE_GAP_MS) {
				accumulated = 0
			}
			lastWheelAt = now
			if (event.deltaY <= 0 || !atBottom) return

			accumulated += event.deltaY
			if (accumulated >= REVEAL_SCROLL_DISTANCE) {
				accumulated = 0
				onReveal()
			}
		},
		{ passive: true },
	)
}
