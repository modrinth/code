import type { Labrinth } from '@modrinth/api-client'
import type { RouteLocationRaw } from 'vue-router'

import type { Option as OverflowMenuOption } from '../base/OverflowMenu.vue'

export type ContentCardProject = Pick<
	Labrinth.Projects.v2.Project,
	'id' | 'slug' | 'title' | 'icon_url'
>

export type ContentCardVersion = Pick<Labrinth.Versions.v2.Version, 'id' | 'version_number'> & {
	file_name: string
	date_published?: string
}

export interface ContentOwner {
	id: string
	name: string
	avatar_url?: string
	type: 'user' | 'organization'
	link?: string | RouteLocationRaw
}

export interface ContentCardTableItem {
	id: string
	project: ContentCardProject
	projectLink?: string | RouteLocationRaw
	version?: ContentCardVersion
	versionLink?: string | RouteLocationRaw
	owner?: ContentOwner
	enabled?: boolean
	disabled?: boolean
	hasUpdate?: boolean
	overflowOptions?: OverflowMenuOption[]
}

export type ContentCardTableSortColumn = 'project' | 'version'
export type ContentCardTableSortDirection = 'asc' | 'desc'

/** Content item returned from the app backend API - maps to ContentCardTableItem for display */
export interface ContentItem extends Omit<
	ContentCardTableItem,
	'id' | 'projectLink' | 'disabled' | 'overflowOptions'
> {
	file_name: string
	file_path?: string
	hash?: string
	size?: number
	project_type: string
	has_update: boolean
	update_version_id: string | null
	date_added?: string
}

export type ContentModpackCardProject = Pick<
	Labrinth.Projects.v2.Project,
	'id' | 'slug' | 'title' | 'icon_url' | 'description' | 'downloads' | 'followers'
>

export type ContentModpackCardVersion = Pick<
	Labrinth.Versions.v2.Version,
	'id' | 'version_number' | 'date_published'
>

export type ContentModpackCardCategory = Labrinth.Tags.v2.Category & {
	action?: (event: MouseEvent) => void
}
