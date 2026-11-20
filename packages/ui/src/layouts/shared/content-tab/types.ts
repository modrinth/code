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
	label?: string
}

export type ClientWarningType = 'retained' | 'depends' | 'environment'
export type ContentWarningType = ClientWarningType | 'server-only' | 'unknown-environment'

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

export type ContentSide = 'server' | 'player'

export interface ContentEnabledForState {
	server: boolean
	player: boolean
	locked: boolean
	lockedTooltip?: string
	disabledSides?: ContentSide[]
	warningTooltip?: string | null
	warningKind?: ContentWarningType | null
}

export interface ContentCardEmbeddedIcon {
	queryKey: readonly unknown[]
	queryFn: () => Promise<Blob>
	fallbackUrl?: string | null
}

export interface ContentCardTableItem {
	id: string
	project: ContentCardProject
	projectType?: string
	projectLink?: string | RouteLocationRaw
	version?: ContentCardVersion
	versionLink?: string | RouteLocationRaw
	owner?: ContentOwner
	source?: ContentSource
	external?: boolean
	externalFile?: boolean
	overrideFile?: boolean
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
	clientWarning?: ClientWarningType | null
	synced?: boolean
	syncUpdatePending?: boolean
	hideDelete?: boolean
	hideSwitchVersion?: boolean
	overflowOptions?: ButtonMenuOption[]
	enabledFor?: ContentEnabledForState
	embeddedIcon?: ContentCardEmbeddedIcon
}

export interface BulkOperationStatus {
	message?: string
	progress?: number
	total?: number
	waiting?: boolean
}

export type ContentType = 'mod' | 'plugin' | 'resourcepack' | 'datapack' | 'shader'

export interface ContentItem {
	id: string
	project_type: ContentType
	file_name: string
	file_path?: string
	size?: number
	date_added?: string
	project?: ContentCardProject
	version?: ContentCardVersion
	owner?: ContentOwner
	embedded_metadata?: EmbeddedContentMetadata | null
	embeddedIcon?: ContentCardEmbeddedIcon
	enabled: boolean
	locked?: boolean
	has_update: boolean
	update_version_id: string | null
	installing?: boolean
	installProgress?: number | null
	source_kind?: ContentSourceKind | null
	synced_pack?: {
		id: string
		instance_ids: string[]
		update_pending: boolean
	} | null
	environment?: Labrinth.Projects.v3.Environment
	pack_client_retained?: boolean
	pack_client_depends?: boolean
	external?: boolean
	external_url?: string
	source?: ContentSource
	enabledFor?: ContentEnabledForState
}

export interface ContentPermissions {
	toggle: boolean
	remove: boolean
	changeVersion: boolean
	reason?: string
}

export type ContentConfirmResult = 'confirmed' | 'cancelled' | 'unhandled'

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
			canSwitchVersion?: boolean
	  })
	| (ManagedContentCardBase & {
			kind: 'server' | 'shared-instance'
			syncedAt?: number
			updateAvailable: boolean
	  })

export interface ManagedContentUpdate {
	currentVersionId: string | null
	initialVersionId?: string
	external?: boolean
	fetchVersions: () => Promise<Labrinth.Versions.v2.Version[]>
	install: (version: Labrinth.Versions.v2.Version) => Promise<void>
}

export interface ManagedContent {
	card: ManagedContentCardData
	locked: boolean
	disabledReason?: string
	view?: () => void
	update?: ManagedContentUpdate
	primaryAction?: (event?: MouseEvent) => void
	openSettings?: () => void
	unlink?: () => void
	prefetchView?: () => void
	prefetchPrimaryAction?: () => void
}
