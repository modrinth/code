import type { ComponentPublicInstance } from 'vue'

export type ContextMenuDivider = {
	type: string
}

export type ContextMenuAction = {
	name: string
	color?: string
	children?: ContextMenuOption[]
}

export type ContextMenuParentAction = ContextMenuAction & {
	children: ContextMenuOption[]
}

export type ContextMenuOption = ContextMenuDivider | ContextMenuAction

export type ContextMenuSelection = {
	item: unknown
	option: string
}

export type ContextMenuEmit = {
	(event: 'menu-closed'): void
	(event: 'option-clicked', selection: ContextMenuSelection): void
}

export type Point = {
	x: number
	y: number
}

export type ViewportRect = {
	width: number
	height: number
	offsetTop: number
	offsetLeft: number
}

export type ButtonRefElement = Element | ComponentPublicInstance | null
