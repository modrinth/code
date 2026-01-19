import type { Labrinth } from '@modrinth/api-client'
import type { RouteLocationRaw } from 'vue-router'

export type ContentCardProject = Pick<
	Labrinth.Projects.v2.Project,
	'id' | 'slug' | 'title' | 'icon_url'
>

export type ContentCardVersion = Pick<Labrinth.Versions.v2.Version, 'id' | 'version_number'> & {
	file_name: string
}

export interface ContentOwner {
	id: string
	name: string
	avatar_url?: string
	type: 'user' | 'organization'
	link?: string | RouteLocationRaw
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
