import { type MaybeElementRef, unrefElement } from '@vueuse/core'
import { type MaybeRefOrGetter, onScopeDispose, toValue, watch } from 'vue'

export interface UseAnimatedWidthOptions {
	enabled?: MaybeRefOrGetter<boolean>
	duration?: number
	easing?: string
}

/**
 * Smoothly animates an element's width whenever its content changes, while keeping its
 * width `auto` at rest.
 */
export function useAnimatedWidth(target: MaybeElementRef, options: UseAnimatedWidthOptions = {}) {
	const { enabled = true, duration = 200, easing = 'cubic-bezier(0.4, 0, 0.2, 1)' } = options

	let settledWidth = 0
	let animation: Animation | null = null
	let disconnect: (() => void) | null = null

	function stop() {
		disconnect?.()
		disconnect = null
		animation?.cancel()
		animation = null
	}

	function animateWidth(el: HTMLElement) {
		const from = animation ? el.offsetWidth : settledWidth
		animation?.cancel()
		animation = null

		const to = el.offsetWidth
		settledWidth = to
		if (!from || from === to || window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
			return
		}

		const current = el.animate([{ width: `${from}px` }, { width: `${to}px` }], {
			duration,
			easing,
		})
		animation = current
		current.onfinish = () => {
			if (animation === current) animation = null
		}
	}

	watch(
		() => [unrefElement(target), toValue(enabled)] as const,
		([target, isEnabled]) => {
			stop()
			if (!isEnabled || !target) return

			const el = target as HTMLElement
			settledWidth = el.offsetWidth

			const mutationObserver = new MutationObserver(() => animateWidth(el))
			mutationObserver.observe(el, { childList: true, characterData: true, subtree: true })

			const resizeObserver = new ResizeObserver(() => {
				if (!animation) settledWidth = el.offsetWidth
			})
			resizeObserver.observe(el)

			disconnect = () => {
				mutationObserver.disconnect()
				resizeObserver.disconnect()
			}
		},
		{ immediate: true, flush: 'post' },
	)

	onScopeDispose(stop)
}
