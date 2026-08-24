import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'

import type {
	ButtonRefElement,
	ContextMenuAction,
	ContextMenuEmit,
	ContextMenuOption,
	ContextMenuParentAction,
	Point,
} from './types'
import { useContextMenuPosition } from './use-context-menu-position'
import {
	focusRelativeButton,
	getFocusableButtons,
	hasChildren,
	isAction,
	isInstanceLink,
	isPointInTriangle,
} from './utils'

const MOBILE_SUBMENU_LAYOUT_QUERY = '(pointer: coarse), (max-width: 800px)'
const CONTEXT_MENU_OPEN_EVENT = 'modrinth-context-menu-open'

export function useContextMenu(emit: ContextMenuEmit) {
	const item = ref<unknown>(null)
	const contextMenu = ref<HTMLElement | null>(null)
	const submenu = ref<HTMLElement | null>(null)
	const options = ref<ContextMenuOption[]>([])
	const shown = ref(false)
	const activeOptionIndex = ref<number | null>(null)
	const pendingOptionIndex = ref<number | null>(null)
	const isCursorInsideSubmenu = ref(false)
	const isMobileSubmenuLayout = ref(false)
	const lastMousePosition = ref<Point | null>(null)
	const contextMenuId = Symbol()
	const optionButtonRefs = new Map<number, HTMLElement>()
	const submenuButtonRefs = new Map<number, HTMLElement>()
	let previousMousePosition: Point | null = null
	let pendingOptionTimeout: ReturnType<typeof setTimeout> | null = null
	let mobileSubmenuMediaQuery: MediaQueryList | null = null

	const activeOption = computed<ContextMenuParentAction | null>(() => {
		if (activeOptionIndex.value === null) return null
		const option = options.value[activeOptionIndex.value]
		return option && isAction(option) && hasChildren(option) ? option : null
	})

	const {
		menuStyle,
		menuAnchor,
		submenuStyle,
		hasSubmenuPosition,
		updateMenuPosition,
		scheduleSubmenuPositionUpdate,
		schedulePositionUpdate,
		cancelPositionUpdate,
	} = useContextMenuPosition({
		shown,
		activeOption,
		activeOptionIndex,
		isMobileSubmenuLayout,
		contextMenu,
		submenu,
		optionButtonRefs,
	})

	const isMobileActiveSubmenu = computed(
		() => isMobileSubmenuLayout.value && activeOption.value !== null && hasSubmenuPosition.value,
	)

	function isOptionVisible(option: ContextMenuOption): option is ContextMenuAction {
		return isAction(option) && !(isInstanceLink(item.value) && option.name === 'add_content')
	}

	function setOptionButtonRef(index: number, element: ButtonRefElement) {
		setButtonRef(optionButtonRefs, index, element)
	}

	function setSubmenuButtonRef(index: number, element: ButtonRefElement) {
		setButtonRef(submenuButtonRefs, index, element)
	}

	function hideMenu() {
		if (!shown.value) return

		shown.value = false
		deactivateSubmenu()
		emit('menu-closed')
	}

	function showMenu(event: MouseEvent, passedItem: unknown, passedOptions: ContextMenuOption[]) {
		window.dispatchEvent(new CustomEvent(CONTEXT_MENU_OPEN_EVENT, { detail: contextMenuId }))

		item.value = passedItem
		options.value = passedOptions
		menuAnchor.value = { x: event.clientX, y: event.clientY }
		shown.value = true
		deactivateSubmenu()
		syncMobileSubmenuLayout()
		nextTick(() => updateMenuPosition(event.clientX, event.clientY))
	}

	function handleOptionClick(option: ContextMenuAction, index: number) {
		if (hasChildren(option)) {
			activateSubmenu(index)
			return
		}

		optionClicked(option.name)
	}

	function optionClicked(option: string) {
		emit('option-clicked', { item: item.value, option })
		hideMenu()
	}

	function handleOptionFocus(option: ContextMenuAction, index: number) {
		if (hasChildren(option) && !isMobileSubmenuLayout.value) {
			activateSubmenu(index)
		} else if (!hasChildren(option)) {
			deactivateSubmenu()
		}
	}

	function handleOptionMouseEnter(option: ContextMenuAction, index: number) {
		if (isMobileSubmenuLayout.value) return

		if (activeOptionIndex.value === null) {
			if (hasChildren(option)) activateSubmenu(index)
			return
		}

		if (activeOptionIndex.value === index) return

		if (!isCursorAimingAtSubmenu(lastMousePosition.value, previousMousePosition)) {
			commitHoveredOption(index)
			return
		}

		pendingOptionIndex.value = index
		clearPendingOptionTimeout()
		pendingOptionTimeout = setTimeout(() => {
			if (pendingOptionIndex.value !== index) return
			if (isCursorInsideSubmenu.value) {
				pendingOptionIndex.value = null
				return
			}

			commitHoveredOption(index)
		}, 180)
	}

	function commitHoveredOption(index: number) {
		const option = options.value[index]
		if (option && hasChildren(option)) {
			activateSubmenu(index)
		} else {
			deactivateSubmenu()
		}
	}

	function activateSubmenu(index: number) {
		clearPendingOptionTimeout()
		pendingOptionIndex.value = null
		activeOptionIndex.value = index
		hasSubmenuPosition.value = false
		scheduleSubmenuPositionUpdate()
	}

	function deactivateSubmenu() {
		clearPendingOptionTimeout()
		activeOptionIndex.value = null
		pendingOptionIndex.value = null
		hasSubmenuPosition.value = false
		isCursorInsideSubmenu.value = false
		lastMousePosition.value = null
		previousMousePosition = null
	}

	function returnToMenu() {
		const previousIndex = activeOptionIndex.value
		deactivateSubmenu()
		nextTick(() => {
			if (previousIndex !== null) optionButtonRefs.get(previousIndex)?.focus()
		})
	}

	function handleSubmenuMouseEnter() {
		isCursorInsideSubmenu.value = true
		clearPendingOptionTimeout()
		pendingOptionIndex.value = null
	}

	function handleMenuMouseMove(event: MouseEvent, source: 'menu' | 'submenu') {
		previousMousePosition = lastMousePosition.value
		lastMousePosition.value = { x: event.clientX, y: event.clientY }

		if (
			source === 'menu' &&
			pendingOptionIndex.value !== null &&
			!isCursorAimingAtSubmenu(lastMousePosition.value, previousMousePosition)
		) {
			commitHoveredOption(pendingOptionIndex.value)
		}
	}

	function isCursorAimingAtSubmenu(cursor: Point | null, origin: Point | null) {
		const submenuRect = submenu.value?.getBoundingClientRect()
		if (!submenuRect || !cursor || !origin) return false

		const submenuTargetX =
			origin.x <= submenuRect.left
				? submenuRect.left
				: origin.x >= submenuRect.right
					? submenuRect.right
					: cursor.x <= submenuRect.left
						? submenuRect.left
						: submenuRect.right
		const upperTarget = { x: submenuTargetX, y: submenuRect.top - 20 }
		const lowerTarget = { x: submenuTargetX, y: submenuRect.bottom + 20 }

		return isPointInTriangle(cursor, origin, upperTarget, lowerTarget)
	}

	function handleMenuKeydown(event: KeyboardEvent) {
		const buttons = getFocusableButtons(optionButtonRefs)
		if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
			event.preventDefault()
			focusRelativeButton(buttons, event.key === 'ArrowDown' ? 1 : -1)
		} else if (event.key === 'Home' || event.key === 'End') {
			event.preventDefault()
			buttons[event.key === 'Home' ? 0 : buttons.length - 1]?.focus()
		} else if (event.key === 'ArrowRight') {
			const focusedIndex = [...optionButtonRefs.entries()].find(
				([, button]) => button === document.activeElement,
			)?.[0]
			const focusedOption = focusedIndex === undefined ? undefined : options.value[focusedIndex]
			if (focusedIndex !== undefined && focusedOption && hasChildren(focusedOption)) {
				event.preventDefault()
				activateSubmenu(focusedIndex)
				nextTick(() => getFocusableButtons(submenuButtonRefs)[0]?.focus())
			}
		} else if (event.key === 'Escape') {
			event.preventDefault()
			hideMenu()
		}
	}

	function handleSubmenuKeydown(event: KeyboardEvent) {
		const buttons = getFocusableButtons(submenuButtonRefs)
		if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
			event.preventDefault()
			focusRelativeButton(buttons, event.key === 'ArrowDown' ? 1 : -1)
		} else if (event.key === 'Home' || event.key === 'End') {
			event.preventDefault()
			buttons[event.key === 'Home' ? 0 : buttons.length - 1]?.focus()
		} else if (event.key === 'ArrowLeft') {
			event.preventDefault()
			returnToMenu()
		} else if (event.key === 'Escape') {
			event.preventDefault()
			hideMenu()
		}
	}

	function syncMobileSubmenuLayout(event?: MediaQueryListEvent) {
		isMobileSubmenuLayout.value = event?.matches ?? mobileSubmenuMediaQuery?.matches ?? false
	}

	function handleDocumentKeydown(event: KeyboardEvent) {
		if (shown.value && event.key === 'Escape') hideMenu()
	}

	function handleContextMenuOpen(event: Event) {
		if (shown.value && event instanceof CustomEvent && event.detail !== contextMenuId) hideMenu()
	}

	function handleClickOutside(event: MouseEvent) {
		const target = event.target
		if (!(target instanceof Node)) return
		if (!contextMenu.value?.contains(target) && !submenu.value?.contains(target)) hideMenu()
	}

	onMounted(() => {
		mobileSubmenuMediaQuery = window.matchMedia(MOBILE_SUBMENU_LAYOUT_QUERY)
		syncMobileSubmenuLayout()
		mobileSubmenuMediaQuery.addEventListener('change', syncMobileSubmenuLayout)
		window.addEventListener('click', handleClickOutside)
		window.addEventListener('resize', schedulePositionUpdate)
		window.addEventListener('scroll', schedulePositionUpdate, true)
		window.visualViewport?.addEventListener('scroll', schedulePositionUpdate)
		window.visualViewport?.addEventListener('resize', schedulePositionUpdate)
		window.addEventListener(CONTEXT_MENU_OPEN_EVENT, handleContextMenuOpen)
		document.addEventListener('keydown', handleDocumentKeydown)
	})

	onBeforeUnmount(() => {
		clearPendingOptionTimeout()
		cancelPositionUpdate()
		mobileSubmenuMediaQuery?.removeEventListener('change', syncMobileSubmenuLayout)
		window.removeEventListener('click', handleClickOutside)
		window.removeEventListener('resize', schedulePositionUpdate)
		window.removeEventListener('scroll', schedulePositionUpdate, true)
		window.visualViewport?.removeEventListener('scroll', schedulePositionUpdate)
		window.visualViewport?.removeEventListener('resize', schedulePositionUpdate)
		window.removeEventListener(CONTEXT_MENU_OPEN_EVENT, handleContextMenuOpen)
		document.removeEventListener('keydown', handleDocumentKeydown)
	})

	function clearPendingOptionTimeout() {
		if (!pendingOptionTimeout) return
		clearTimeout(pendingOptionTimeout)
		pendingOptionTimeout = null
	}

	return {
		shown,
		contextMenu,
		submenu,
		options,
		menuStyle,
		submenuStyle,
		activeOption,
		activeOptionIndex,
		pendingOptionIndex,
		hasSubmenuPosition,
		isMobileSubmenuLayout,
		isMobileActiveSubmenu,
		isCursorInsideSubmenu,
		isOptionVisible,
		setOptionButtonRef,
		setSubmenuButtonRef,
		showMenu,
		hideMenu,
		handleOptionClick,
		optionClicked,
		handleOptionFocus,
		handleOptionMouseEnter,
		returnToMenu,
		handleSubmenuMouseEnter,
		handleMenuMouseMove,
		handleMenuKeydown,
		handleSubmenuKeydown,
	}
}

function setButtonRef(
	buttonRefs: Map<number, HTMLElement>,
	index: number,
	element: ButtonRefElement,
) {
	if (element instanceof HTMLElement) {
		buttonRefs.set(index, element)
	} else {
		buttonRefs.delete(index)
	}
}
