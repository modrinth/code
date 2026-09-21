import { createContext } from '@modrinth/ui'
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
	const panel = shallowRef<HTMLElement | null>(null)
	const childPanels = new Set<HTMLElement>()
	const pinned = ref(false)
	let openTimer: ReturnType<typeof setTimeout> | undefined
	let closeTimer: ReturnType<typeof setTimeout> | undefined
	let pendingAnchor: ReviewAnchor | undefined

	function cancelClose() {
		clearTimeout(closeTimer)
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
		pendingAnchor = undefined
		active.value = undefined
		pinned.value = false
		if (restoreFocus) {
			void nextTick(() => {
				if (trigger?.isConnected) trigger.focus()
			})
		}
	}

	function open(anchor: ReviewAnchor, explicit = false) {
		clearTimeout(openTimer)
		pendingAnchor = undefined
		const show = () => {
			pendingAnchor = undefined
			if (!anchor.element.isConnected || !anchor.available() || !isAvailable(anchor.target)) return
			cancelClose()
			active.value = anchor
			pinned.value = explicit
		}
		if (explicit) show()
		else {
			pendingAnchor = anchor
			openTimer = setTimeout(show, 100)
		}
	}

	function leave(id: string) {
		if (pendingAnchor?.id === id) {
			clearTimeout(openTimer)
			pendingAnchor = undefined
		}
		if (active.value?.id !== id) return
		cancelClose()
		closeTimer = setTimeout(() => {
			if (active.value?.id !== id) return
			if (
				active.value.element.matches(':hover') ||
				panel.value?.matches(':hover') ||
				[...childPanels].some(
					(child) => child.matches(':hover') || child.contains(document.activeElement),
				)
			)
				return
			close()
		}, CLOSE_DELAY)
	}

	function release(id: string) {
		if (pendingAnchor?.id === id) {
			clearTimeout(openTimer)
			pendingAnchor = undefined
		}
		if (active.value?.id === id) close(!!panel.value?.contains(document.activeElement))
	}

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
		contains,
		registerChildPanel,
	}
}
