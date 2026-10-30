import { moderationSettings } from '@modrinth/moderation'
import { onScopeDispose } from 'vue'

import { useModerationSettings } from '~/composables/moderation'

export function useFastScroll() {
	const settings = useModerationSettings()
	let active: { element: HTMLElement; top: number; left: number; cancel: () => void } | undefined
	onScopeDispose(() => active?.cancel())

	return (element: HTMLElement, top: number, left = element.scrollLeft) => {
		const targetTop = Math.max(0, Math.min(element.scrollHeight - element.clientHeight, top))
		const targetLeft = Math.max(0, Math.min(element.scrollWidth - element.clientWidth, left))
		if (
			Math.abs(targetTop - element.scrollTop) < 0.5 &&
			Math.abs(targetLeft - element.scrollLeft) < 0.5
		) {
			active?.cancel()
			return
		}
		const view = element.ownerDocument.defaultView
		if (
			!view ||
			settings.value.get(moderationSettings.General.TechReviewScrollTiming) === 0 ||
			view.matchMedia('(prefers-reduced-motion: reduce)').matches
		) {
			active?.cancel()
			element.scrollTo({ top: targetTop, left: targetLeft, behavior: 'instant' })
			return
		}
		if (active?.element === element) {
			active.top = targetTop
			active.left = targetLeft
			return
		}
		active?.cancel()

		let positionTop = element.scrollTop
		let positionLeft = element.scrollLeft
		let velocityTop = 0
		let velocityLeft = 0
		let previousTime = view.performance.now()
		let frame: number
		const stop = () => {
			view.cancelAnimationFrame(frame)
			for (const event of ['wheel', 'pointerdown']) element.removeEventListener(event, stop)
			active = undefined
		}
		const animation = { element, top: targetTop, left: targetLeft, cancel: stop }
		active = animation
		for (const event of ['wheel', 'pointerdown']) {
			element.addEventListener(event, stop, { passive: true })
		}

		function animate(now: number) {
			if (!element.isConnected) {
				stop()
				return
			}
			if (settings.value.get(moderationSettings.General.TechReviewScrollTiming) === 0) {
				element.scrollTo({ top: animation.top, left: animation.left, behavior: 'instant' })
				stop()
				return
			}
			const elapsed = Math.min((now - previousTime) / 1000, 1 / 30)
			previousTime = now
			const frequency = 9000 / settings.value.get(moderationSettings.General.TechReviewScrollTiming)
			const decay = Math.exp(-frequency * elapsed)
			const offsetTop = positionTop - animation.top
			const offsetLeft = positionLeft - animation.left
			const stepTop = (velocityTop + frequency * offsetTop) * elapsed
			const stepLeft = (velocityLeft + frequency * offsetLeft) * elapsed
			positionTop = animation.top + (offsetTop + stepTop) * decay
			positionLeft = animation.left + (offsetLeft + stepLeft) * decay
			velocityTop = (velocityTop - frequency * stepTop) * decay
			velocityLeft = (velocityLeft - frequency * stepLeft) * decay
			const settled =
				Math.abs(positionTop - animation.top) < 0.5 &&
				Math.abs(positionLeft - animation.left) < 0.5 &&
				Math.abs(velocityTop) < 5 &&
				Math.abs(velocityLeft) < 5
			element.scrollTo({
				top: settled ? animation.top : positionTop,
				left: settled ? animation.left : positionLeft,
				behavior: 'instant',
			})
			if (settled) stop()
			else frame = view.requestAnimationFrame(animate)
		}
		frame = view.requestAnimationFrame(animate)
	}
}
