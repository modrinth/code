import type { Ref } from 'vue'
import { onUnmounted, ref, watch } from 'vue'

/**
 * Observes when a target element becomes "stuck" (i.e. its natural position has scrolled out of view).
 * Injects a zero-height sentinel element before the target and uses IntersectionObserver to detect
 * when the sentinel leaves the viewport.
 */
export function useStickyObserver(target: Ref<HTMLElement | null | undefined>) {
	const isStuck = ref(false)
	let sentinel: HTMLElement | null = null
	let observer: IntersectionObserver | null = null

	watch(
		target,
		(el) => {
			observer?.disconnect()
			sentinel?.remove()
			observer = null
			sentinel = null

			if (el) {
				sentinel = document.createElement('div')
				sentinel.style.height = '0'
				sentinel.setAttribute('aria-hidden', 'true')
				el.parentElement?.insertBefore(sentinel, el)

				observer = new IntersectionObserver(
					([entry]) => {
						isStuck.value = !entry.isIntersecting
					},
					{ threshold: 0, rootMargin: '-1px 0px 0px 0px' },
				)
				observer.observe(sentinel)
			}
		},
		{ flush: 'post' },
	)

	onUnmounted(() => {
		observer?.disconnect()
		sentinel?.remove()
	})

	return { isStuck }
}
