import type { Component } from 'vue'
import type { RouteLocationRaw } from 'vue-router'

import type { AnchoredTeleportPlacement } from '../../../utils/use-anchored-teleport'

export type ButtonType = 'base' | 'colored' | 'colored-text' | 'outlined' | 'quiet'

export type ButtonSize = 'xs' | 'sm' | 'md' | 'lg' | 'xl'

export type ButtonInteraction = 'surface' | 'filled' | 'none'

// TODO: Standardized color string enum props across @modrinth/ui
export type ButtonColor =
	| 'brand'
	| 'red'
	| 'orange'
	| 'green'
	| 'blue'
	| 'purple'
	| 'medal_promotion'

export type ButtonVisualProps = {
	size?: ButtonSize
	interaction?: ButtonInteraction
} & (
	| {
			type?: 'base'
			color?: never
	  }
	| {
			type: 'outlined'
			color?: ButtonColor
	  }
	| {
			type: 'colored'
			color?: ButtonColor
	  }
	| {
			type: 'colored-text'
			color?: ButtonColor
	  }
	| {
			type: 'quiet'
			color?: ButtonColor
	  }
)

export type ButtonNativeType = 'button' | 'submit' | 'reset'

export interface ButtonProps {
	type?: ButtonType
	color?: ButtonColor
	size?: ButtonSize
	interaction?: ButtonInteraction
	nativeType?: ButtonNativeType
	disabled?: boolean
	loading?: boolean
}

export type ButtonLinkDestination =
	| {
			to: RouteLocationRaw
			href?: never
	  }
	| {
			href: string
			to?: never
	  }

export type TeleportPlacement = AnchoredTeleportPlacement

export interface OverflowMenuItemBase {
	id: string
	label: string
	icon?: Component
	shown?: boolean
	disabled?: boolean
	tooltip?: string
	remainOpen?: boolean
	tone?: 'default' | ButtonColor
	hoverFilled?: boolean
	hoverFilledOnly?: boolean
}

export interface OverflowMenuAction extends OverflowMenuItemBase {
	type?: 'action'
	action: (event: MouseEvent) => void
}

export interface OverflowMenuLink extends OverflowMenuItemBase {
	type: 'link'
	to?: RouteLocationRaw
	href?: string
	target?: string
	rel?: string
	download?: string | boolean
}

export interface OverflowMenuDivider {
	type: 'divider'
	id?: string
	shown?: boolean
}

export type OverflowMenuOption = OverflowMenuAction | OverflowMenuLink | OverflowMenuDivider

export interface ButtonElementHandle {
	element: HTMLElement | null
}
