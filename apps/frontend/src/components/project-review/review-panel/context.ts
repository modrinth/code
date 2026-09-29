import { createContext } from '@modrinth/ui'
import { useEventListener, useFocusWithin, useRafFn } from '@vueuse/core'
import {
	computed,
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
	focused: () => boolean
	dropdownOpen: () => boolean
}

const CLOSE_DELAY = 350

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
) {
	const panelId = useId()
	const activeAnchor = shallowRef<ReviewAnchor>()
	const pendingAnchor = shallowRef<ReviewAnchor>()
	const panel = shallowRef<HTMLElement | null>(null)
	const { focused: panelFocused } = useFocusWithin(panel)
	const childPanels = new Set<HTMLElement>()
	const pinned = ref(false)
	const heldTab = shallowRef<ProjectReviewTab>()
	const shortcutTab = shallowRef<ProjectReviewTab>()
	const inlinePanels = shallowReactive(
		new Map<string, { panel: InlineReviewPanel; visible: Ref<boolean> }>(),
	)
	let openTimer: ReturnType<typeof setTimeout> | undefined
	let closeTimer: ReturnType<typeof setTimeout> | undefined
	const openDropdowns = ref(0)
	const activePanelId = computed(() => {
		const availablePanels = [...inlinePanels.values()]
			.filter(
				({ panel, visible }) => visible.value && panel.available() && isAvailable(panel.target()),
			)
			.map(({ panel }) => panel)
		const tab = heldTab.value ?? shortcutTab.value
		if (tab) return availablePanels.find((panel) => panel.target().kind === tab)?.id
		if (activeAnchor.value && (pinned.value || openDropdowns.value > 0 || panelFocused.value))
			return activeAnchor.value.id
		const interacting = mostSpecificPanel(
			availablePanels.filter((panel) => panel.dropdownOpen() || panel.focused()),
		)
		if (interacting) return interacting.id
		const hovered = mostSpecificPanel(availablePanels.filter((panel) => panel.hovered()))
		if (hovered && !hovered.element()?.contains(activeAnchor.value?.element ?? null))
			return hovered.id
		return (
			activeAnchor.value?.id ??
			hovered?.id ??
			availablePanels.find((panel) => panel.scopeHovered())?.id
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

	function registerInlinePanel(panel: InlineReviewPanel) {
		inlinePanels.set(panel.id, { panel, visible: ref(isElementVisible(panel.element())) })
		return () => inlinePanels.delete(panel.id)
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

	function close() {
		cancelClose()
		clearTimeout(openTimer)
		pendingAnchor.value = undefined
		activeAnchor.value = undefined
		openDropdowns.value = 0
		pinned.value = false
	}

	function open(anchor: ReviewAnchor) {
		if (heldTab.value || shortcutTab.value) return
		if (pendingAnchor.value?.id === anchor.id) return
		clearTimeout(openTimer)
		pendingAnchor.value = undefined
		const show = () => {
			pendingAnchor.value = undefined
			if (heldTab.value || shortcutTab.value || pinned.value || openDropdowns.value > 0) return
			if (!isAnchorVisible(anchor) || !anchor.available() || !isAvailable(anchor.target)) return
			cancelClose()
			if (active.value?.id !== anchor.id) {
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
		if (active.value?.id !== id) return
		cancelClose()
		closeTimer = setTimeout(() => {
			if (active.value?.id !== id) return
			if (pinned.value || openDropdowns.value > 0) return
			if (
				active.value.element.matches(':hover') ||
				active.value.element.contains(document.activeElement) ||
				panel.value?.matches(':hover') ||
				panel.value?.contains(document.activeElement) ||
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
	useEventListener(
		'pointermove',
		() => {
			if (!heldTab.value) shortcutTab.value = undefined
		},
		{ capture: true },
	)
	useEventListener(
		'pointerdown',
		() => {
			shortcutTab.value = undefined
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

	onScopeDispose(close)
	return {
		active,
		activePanelId,
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
		cancelClose,
		setDropdownOpen,
		contains,
		registerChildPanel,
		registerInlinePanel,
	}
}
