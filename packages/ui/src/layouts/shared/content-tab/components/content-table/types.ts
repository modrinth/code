import type { ContentCardTableItem } from '../../types'

export type ContentTableAction = 'update' | 'switchVersion' | 'toggle' | 'remove'

export interface ContentTableColumns {
	selection?: boolean
	version?: boolean
	enabledFor?: boolean
	actions?: ContentTableAction[]
}

export interface ContentTableSection {
	id: string
	label?: string
	items: ContentCardTableItem[]
}

export type ContentTableLayout = 'wide' | 'compact' | 'stacked' | 'narrow'
