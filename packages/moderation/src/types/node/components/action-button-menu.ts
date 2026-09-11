/**
 * Cross-instance coordination for `ActionButton`'s sub-option hover menu.
 *
 * That menu never closes on its own from a mouse-leave (a sub-option can be a dropdown whose own
 * listbox teleports elsewhere in the DOM, so treating "cursor left this popper" as "close it"
 * would yank the menu away mid-click). Without this, hovering from one toggle's menu to a
 * different one — without an explicit click to dismiss the first — would leave both open at
 * once. At most one may be open at a time: opening one force-closes whichever other one was open.
 */

let currentlyOpenMenu: (() => void) | null = null

export function closeOtherChecklistMenus(hide: () => void): void {
	if (currentlyOpenMenu && currentlyOpenMenu !== hide) currentlyOpenMenu()
	currentlyOpenMenu = hide
}

export function clearChecklistMenu(hide: () => void): void {
	if (currentlyOpenMenu === hide) currentlyOpenMenu = null
}
