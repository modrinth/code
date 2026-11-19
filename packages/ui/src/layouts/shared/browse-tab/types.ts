import type { Labrinth } from '@modrinth/api-client'
import type { Component, Ref } from 'vue'
import type { RouteLocationRaw } from 'vue-router'

export type BrowseHit = Labrinth.Search.v3.ResultSearchProject

export interface BrowseTags {
	gameVersions: Labrinth.Tags.v2.GameVersion[]
	loaders: Labrinth.Tags.v2.Loader[]
	categories: Labrinth.Tags.v2.Category[]
}

export type BrowseSearchResults = Pick<Labrinth.Search.v3.SearchResults, 'hits' | 'total_hits'>

export interface BrowseSearchQueryOptions {
	queryKey: readonly unknown[]
	queryFn: () => Promise<BrowseSearchResults>
	staleTime?: number
	networkMode?: 'online' | 'always' | 'offlineFirst'
}

export type BrowseDisplayMode = 'list' | 'grid' | 'gallery'

export interface BrowseProjectTypeLink {
	label: string
	href: string
	shown?: boolean
}

export interface BrowseTarget {
	name: string
	iconSrc?: string | null
	tintBy?: string | null
	isMedal?: boolean
	serverId?: string | null
	gameVersion?: string | null
	loader?: string | null
	heading?: string
	warning?: string
	back: {
		to: string | RouteLocationRaw
		label: string
	}
}

export interface BrowseSelectedProject {
	id: string
	name: string
	iconUrl?: string | null
}

export interface BrowseSelection {
	projects: Readonly<Ref<BrowseSelectedProject[]>>
	installing: Readonly<Ref<boolean>>
	clear: () => void
	installAndBack: () => Promise<boolean>
	discardAndBack: () => Promise<void>
}

export type BrowseInstallState =
	| 'idle'
	| 'selected'
	| 'selected-dependency'
	| 'validating'
	| 'installing'
	| 'installed'

export interface BrowseInstallCapability {
	state: (hit: BrowseHit) => BrowseInstallState | null
	run: (hit: BrowseHit) => void | Promise<void>
	idleLabel?: (hit: BrowseHit) => string
	idleIcon?: Component
}

export interface BrowseToggle {
	id: string
	label: string
	model: Ref<boolean>
}

export interface BrowseLockedFilterMessages {
	gameVersion?: string
	modLoader?: string
	environment?: string
	syncButton?: string
	providedBy?: string
	gameVersionShaderMessage?: string
}

export interface CardAction {
	key: string
	label: string
	icon: Component
	iconClass?: string
	disabled?: boolean
	color?: 'brand' | 'red' | 'green'
	type?: 'standard' | 'outlined' | 'transparent'
	circular?: boolean
	tooltip?: string
	onClick: () => void | Promise<void>
}

export interface ServerModpackContent {
	name: string
	icon?: string
	onclick?: () => void
	showCustomModpackTooltip: boolean
}
