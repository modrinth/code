import type { Component, HTMLAttributes, StyleValue } from 'vue'
import type { RouteLocationRaw } from 'vue-router'

import type { JoinedButtonAction } from '../JoinedButtons.vue'
import type { Item as TeleportOverflowMenuItem } from '../TeleportOverflowMenu.vue'

export type PageHeaderTarget = string | RouteLocationRaw
export type PageHeaderClass = HTMLAttributes['class']

export type ButtonColor =
	| 'standard'
	| 'brand'
	| 'red'
	| 'orange'
	| 'green'
	| 'blue'
	| 'purple'
	| 'medal-promo'
export type JoinedButtonColor = Exclude<ButtonColor, 'medal-promo'>
export type ButtonSize = 'standard' | 'large' | 'small'
export type ButtonType =
	| 'standard'
	| 'outlined'
	| 'transparent'
	| 'highlight'
	| 'highlight-colored-text'
	| 'chip'

export type FloatingPlacement =
	| 'auto'
	| 'auto-start'
	| 'auto-end'
	| 'top'
	| 'top-start'
	| 'top-end'
	| 'right'
	| 'right-start'
	| 'right-end'
	| 'bottom'
	| 'bottom-start'
	| 'bottom-end'
	| 'left'
	| 'left-start'
	| 'left-end'

export type PageHeaderClickHandler = (event: MouseEvent) => void | Promise<void>

type ComponentEscapeHatch = {
	component: Component
	componentProps?: Record<string, unknown>
	class?: PageHeaderClass
}

export type PageHeaderAvatarLeading = {
	id?: string
	type: 'avatar'
	src?: string | null
	alt?: string
	tintBy?: string | null
	avatarSize?: string
	circle?: boolean
	noShadow?: boolean
	raised?: boolean
	loading?: 'eager' | 'lazy'
	class?: PageHeaderClass
}

type PageHeaderButtonLeadingBase = {
	id?: string
	type: 'button'
	icon?: Component
	iconProps?: Record<string, unknown>
	wrapperClass?: PageHeaderClass
	tooltip?: string
	ariaLabel?: string
	color?: ButtonColor
	size?: ButtonSize
	buttonType?: ButtonType
	disabled?: boolean
}

export type PageHeaderButtonLeading =
	| (PageHeaderButtonLeadingBase & {
			to: PageHeaderTarget
			onClick?: never
	  })
	| (PageHeaderButtonLeadingBase & {
			to?: never
			onClick: PageHeaderClickHandler
	  })

export type PageHeaderComponentLeading = {
	id?: string
	type: 'component'
} & ComponentEscapeHatch

export type PageHeaderLeading =
	| PageHeaderAvatarLeading
	| PageHeaderButtonLeading
	| PageHeaderComponentLeading

export type PageHeaderBadgeComponent = {
	id: string
	type: 'component'
} & ComponentEscapeHatch

export type PageHeaderBadgeContent = {
	id: string
	type?: 'badge'
	label?: string
	icon?: Component
	iconProps?: Record<string, unknown>
	tooltip?: string
	ariaLabel?: string
	to?: PageHeaderTarget
	onClick?: PageHeaderClickHandler
	disabled?: boolean
	class?: PageHeaderClass
	style?: StyleValue
}

export type PageHeaderBadge = PageHeaderBadgeComponent | PageHeaderBadgeContent

export type PageHeaderMetadataTag = {
	id: string
	tag?: string
	label?: string
	onClick?: PageHeaderClickHandler
}

export type PageHeaderMetadataComponentItem = {
	id: string
	type: 'component'
} & ComponentEscapeHatch

export type PageHeaderMetadataContentItem = {
	id: string
	type?: 'text'
	label?: string
	value?: string | number | null
	icon?: Component
	iconProps?: Record<string, unknown>
	avatarSrc?: string | null
	avatarAlt?: string
	avatarSize?: string
	avatarTintBy?: string | null
	avatarCircle?: boolean
	avatarNoShadow?: boolean
	avatarRaised?: boolean
	tags?: PageHeaderMetadataTag[]
	tooltip?: string
	ariaLabel?: string
	to?: PageHeaderTarget
	onClick?: PageHeaderClickHandler
	disabled?: boolean
	class?: PageHeaderClass
	labelClass?: PageHeaderClass
	valueClass?: PageHeaderClass
}

export type PageHeaderMetadataItem =
	| PageHeaderMetadataComponentItem
	| PageHeaderMetadataContentItem

export type PageHeaderActionPrompt = {
	title: string
	description: string
	badge?: string
	footer?: string
	dismissLabel?: string
	shown?: boolean
	placement?: FloatingPlacement
	onDismiss?: () => void
}

type PageHeaderActionBase = {
	id: string
	label: string
	icon?: Component
	iconProps?: Record<string, unknown>
	iconClass?: PageHeaderClass
	tooltip?: string
	ariaLabel?: string
	onClick?: PageHeaderClickHandler
	disabled?: boolean
	labelHidden?: boolean
	circular?: boolean
	color?: ButtonColor
	size?: ButtonSize
	type?: ButtonType
	prompt?: PageHeaderActionPrompt
}

export type PageHeaderComponentAction = PageHeaderActionBase &
	ComponentEscapeHatch & {
		kind: 'component'
	}

export type PageHeaderJoinedAction = PageHeaderActionBase & {
	joinedActions: JoinedButtonAction[]
	primaryDisabled?: boolean
	dropdownDisabled?: boolean
	primaryMuted?: boolean
	kind?: never
	component?: never
	componentProps?: never
	menuActions?: never
	to?: never
}

export type PageHeaderMenuAction = PageHeaderActionBase & {
	menuActions: TeleportOverflowMenuItem[]
	kind?: never
	component?: never
	componentProps?: never
	joinedActions?: never
	to?: never
}

export type PageHeaderLinkAction = PageHeaderActionBase & {
	to?: PageHeaderTarget
	kind?: never
	component?: never
	componentProps?: never
	joinedActions?: never
	menuActions?: never
}

export type PageHeaderButtonAction = PageHeaderActionBase & {
	kind?: never
	component?: never
	componentProps?: never
	joinedActions?: never
	menuActions?: never
	to?: never
}

export type PageHeaderAction =
	| PageHeaderComponentAction
	| PageHeaderJoinedAction
	| PageHeaderMenuAction
	| PageHeaderLinkAction
	| PageHeaderButtonAction

export type PageHeaderProps = {
	title: string
	summary?: string | null
	leading?: PageHeaderLeading | PageHeaderLeading[] | null
	badges?: PageHeaderBadge[]
	metadata?: PageHeaderMetadataItem[]
	actions?: PageHeaderAction[]
	headerClass?: PageHeaderClass
	rowClass?: PageHeaderClass
	mainClass?: PageHeaderClass
	titleClass?: PageHeaderClass
	truncateTitle?: boolean
	divider?: boolean
	bottomPadding?: boolean
	disableLineClamp?: boolean
}
