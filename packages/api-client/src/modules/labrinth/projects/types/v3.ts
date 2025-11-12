import type { MonetizationStatus, ProjectStatus } from './v2'

export type GalleryItemV3 = {
	url: string
	raw_url: string
	featured: boolean
	name?: string
	description?: string
	created: string
	ordering: number
}

export type LinkV3 = {
	platform: string
	donation: boolean
	url: string
}

export type ProjectV3 = {
	id: string
	slug?: string
	project_types: string[]
	games: string[]
	team_id: string
	organization?: string
	name: string
	summary: string
	description: string
	published: string
	updated: string
	approved?: string
	queued?: string
	status: ProjectStatus
	requested_status?: ProjectStatus
	license: {
		id: string
		name: string
		url?: string
	}
	downloads: number
	followers: number
	categories: string[]
	additional_categories: string[]
	loaders: string[]
	versions: string[]
	icon_url?: string
	link_urls: Record<string, LinkV3>
	gallery: GalleryItemV3[]
	color?: number
	thread_id: string
	monetization_status: MonetizationStatus
	side_types_migration_review_status: 'reviewed' | 'pending'
	[key: string]: unknown
}
