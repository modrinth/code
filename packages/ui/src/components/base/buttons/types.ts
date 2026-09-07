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

export interface ButtonMenuItemBase {
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
	selected?: boolean
	trailingAction?: {
		label: string
		icon: Component
		color?: ButtonColor
		action: (event: MouseEvent) => void
	}
}

export interface ButtonMenuAction extends ButtonMenuItemBase {
	type?: 'action'
	action: (event: MouseEvent) => void
}

export interface ButtonMenuLink extends ButtonMenuItemBase {
	type: 'link'
	to?: RouteLocationRaw
	href?: string
	target?: string
	rel?: string
	download?: string | boolean
}

export interface ButtonMenuDivider {
	type: 'divider'
	id?: string
	shown?: boolean
}

export interface ButtonMenuHeading {
	type: 'heading'
	id?: string
	label: string
	shown?: boolean
}

export type ButtonMenuLeafOption =
	| ButtonMenuAction
	| ButtonMenuLink
	| ButtonMenuDivider
	| ButtonMenuHeading

export interface ButtonMenuSubmenu extends ButtonMenuItemBase {
	type: 'submenu'
	options: ButtonMenuLeafOption[]
}

export type ButtonMenuOption = ButtonMenuLeafOption | ButtonMenuSubmenu

export interface ButtonElementHandle {
	element: HTMLElement | null
}
