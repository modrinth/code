import type { CSSProperties, Ref } from 'vue'
import { computed, nextTick, ref } from 'vue'

import type { ContextMenuParentAction, Point, ViewportRect } from './types'

const MENU_GAP = 8
const VIEWPORT_MARGIN = 10

type ContextMenuPositionOptions = {
	shown: Ref<boolean>
	activeOption: Ref<ContextMenuParentAction | null>
	activeOptionIndex: Ref<number | null>
	isMobileSubmenuLayout: Ref<boolean>
	contextMenu: Ref<HTMLElement | null>
	submenu: Ref<HTMLElement | null>
	optionButtonRefs: Map<number, HTMLElement>
}

export function useContextMenuPosition({
	shown,
	activeOption,
	activeOptionIndex,
	isMobileSubmenuLayout,
	contextMenu,
	submenu,
	optionButtonRefs,
}: ContextMenuPositionOptions) {
	const menuStyle = ref<CSSProperties>({ left: '0px', top: '0px' })
	const menuAnchor = ref<Point>({ x: 0, y: 0 })
	const submenuPosition = ref<Point>({ x: 0, y: 0 })
	const hasSubmenuPosition = ref(false)
	let positionRafId: number | null = null

	const submenuStyle = computed<CSSProperties>(() => {
		if (isMobileSubmenuLayout.value) return menuStyle.value

		return {
			left: `${submenuPosition.value.x}px`,
			top: `${submenuPosition.value.y}px`,
		}
	})

	function updateMenuPosition(x: number, y: number) {
		if (!contextMenu.value) return

		const viewport = getViewportRect()
		const menuRect = contextMenu.value.getBoundingClientRect()
		const viewportLeft = viewport.offsetLeft
		const viewportTop = viewport.offsetTop
		const viewportRight = viewport.offsetLeft + viewport.width
		const viewportBottom = viewport.offsetTop + viewport.height
		const anchorX = x + viewport.offsetLeft
		const anchorY = y + viewport.offsetTop
		const left = Math.min(
			Math.max(viewportLeft + VIEWPORT_MARGIN, anchorX + MENU_GAP),
			Math.max(viewportLeft + VIEWPORT_MARGIN, viewportRight - menuRect.width - VIEWPORT_MARGIN),
		)
		const top = Math.min(
			Math.max(viewportTop + VIEWPORT_MARGIN, anchorY + MENU_GAP),
			Math.max(viewportTop + VIEWPORT_MARGIN, viewportBottom - menuRect.height - VIEWPORT_MARGIN),
		)

		menuStyle.value = { left: `${left}px`, top: `${top}px` }
		if (activeOption.value) scheduleSubmenuPositionUpdate()
	}

	function updateSubmenuPosition() {
		if (!activeOption.value || activeOptionIndex.value === null) return false

		if (isMobileSubmenuLayout.value) {
			hasSubmenuPosition.value = true
			return true
		}

		const optionButton = optionButtonRefs.get(activeOptionIndex.value)
		if (!optionButton || !contextMenu.value) return false

		const viewport = getViewportRect()
		const buttonRect = optionButton.getBoundingClientRect()
		const menuRect = contextMenu.value.getBoundingClientRect()
		const submenuRect = submenu.value?.getBoundingClientRect()
		const submenuWidth = submenuRect?.width ?? menuRect.width
		const submenuHeight = submenuRect?.height ?? 100
		const direction = getSubmenuOpenDirection(menuRect, submenuWidth, viewport)
		const preferredLeft =
			direction === 'right'
				? buttonRect.right + MENU_GAP
				: buttonRect.left - submenuWidth - MENU_GAP
		const minLeft = viewport.offsetLeft + VIEWPORT_MARGIN
		const maxLeft = Math.max(
			minLeft,
			viewport.offsetLeft + viewport.width - submenuWidth - VIEWPORT_MARGIN,
		)
		const minTop = viewport.offsetTop + VIEWPORT_MARGIN
		const maxTop = Math.max(
			minTop,
			viewport.offsetTop + viewport.height - submenuHeight - VIEWPORT_MARGIN,
		)

		submenuPosition.value = {
			x: Math.min(Math.max(minLeft, preferredLeft), maxLeft),
			y: Math.min(Math.max(minTop, buttonRect.top), maxTop),
		}
		hasSubmenuPosition.value = true
		return true
	}

	function scheduleSubmenuPositionUpdate(retries = 8) {
		nextTick(() => {
			if (!shown.value || !activeOption.value) return

			const hasRenderedSubmenu = submenu.value !== null
			if (updateSubmenuPosition()) {
				if (!hasRenderedSubmenu) nextTick(updateSubmenuPosition)
				return
			}

			if (retries > 0) setTimeout(() => scheduleSubmenuPositionUpdate(retries - 1), 0)
		})
	}

	function schedulePositionUpdate() {
		if (!shown.value || positionRafId !== null) return

		positionRafId = window.requestAnimationFrame(() => {
			positionRafId = null
			updateMenuPosition(menuAnchor.value.x, menuAnchor.value.y)
		})
	}

	function cancelPositionUpdate() {
		if (positionRafId !== null) window.cancelAnimationFrame(positionRafId)
	}

	return {
		menuStyle,
		menuAnchor,
		submenuStyle,
		hasSubmenuPosition,
		updateMenuPosition,
		scheduleSubmenuPositionUpdate,
		schedulePositionUpdate,
		cancelPositionUpdate,
	}
}

function getViewportRect(): ViewportRect {
	const visualViewport = window.visualViewport
	return {
		width: visualViewport?.width ?? window.innerWidth,
		height: visualViewport?.height ?? window.innerHeight,
		offsetTop: visualViewport?.offsetTop ?? 0,
		offsetLeft: visualViewport?.offsetLeft ?? 0,
	}
}

function getSubmenuOpenDirection(
	menuRect: DOMRect,
	submenuWidth: number,
	viewport: ViewportRect,
): 'left' | 'right' {
	const viewportLeft = viewport.offsetLeft
	const viewportRight = viewport.offsetLeft + viewport.width
	const rightSpace = viewportRight - menuRect.right - MENU_GAP - VIEWPORT_MARGIN
	const leftSpace = menuRect.left - viewportLeft - MENU_GAP - VIEWPORT_MARGIN

	if (rightSpace >= submenuWidth) return 'right'
	if (leftSpace >= submenuWidth) return 'left'
	return rightSpace >= leftSpace ? 'right' : 'left'
}
