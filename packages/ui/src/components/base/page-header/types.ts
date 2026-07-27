import type { Component, HTMLAttributes } from 'vue'
import type { RouteLocationRaw } from 'vue-router'

export type PageHeaderTarget = string | RouteLocationRaw
export type PageHeaderClass = HTMLAttributes['class']

export type PageHeaderClickHandler = (event: MouseEvent) => void | Promise<void>

export type PageHeaderIconProps = {
	icon?: Component
	iconProps?: Record<string, unknown>
	iconClass?: PageHeaderClass
}

export type PageHeaderInteractiveProps = {
	tooltip?: string
	ariaLabel?: string
	to?: PageHeaderTarget
	action?: PageHeaderClickHandler
	disabled?: boolean
}

export type PageHeaderMetadataItemProps = PageHeaderIconProps & PageHeaderInteractiveProps

export type PageHeaderProps = {
	title: string
	summary?: string | null
	headerClass?: PageHeaderClass
	rowClass?: PageHeaderClass
	mainClass?: PageHeaderClass
	titleClass?: PageHeaderClass
	truncateTitle?: boolean
	divider?: boolean
	bottomPadding?: boolean
	disableLineClamp?: boolean
}
