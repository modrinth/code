import type { Component } from 'vue'
import type { RouteLocationRaw } from 'vue-router'

export type ButtonVariant = 'base' | 'colored' | 'outlined' | 'quiet'

export type ButtonSize = 'sm' | 'default' | 'md' | 'lg'

export type ButtonTone = 'brand' | 'red' | 'orange' | 'green' | 'blue' | 'purple' | 'promotion'

export type ButtonVisualProps = {
	size?: ButtonSize
} & (
	| {
			variant?: 'base'
			tone?: never
	  }
	| {
			variant: 'outlined'
			tone?: never
	  }
	| {
			variant: 'colored'
			tone?: ButtonTone
	  }
	| {
			variant: 'quiet'
			tone?: ButtonTone
	  }
)

export type ButtonNativeType = 'button' | 'submit' | 'reset'

export type ButtonLinkDestination =
	| {
			to: RouteLocationRaw
			href?: never
	  }
	| {
			href: string
			to?: never
	  }

export type TeleportPlacement = 'bottom-start' | 'bottom-end' | 'top-start' | 'top-end'

export interface OverflowMenuItemBase {
	id: string
	label: string
	icon?: Component
	shown?: boolean
	disabled?: boolean
	tooltip?: string
	remainOpen?: boolean
	tone?: 'default' | 'red'
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
