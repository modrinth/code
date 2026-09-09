import type { CSSProperties, Ref } from 'vue'
import { onUnmounted, ref } from 'vue'

import type {
	ButtonColor,
	ButtonMenuAction,
	ButtonMenuDivider,
	ButtonMenuHeading,
	ButtonMenuLink,
	ButtonMenuOption,
	ButtonMenuSubmenu,
} from '../types'

export const buttonMenuItemClasses =
	'button-menu-item flex min-h-10 z-10 w-full items-center gap-2 rounded-[10px] border-0 bg-transparent px-3 py-2 text-left text-base font-semibold leading-5 text-contrast no-underline ' +
	'cursor-pointer whitespace-nowrap hover:bg-surface-4 focus-visible:bg-surface-4 focus-visible:outline-none ' +
	'disabled:cursor-not-allowed disabled:opacity-50 [&[aria-disabled=true]]:cursor-not-allowed [&[aria-disabled=true]]:opacity-50 ' +
	'[&>svg]:size-5 [&>svg]:shrink-0 [&>svg]:text-primary'

export const buttonMenuPanelClasses =
	'fixed isolate z-[9999] rounded-[14px] bg-surface-3 shadow-lg ring-1 ring-surface-5 select-none'

export const menuPanelPadding = 8
export const submenuGap = 2
export const menuItemSelector = '[role="menuitem"]'
// submenu items render inside this panel, so skip them when moving focus
export const topLevelMenuItemSelector = `${menuItemSelector}:not([data-button-menu-submenu-item])`

const TYPEAHEAD_RESET_DELAY = 500

export const buttonMenuTones: Record<ButtonColor, string> = {
	brand: 'var(--color-brand)',
	red: 'var(--color-red)',
	orange: 'var(--color-orange)',
	green: 'var(--color-green)',
	blue: 'var(--color-blue)',
	purple: 'var(--color-purple)',
	medal_promotion: 'var(--medal-promotion-text-orange, var(--color-orange))',
}

export function isDivider(option: ButtonMenuOption): option is ButtonMenuDivider {
	return option.type === 'divider'
}

export function isHeading(option: ButtonMenuOption): option is ButtonMenuHeading {
	return option.type === 'heading'
}

export function isLink(option: ButtonMenuOption): option is ButtonMenuLink {
	return option.type === 'link'
}

export function isSubmenu(option: ButtonMenuOption): option is ButtonMenuSubmenu {
	return option.type === 'submenu'
}

export function isMenuRow(
	option: ButtonMenuOption,
): option is ButtonMenuAction | ButtonMenuLink | ButtonMenuSubmenu {
	return option.type !== 'divider' && option.type !== 'heading'
}

export function visibleOptions<T extends { shown?: boolean }>(options: T[]): T[] {
	return options.filter((option) => option.shown !== false)
}

export function getButtonMenuItemAttrs(
	option: ButtonMenuAction | ButtonMenuLink | ButtonMenuSubmenu,
) {
	const tone = option.tone && option.tone !== 'default' ? option.tone : undefined

	return {
		role: 'menuitem',
		tabindex: '-1',
		style: tone
			? ({ '--button-menu-item-tone': buttonMenuTones[tone] } as CSSProperties)
			: undefined,
		'data-tone': tone,
		'data-hover-filled': option.hoverFilled || option.hoverFilledOnly || undefined,
		'data-hover-filled-only': option.hoverFilledOnly || undefined,
	}
}

export function useButtonMenuNavigation(
	panel: Readonly<Ref<HTMLElement | null>>,
	itemSelector: string,
) {
	const focusedIndex = ref(-1)

	function getItems() {
		if (!panel.value) return []
		return Array.from(panel.value.querySelectorAll<HTMLElement>(itemSelector))
	}

	function focusItem(index: number) {
		const items = getItems()
		if (items.length === 0) return
		focusedIndex.value = (index + items.length) % items.length
		items[focusedIndex.value]?.focus()
	}

	function handleNavigationKeydown(event: KeyboardEvent) {
		const items = getItems()
		if (items.length === 0) return false

		const activeIndex = items.indexOf(document.activeElement as HTMLElement)
		const currentIndex = activeIndex === -1 ? focusedIndex.value : activeIndex

		switch (event.key) {
			case 'ArrowDown':
				focusItem(currentIndex + 1)
				break
			case 'ArrowUp':
				focusItem(currentIndex === -1 ? items.length - 1 : currentIndex - 1)
				break
			case 'Home':
				focusItem(0)
				break
			case 'End':
				focusItem(items.length - 1)
				break
			default:
				return false
		}

		event.preventDefault()
		return true
	}

	return { focusedIndex, getItems, focusItem, handleNavigationKeydown }
}

export function useMenuKeyboard(options: {
	panel: Readonly<Ref<HTMLElement | null>>
	rows: () => { label: string }[]
	onEscape: () => void
	onTab?: () => void
}) {
	const navigation = useButtonMenuNavigation(options.panel, topLevelMenuItemSelector)
	const typeahead = ref('')
	let typeaheadTimer: ReturnType<typeof setTimeout> | undefined

	function clearTypeahead() {
		if (typeaheadTimer) clearTimeout(typeaheadTimer)
		typeaheadTimer = undefined
		typeahead.value = ''
	}

	function focusTypeaheadMatch(event: KeyboardEvent) {
		if (
			event.key === ' ' ||
			event.key.length !== 1 ||
			event.ctrlKey ||
			event.metaKey ||
			event.altKey
		) {
			return
		}

		const character = event.key.toLocaleLowerCase()
		// repeating a letter cycles matches instead of appending
		const query = typeahead.value === character ? character : `${typeahead.value}${character}`
		const startIndex = query.length === 1 ? navigation.focusedIndex.value + 1 : 0
		const rows = options.rows()
		typeahead.value = query

		for (let offset = 0; offset < rows.length; offset++) {
			const index = (startIndex + offset) % rows.length
			if (rows[index]?.label.toLocaleLowerCase().startsWith(query)) {
				navigation.focusItem(index)
				break
			}
		}

		if (typeaheadTimer) clearTimeout(typeaheadTimer)
		typeaheadTimer = setTimeout(clearTypeahead, TYPEAHEAD_RESET_DELAY)
	}

	function handleKeydown(event: KeyboardEvent) {
		if (navigation.handleNavigationKeydown(event)) return

		switch (event.key) {
			case 'Escape':
				event.preventDefault()
				options.onEscape()
				break
			case 'Tab':
				options.onTab?.()
				break
			default:
				focusTypeaheadMatch(event)
		}
	}

	function reset() {
		navigation.focusedIndex.value = -1
		clearTypeahead()
	}

	onUnmounted(clearTypeahead)

	return { ...navigation, handleKeydown, reset }
}

export function useHoverIntent(options: {
	closeDelay: number
	enabled?: () => boolean
	onEnter: () => void
	onLeave: () => void
}) {
	let leaveTimer: ReturnType<typeof setTimeout> | undefined

	function cancelLeave() {
		if (leaveTimer === undefined) return
		clearTimeout(leaveTimer)
		leaveTimer = undefined
	}

	function isIgnored() {
		// touch reports hover on tap, which would open menus you meant to scroll past
		return options.enabled?.() === false || !window.matchMedia('(hover: hover)').matches
	}

	function handleMouseEnter() {
		if (isIgnored()) return
		cancelLeave()
		options.onEnter()
	}

	function handleMouseLeave() {
		if (isIgnored()) return
		cancelLeave()
		leaveTimer = setTimeout(options.onLeave, options.closeDelay)
	}

	onUnmounted(cancelLeave)

	return { handleMouseEnter, handleMouseLeave, cancelLeave }
}
