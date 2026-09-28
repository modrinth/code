import { createContext } from '@modrinth/ui'
import { useRafFn } from '@vueuse/core'
import { nextTick, onScopeDispose, type Ref, ref, shallowRef, useId, watch } from 'vue'

import type { ReviewTarget } from '~/providers/project-review/review'

export interface ReviewAnchor {
	id: string
	target: ReviewTarget
	element: HTMLElement
	trigger: HTMLElement | null
	available: () => boolean
}

const CLOSE_DELAY = 350

export const [injectReviewContext, provideReviewContext] =
	createContext<ReturnType<typeof createReviewContext>>('ProjectReviewActions')

export function createReviewContext(
	projectId: Ref<string | undefined>,
	isAvailable: (target: ReviewTarget) => boolean,
) {
	const panelId = useId()
	const active = shallowRef<ReviewAnchor>()
	const pendingAnchor = shallowRef<ReviewAnchor>()
	const panel = shallowRef<HTMLElement | null>(null)
	const childPanels = new Set<HTMLElement>()
	const pinned = ref(false)
	let openTimer: ReturnType<typeof setTimeout> | undefined
	let closeTimer: ReturnType<typeof setTimeout> | undefined
	let openDropdowns = 0

	function cancelClose() {
		clearTimeout(closeTimer)
	}

	function setDropdownOpen(id: string, open: boolean) {
		if (active.value?.id !== id) return
		openDropdowns = Math.max(0, openDropdowns + (open ? 1 : -1))
		if (open) cancelClose()
		else leave(id)
	}

	function contains(target: Node) {
		return panel.value?.contains(target) || [...childPanels].some((child) => child.contains(target))
	}

	function registerChildPanel(element: HTMLElement) {
		childPanels.add(element)
		cancelClose()
		return () => {
			childPanels.delete(element)
			if (active.value) leave(active.value.id)
		}
	}

	function close(restoreFocus = false) {
		const trigger = active.value?.trigger
		cancelClose()
		clearTimeout(openTimer)
		pendingAnchor.value = undefined
		active.value = undefined
		openDropdowns = 0
		pinned.value = false
		if (restoreFocus) {
			void nextTick(() => {
				if (trigger?.isConnected) trigger.focus()
			})
		}
	}

	function open(anchor: ReviewAnchor, explicit = false) {
		clearTimeout(openTimer)
		pendingAnchor.value = undefined
		const show = () => {
			pendingAnchor.value = undefined
			if ((pinned.value || openDropdowns > 0) && !explicit) return
			if (!isAnchorVisible(anchor) || !anchor.available() || !isAvailable(anchor.target)) return
			cancelClose()
			if (active.value?.id !== anchor.id) {
				openDropdowns = 0
				pinned.value = false
			}
			active.value = anchor
		}
		if (explicit) show()
		else {
			pendingAnchor.value = anchor
			openTimer = setTimeout(show, 100)
		}
	}

	function leave(id: string) {
		if (pendingAnchor.value?.id === id) {
			clearTimeout(openTimer)
			pendingAnchor.value = undefined
		}
		if (active.value?.id !== id) return
		cancelClose()
		closeTimer = setTimeout(() => {
			if (active.value?.id !== id) return
			if (pinned.value || openDropdowns > 0) return
			if (
				active.value.element.matches(':hover') ||
				panel.value?.matches(':hover') ||
				[...childPanels].some(
					(child) => child.matches(':hover') || child.contains(document.activeElement),
				)
			)
				return
			if (pendingAnchor.value && pendingAnchor.value.id !== id) {
				leave(id)
				return
			}
			close()
		}, CLOSE_DELAY)
	}

	function release(id: string) {
		if (pendingAnchor.value?.id === id) {
			clearTimeout(openTimer)
			pendingAnchor.value = undefined
		}
		if (active.value?.id === id) close(!!panel.value?.contains(document.activeElement))
	}

	function isAnchorVisible(anchor: ReviewAnchor) {
		const element = anchor.element
		if (!element.isConnected || element.closest('[inert], [hidden]')) return false
		if (element.getClientRects().length === 0) return false
		const { visibility } = getComputedStyle(element)
		return visibility !== 'hidden' && visibility !== 'collapse'
	}

	const { pause, resume } = useRafFn(
		() => {
			if (active.value && !isAnchorVisible(active.value)) close()
			if (pendingAnchor.value && !isAnchorVisible(pendingAnchor.value)) {
				clearTimeout(openTimer)
				pendingAnchor.value = undefined
			}
		},
		{ immediate: false },
	)
	watch(
		[active, pendingAnchor],
		([activeAnchor, pending]) => {
			if (activeAnchor || pending) resume()
			else pause()
		},
		{ flush: 'sync' },
	)

	watch(projectId, () => close(), { flush: 'sync' })
	watch(
		() => active.value && isAvailable(active.value.target),
		(available) => {
			if (active.value && !available) release(active.value.id)
		},
	)

	onScopeDispose(close)
	return {
		active,
		panel,
		panelId,
		pinned,
		isAvailable,
		open,
		close,
		release,
		leave,
		cancelClose,
		setDropdownOpen,
		contains,
		registerChildPanel,
	}
}
