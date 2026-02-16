import type { Ref } from 'vue'
import { onUnmounted, ref, watch } from 'vue'

import { useDebugLogger } from './debug-logger'

/**
 * Observes when a target element becomes "stuck" (i.e. its natural position has scrolled out of view).
 * Injects a zero-height sentinel element before the target and uses IntersectionObserver to detect
 * when the sentinel leaves the viewport.
 */
export function useStickyObserver(target: Ref<HTMLElement | null | undefined>, label?: string) {
	const debug = useDebugLogger(`sticky-observer${label ? `:${label}` : ''}`)
	const isStuck = ref(false)
	let sentinel: HTMLElement | null = null
	let observer: IntersectionObserver | null = null

	debug('init, target value:', target.value)

	watch(
		target,
		(el, oldEl) => {
			debug('watch fired, el:', el, 'oldEl:', oldEl)
			observer?.disconnect()
			sentinel?.remove()
			observer = null
			sentinel = null

			if (el) {
				debug(
					'setting up sentinel, parent:',
					el.parentElement,
					'parentClasses:',
					el.parentElement?.className,
				)
				debug('el classes:', el.className)
				debug('el computed overflow:', getComputedStyle(el).overflow)
				debug(
					'parent computed overflow:',
					el.parentElement ? getComputedStyle(el.parentElement).overflow : 'no parent',
				)

				sentinel = document.createElement('div')
				sentinel.style.height = '0'
				const parentGap = getComputedStyle(el.parentElement!).gap
				sentinel.style.marginBottom = parentGap ? `-${parentGap}` : '0'
				sentinel.setAttribute('aria-hidden', 'true')
				el.parentElement?.insertBefore(sentinel, el)

				debug('sentinel inserted, sentinel parent:', sentinel.parentElement?.className)

				observer = new IntersectionObserver(
					([entry]) => {
						const wasStuck = isStuck.value
						isStuck.value = !entry.isIntersecting
						if (wasStuck !== isStuck.value) {
							debug(
								'isStuck changed:',
								isStuck.value,
								'intersectionRatio:',
								entry.intersectionRatio,
								'boundingClientRect:',
								entry.boundingClientRect,
							)
						}
					},
					{ threshold: 0, rootMargin: '-1px 0px 0px 0px' },
				)
				observer.observe(sentinel)
				debug('observer started')
			} else {
				debug('el is null, no observer set up')
			}
		},
		{ flush: 'post' },
	)

	onUnmounted(() => {
		debug('unmounted, cleaning up')
		observer?.disconnect()
		sentinel?.remove()
	})

	return { isStuck }
}
