import { createContext } from '@modrinth/ui'
import { useEventListener, useRafFn } from '@vueuse/core'
import {
	computed,
	nextTick,
	onScopeDispose,
	type Ref,
	ref,
	shallowReactive,
	shallowRef,
	useId,
	watch,
} from 'vue'

import type { ReviewTarget } from '~/providers/project-review/review'

import type { ProjectReviewTab } from '../layout/types'
import { useActionKeybinds } from './use-action-keybinds'

export interface ReviewAnchor {
	id: string
	target: ReviewTarget
	element: HTMLElement
	available: () => boolean
}

interface InlineReviewPanel {
	id: string
	target: () => ReviewTarget
	element: () => HTMLElement | null
	available: () => boolean
	hovered: () => boolean
	scopeHovered: () => boolean
	scopeElement: () => HTMLElement | null
	focused: () => boolean
	dropdownOpen: () => boolean
}

interface ReviewDestination {
	key: () => string | undefined
	reveal: () => Promise<void>
}

export function flashReviewElement(element: HTMLElement) {
	for (const animation of element.getAnimations()) {
		if (animation.id === 'review-reveal') animation.cancel()
	}
	const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches
	const highlightColor = 'color-mix(in srgb, var(--color-orange) 6%, transparent)'
	const animation = element.animate(
		[
			{ backgroundColor: highlightColor },
			{ backgroundColor: highlightColor },
			{
				backgroundColor: reducedMotion ? highlightColor : getComputedStyle(element).backgroundColor,
			},
		],
		{ duration: 800, easing: 'ease-out' },
	)
	animation.id = 'review-reveal'
}

export function scrollReviewElement(element: HTMLElement) {
	const panel = element.closest('.layout-panel')
	if (!panel || element === panel) return
	for (let parent = element.parentElement; parent; parent = parent.parentElement) {
		const style = getComputedStyle(parent)
		const targetBounds = element.getBoundingClientRect()
		const bounds = parent.getBoundingClientRect()
		const top = bounds.top + parent.clientTop
		const left = bounds.left + parent.clientLeft
		if (style.overflowY === 'auto' || style.overflowY === 'scroll') {
			const offset =
				targetBounds.top < top
					? targetBounds.top - top
					: Math.max(0, targetBounds.bottom - top - parent.clientHeight)
			parent.scrollTop += offset
		}
		if (style.overflowX === 'auto' || style.overflowX === 'scroll') {
			const offset =
				targetBounds.left < left
					? targetBounds.left - left
					: Math.max(0, targetBounds.right - left - parent.clientWidth)
			parent.scrollLeft += offset
		}
		if (parent === panel) break
	}
}

const HOVER_DELAY = 100
const CLOSE_DELAY = 250

function mostSpecificPanel(panels: InlineReviewPanel[]) {
	return panels.find(
		(panel) =>
			!panels.some((other) => other.id !== panel.id && panel.element()?.contains(other.element())),
	)
}

export const [injectReviewContext, provideReviewContext] =
	createContext<ReturnType<typeof createReviewContext>>('ProjectReviewActions')

export function createReviewContext(
	projectId: Ref<string | undefined>,
	isAvailable: (target: ReviewTarget) => boolean,
	keyboardFocusedElement: Ref<HTMLElement | undefined>,
) {
	const panelId = useId()
	const activeAnchor = shallowRef<ReviewAnchor>()
	const pendingAnchor = shallowRef<ReviewAnchor>()
	const panel = shallowRef<HTMLElement | null>(null)
	const pinned = ref(false)
	const revealedPanelId = ref<string>()
	const popoverHovered = ref(false)
	const heldTab = shallowRef<ProjectReviewTab>()
	const shortcutTab = shallowRef<ProjectReviewTab>()
	const inlinePanels = shallowReactive(
		new Map<string, { panel: InlineReviewPanel; visible: Ref<boolean> }>(),
	)
	const destinations = new Map<string, ReviewDestination>()
	const routes = new Map<string, () => void>()
	let revealSequence = 0
	const revealedPanelEntered = ref(false)
	let openTimer: ReturnType<typeof setTimeout> | undefined
	let closeTimer: ReturnType<typeof setTimeout> | undefined
	let hoverTimer: ReturnType<typeof setTimeout> | undefined
	let popoverLeaveTimer: ReturnType<typeof setTimeout> | undefined
	const openDropdowns = ref(0)
	const availableInlinePanels = computed(() =>
		[...inlinePanels.values()]
			.filter(
				({ panel, visible }) => visible.value && panel.available() && isAvailable(panel.target()),
			)
			.map(({ panel }) => panel),
	)
	const hoverZones = shallowRef<{ hovered?: string; scoped?: string }>({})
	watch(
		() => ({
			hovered: mostSpecificPanel(availableInlinePanels.value.filter((panel) => panel.hovered()))
				?.id,
			scoped: availableInlinePanels.value.find((panel) => panel.scopeHovered())?.id,
		}),
		(zones) => {
			clearTimeout(hoverTimer)
			hoverTimer = setTimeout(() => {
				hoverZones.value = zones
			}, HOVER_DELAY)
		},
		{ flush: 'sync' },
	)
	const activePanelId = computed(() => {
		const availablePanels = availableInlinePanels.value
		const tab = heldTab.value ?? shortcutTab.value
		if (tab) return availablePanels.find((panel) => panel.target().kind === tab)?.id
		if (activeAnchor.value && popoverHovered.value) return activeAnchor.value.id
		const hovered = availablePanels.find((panel) => panel.id === hoverZones.value.hovered)
		if (revealedPanelId.value && !pinned.value) {
			const inline = availablePanels.find((entry) => entry.id === revealedPanelId.value)
			if (
				!revealedPanelEntered.value &&
				(inline || activeAnchor.value?.id === revealedPanelId.value)
			)
				return revealedPanelId.value
			const element = inline?.element() ?? activeAnchor.value?.element
			if (
				hovered &&
				hovered.id !== revealedPanelId.value &&
				!hovered.element()?.contains(element ?? null)
			)
				return hovered.id
			const scoped = availablePanels.find((entry) => entry.id === hoverZones.value.scoped)
			if (
				scoped &&
				scoped.id !== revealedPanelId.value &&
				!inline?.scopeHovered() &&
				!scoped.element()?.contains(element ?? null)
			)
				return scoped.id
			if (inline || activeAnchor.value?.id === revealedPanelId.value) return revealedPanelId.value
		}
		if (
			activeAnchor.value &&
			(pinned.value || openDropdowns.value > 0 || hasVisibleFocus(panel.value))
		)
			return activeAnchor.value.id
		const interacting = mostSpecificPanel(
			availablePanels.filter((panel) => panel.dropdownOpen() || panel.focused()),
		)
		if (interacting) return interacting.id
		if (hovered && !hovered.element()?.contains(activeAnchor.value?.element ?? null))
			return hovered.id
		return (
			activeAnchor.value?.id ??
			hovered?.id ??
			availablePanels.find((panel) => panel.id === hoverZones.value.scoped)?.id
		)
	})
	const active = computed(() =>
		activePanelId.value === activeAnchor.value?.id ? activeAnchor.value : undefined,
	)
	const activePanelElement = computed(() => {
		const id = activePanelId.value
		if (!id) return null
		const inline = inlinePanels.get(id)
		if (inline) return inline.panel.element()
		return active.value && panel.value?.dataset.reviewPanel === id ? panel.value : null
	})
	useActionKeybinds(activePanelElement)

	function hasVisibleFocus(element: HTMLElement | null) {
		const focused = keyboardFocusedElement.value
		return !!focused && !!element?.contains(focused)
	}

	function registerInlinePanel(panel: InlineReviewPanel) {
		inlinePanels.set(panel.id, { panel, visible: ref(isElementVisible(panel.element())) })
		return () => inlinePanels.delete(panel.id)
	}

	function registerDestination(id: string, destination: ReviewDestination) {
		destinations.set(id, destination)
		return () => {
			destinations.delete(id)
		}
	}

	function registerRoute(key: string, reveal: () => void) {
		routes.set(key, reveal)
		return () => {
			routes.delete(key)
		}
	}

	async function revealPanel(key: string) {
		const sequence = ++revealSequence
		close()
		heldTab.value = undefined
		shortcutTab.value = undefined
		routes.get(key)?.()
		await nextTick()
		if (sequence !== revealSequence) return
		const destination = [...destinations.values()].find((entry) => entry.key() === key)
		await destination?.reveal()
	}

	function revealAnchor(anchor: ReviewAnchor) {
		close()
		heldTab.value = undefined
		shortcutTab.value = undefined
		if (!anchor.available() || !isAvailable(anchor.target)) return
		revealedPanelId.value = anchor.id
		revealedPanelEntered.value = anchor.element.matches(':hover')
		activeAnchor.value = anchor
	}

	function revealInlinePanel(id: string) {
		close()
		const entry = inlinePanels.get(id)
		if (!entry) return
		entry.visible.value = isElementVisible(entry.panel.element())
		revealedPanelId.value = id
		revealedPanelEntered.value = entry.panel.hovered() || entry.panel.scopeHovered()
	}

	function enter(id: string) {
		if (revealedPanelId.value === id) revealedPanelEntered.value = true
		cancelClose()
	}

	function setPopoverHovered(id: string, hovered: boolean) {
		if (activeAnchor.value?.id !== id) return
		clearTimeout(popoverLeaveTimer)
		if (hovered) {
			clearTimeout(openTimer)
			pendingAnchor.value = undefined
			popoverHovered.value = true
			enter(id)
		} else {
			popoverLeaveTimer = setTimeout(() => {
				if (activeAnchor.value?.id === id) popoverHovered.value = false
			}, HOVER_DELAY)
			leave(id)
		}
	}

	function cancelClose() {
		clearTimeout(closeTimer)
	}

	function setDropdownOpen(id: string, open: boolean) {
		if (active.value?.id !== id) return
		openDropdowns.value = Math.max(0, openDropdowns.value + (open ? 1 : -1))
		if (open) cancelClose()
		else leave(id)
	}

	function contains(target: Node) {
		return panel.value?.contains(target) ?? false
	}

	function close() {
		cancelClose()
		clearTimeout(openTimer)
		clearTimeout(popoverLeaveTimer)
		pendingAnchor.value = undefined
		activeAnchor.value = undefined
		popoverHovered.value = false
		revealedPanelId.value = undefined
		revealedPanelEntered.value = false
		openDropdowns.value = 0
		pinned.value = false
	}

	function open(anchor: ReviewAnchor) {
		if (heldTab.value || shortcutTab.value) return
		if (revealedPanelId.value && !revealedPanelEntered.value && anchor.id !== revealedPanelId.value)
			return
		if (pendingAnchor.value?.id === anchor.id) return
		clearTimeout(openTimer)
		pendingAnchor.value = undefined
		const show = () => {
			pendingAnchor.value = undefined
			if (
				heldTab.value ||
				shortcutTab.value ||
				pinned.value ||
				popoverHovered.value ||
				openDropdowns.value > 0
			)
				return
			if (!isAnchorVisible(anchor) || !anchor.available() || !isAvailable(anchor.target)) return
			cancelClose()
			if (active.value?.id !== anchor.id) {
				if (revealedPanelId.value) close()
				openDropdowns.value = 0
				pinned.value = false
			}
			activeAnchor.value = anchor
		}
		pendingAnchor.value = anchor
		openTimer = setTimeout(show, 100)
	}

	function leave(id: string) {
		if (pendingAnchor.value?.id === id) {
			clearTimeout(openTimer)
			pendingAnchor.value = undefined
		}
		if (revealedPanelId.value === id && !revealedPanelEntered.value) return
		const inline = inlinePanels.get(id)?.panel
		if (inline && revealedPanelId.value === id) {
			cancelClose()
			closeTimer = setTimeout(() => {
				if (revealedPanelId.value !== id) return
				if (inline.hovered() || inline.scopeHovered() || inline.dropdownOpen()) return
				close()
			}, CLOSE_DELAY)
			return
		}
		if (active.value?.id !== id) return
		cancelClose()
		closeTimer = setTimeout(() => {
			if (active.value?.id !== id) return
			if (pinned.value || openDropdowns.value > 0) return
			const preserveFocus = revealedPanelId.value !== id
			if (
				active.value.element.matches(':hover') ||
				(preserveFocus && hasVisibleFocus(active.value.element)) ||
				panel.value?.matches(':hover') ||
				(preserveFocus && hasVisibleFocus(panel.value))
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
		if (active.value?.id === id) close()
	}

	function isElementVisible(element: HTMLElement | null) {
		if (!element?.isConnected || element.closest('[inert], [hidden]')) return false
		if (element.getClientRects().length === 0) return false
		const { visibility } = getComputedStyle(element)
		return visibility !== 'hidden' && visibility !== 'collapse'
	}

	function isAnchorVisible(anchor: ReviewAnchor) {
		return isElementVisible(anchor.element)
	}

	const { pause, resume } = useRafFn(
		() => {
			for (const { panel, visible } of inlinePanels.values()) {
				visible.value = isElementVisible(panel.element())
			}
			if (active.value && !isAnchorVisible(active.value)) close()
			if (pendingAnchor.value && !isAnchorVisible(pendingAnchor.value)) {
				clearTimeout(openTimer)
				pendingAnchor.value = undefined
			}
		},
		{ immediate: false },
	)
	watch(
		[activeAnchor, pendingAnchor, () => inlinePanels.size],
		([anchor, pending, inlineCount]) => {
			if (anchor || pending || inlineCount) resume()
			else pause()
		},
		{ flush: 'sync' },
	)
	watch(
		[activeAnchor, activePanelId],
		([anchor, id]) => {
			if (anchor && anchor.id !== id) close()
		},
		{ flush: 'sync' },
	)
	watch(
		() => heldTab.value ?? shortcutTab.value,
		(tab) => {
			if (tab) close()
		},
		{ flush: 'sync' },
	)
	watch(
		() => {
			const id = revealedPanelId.value
			const inline = id ? inlinePanels.get(id)?.panel : undefined
			return {
				id,
				hovered: !!inline && (inline.hovered() || inline.scopeHovered()),
				dropdownOpen: !!inline?.dropdownOpen(),
			}
		},
		(current, previous) => {
			if (current.id && current.hovered) enter(current.id)
			else if (current.dropdownOpen) cancelClose()
			else if (
				current.id &&
				current.id === previous.id &&
				(previous.hovered || previous.dropdownOpen)
			)
				leave(current.id)
		},
		{ flush: 'sync' },
	)
	watch(
		[revealedPanelId, activePanelId],
		([revealed, active]) => {
			if (revealed && revealed !== active) close()
		},
		{ flush: 'post' },
	)
	useEventListener(
		'pointermove',
		() => {
			if (!heldTab.value) shortcutTab.value = undefined
		},
		{ capture: true },
	)
	useEventListener(
		'pointerdown',
		(event) => {
			shortcutTab.value = undefined
			const inline = revealedPanelId.value
				? inlinePanels.get(revealedPanelId.value)?.panel
				: undefined
			if (
				inline &&
				event.target instanceof Node &&
				!inline.element()?.contains(event.target) &&
				!inline.scopeElement()?.contains(event.target) &&
				!contains(event.target)
			)
				close()
		},
		{ capture: true },
	)

	watch(
		projectId,
		() => {
			close()
			heldTab.value = undefined
			shortcutTab.value = undefined
		},
		{ flush: 'sync' },
	)
	watch(
		() => active.value && isAvailable(active.value.target),
		(available) => {
			if (active.value && !available) release(active.value.id)
		},
	)

	onScopeDispose(() => {
		clearTimeout(hoverTimer)
		close()
	})
	return {
		active,
		activePanelId,
		keyboardFocusedElement,
		panel,
		panelId,
		pinned,
		heldTab,
		shortcutTab,
		isAvailable,
		open,
		close,
		release,
		leave,
		enter,
		setPopoverHovered,
		cancelClose,
		setDropdownOpen,
		contains,
		registerInlinePanel,
		registerDestination,
		registerRoute,
		revealPanel,
		revealAnchor,
		revealInlinePanel,
	}
}
