import type { Labrinth } from '@modrinth/api-client'
import type { RouteLocationRaw } from 'vue-router'

import type { ButtonMenuOption } from '#ui/components/base/buttons'

export type ContentCardProject = Pick<
	Labrinth.Projects.v2.Project,
	'id' | 'slug' | 'title' | 'icon_url'
> &
	Partial<Pick<Labrinth.Projects.v2.Project, 'license' | 'categories' | 'additional_categories'>>

export type ContentCardVersion = Pick<Labrinth.Versions.v2.Version, 'id' | 'version_number'> & {
	file_name: string
	date_published?: string
}

export interface ContentOwner {
	id: string
	name: string
	avatar_url?: string
	type: 'user' | 'organization'
	link?: string | RouteLocationRaw | (() => void)
}

export interface ContentSource {
	project: ContentCardProject
	link?: string | RouteLocationRaw | (() => void)
}

export type ClientWarningType = 'retained' | 'depends' | 'environment'

export type ContentSourceKind =
	| 'local'
	| 'modrinth_modpack'
	| 'server_project'
	| 'modrinth_hosting'
	| 'imported_modpack'
	| 'shared_instance'

export interface ContentActionWarning {
	admonitionHeader: string
	admonitionBody: string
	actionLabel: string
}

export interface EmbeddedContentMetadata {
	name?: string | null
	version?: string | null
	icon_path?: string | null
	icon_url?: string | null
}

export interface ContentCardTableItem {
	id: string
	project: ContentCardProject
	projectLink?: string | RouteLocationRaw
	version?: ContentCardVersion
	versionLink?: string | RouteLocationRaw
	owner?: ContentOwner
	source?: ContentSource
	external?: boolean
	enabled?: boolean
	locked?: boolean
	disabled?: boolean
	disabledTooltip?: string | null
	toggleDisabled?: boolean
	toggleDisabledTooltip?: string | null
	hideToggle?: boolean
	installing?: boolean
	installProgress?: number | null
	hasUpdate?: boolean
	isClientOnly?: boolean
	clientWarning?: ClientWarningType | null
	hideDelete?: boolean
	hideSwitchVersion?: boolean
	overflowOptions?: ButtonMenuOption[]
}

export type ContentCardTableSortColumn = 'project' | 'version'
export type ContentCardTableSortDirection = 'asc' | 'desc'

export interface BulkOperationStatus {
	message?: string
	progress?: number
	total?: number
	waiting?: boolean
}

/** Content item returned from the app backend API - maps to ContentCardTableItem for display */
export interface ContentItem extends Omit<
	ContentCardTableItem,
	'id' | 'projectLink' | 'disabled' | 'overflowOptions'
> {
	id: string
	file_name: string
	file_path?: string
	size?: number
	project_type: string
	has_update: boolean
	update_version_id: string | null
	date_added?: string
	environment?: Labrinth.Projects.v3.Environment
	pack_client_retained?: boolean
	pack_client_depends?: boolean
	installing?: boolean
	installProgress?: number | null
	source_kind?: ContentSourceKind | null
	external?: boolean
	external_url?: string
	embedded_metadata?: EmbeddedContentMetadata | null
}

export type ManagedContentProject = Pick<
	Labrinth.Projects.v2.Project,
	'id' | 'slug' | 'title' | 'icon_url'
> & {
	filename?: string | null
}

export type ManagedContentVersion = Pick<
	Labrinth.Versions.v2.Version,
	'id' | 'version_number' | 'date_published'
>

export type ManagedContentSummaryType = 'mod' | 'plugin' | 'datapack' | 'resourcepack' | 'shader'

export interface ManagedContentManager {
	name: string
	iconUrl?: string
	link?: string | RouteLocationRaw
}

export interface ManagedContentSummaryItem {
	type: ManagedContentSummaryType
	count: number
}

interface ManagedContentCardBase {
	manager: ManagedContentManager
	summary?: ManagedContentSummaryItem[]
	installing?: boolean
}

export type ManagedContentCardData =
	| (ManagedContentCardBase & {
			kind: 'modpack'
			versionNumber?: string
			versionLink?: string | RouteLocationRaw
			updatedAt?: string
	  })
	| (ManagedContentCardBase & {
			kind: 'server' | 'shared-instance'
			syncedAt?: number
			updateAvailable: boolean
	  })
