export type Environment = 'required' | 'optional' | 'unsupported' | 'unknown'

export type ProjectStatus =
	| 'approved'
	| 'archived'
	| 'rejected'
	| 'draft'
	| 'unlisted'
	| 'processing'
	| 'withheld'
	| 'scheduled'
	| 'private'
	| 'unknown'

export type MonetizationStatus = 'monetized' | 'demonetized' | 'force-demonetized'

export type ProjectType = 'mod' | 'modpack' | 'resourcepack' | 'shader' | 'plugin' | 'datapack'

export type GalleryImageV2 = {
	url: string
	featured: boolean
	title?: string
	description?: string
	created: string
	ordering: number
}

export type DonationLinkV2 = {
	id: string
	platform: string
	url: string
}

export type ProjectV2 = {
	id: string
	slug: string
	project_type: ProjectType
	team: string
	title: string
	description: string
	body: string
	published: string
	updated: string
	approved?: string
	queued?: string
	status: ProjectStatus
	requested_status?: ProjectStatus
	moderator_message?: {
		message: string
		body?: string
	}
	license: {
		id: string
		name: string
		url?: string
	}
	client_side: Environment
	server_side: Environment
	downloads: number
	followers: number
	categories: string[]
	additional_categories: string[]
	game_versions: string[]
	loaders: string[]
	versions: string[]
	icon_url?: string
	issues_url?: string
	source_url?: string
	wiki_url?: string
	discord_url?: string
	donation_urls?: DonationLinkV2[]
	gallery?: GalleryImageV2[]
	color?: number
	thread_id: string
	monetization_status: MonetizationStatus
}

export type SearchResultHit = {
	project_id: string
	project_type: ProjectType
	slug: string
	author: string
	title: string
	description: string
	categories: string[]
	display_categories: string[]
	versions: string[]
	downloads: number
	follows: number
	icon_url: string
	date_created: string
	date_modified: string
	latest_version?: string
	license: string
	client_side: Environment
	server_side: Environment
	gallery: string[]
	color?: number
}

export type SearchResult = {
	hits: SearchResultHit[]
	offset: number
	limit: number
	total_hits: number
}

export type ProjectSearchParams = {
	query?: string
	facets?: string[][]
	filters?: string
	index?: 'relevance' | 'downloads' | 'follows' | 'newest' | 'updated'
	offset?: number
	limit?: number
}
