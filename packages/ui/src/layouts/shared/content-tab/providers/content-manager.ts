import type { ComputedRef, Ref } from 'vue'
import type { RouteLocationRaw } from 'vue-router'

import type { Option as OverflowMenuOption } from '#ui/components/base/OverflowMenu.vue'
import { createContext } from '#ui/providers/create-context'

import type {
	BulkOperationStatus,
	ContentActionWarningMode,
	ContentCardTableItem,
	ContentItem,
	ContentModpackCardCategory,
	ContentModpackCardProject,
	ContentModpackCardVersion,
	ContentOwner,
} from '../types'

export interface ContentModpackData {
	project: ContentModpackCardProject
	projectLink?: string | RouteLocationRaw
	version?: ContentModpackCardVersion
	versionLink?: string | RouteLocationRaw
	owner?: ContentOwner
	categories: ContentModpackCardCategory[]
	hasUpdate: boolean
	disabled?: boolean
	disabledText?: string
}

export interface ContentDependencyWarning {
	items: ContentItem[]
	dependents: Array<{
		item: ContentItem
		dependencies: ContentItem[]
	}>
}

export interface ContentManagerContext {
	// Data
	items: Ref<ContentItem[]> | ComputedRef<ContentItem[]>
	loading: Ref<boolean>
	error: Ref<Error | null>

	// Modpack
	modpack: Ref<ContentModpackData | null> | ComputedRef<ContentModpackData | null>
	isPackLocked: Ref<boolean> | ComputedRef<boolean>

	// Guards
	isBusy: Ref<boolean> | ComputedRef<boolean>
	busyMessage?: Ref<string | null> | ComputedRef<string | null>
	skipNonEssentialWarnings?: Ref<boolean> | ComputedRef<boolean>
	disableAddContent?: Ref<boolean> | ComputedRef<boolean>
	disableAddContentTooltip?: string

	// Labelling
	contentTypeLabel: Ref<string> | ComputedRef<string>

	// Core actions
	toggleEnabled: (item: ContentItem) => Promise<void>
	deleteItem: (item: ContentItem) => Promise<void>
	refresh: () => Promise<void>
	browse: () => void
	uploadFiles: () => void

	// Bulk actions (optional — when provided, used instead of one-by-one loops)
	bulkDeleteItems?: (items: ContentItem[]) => Promise<void>
	bulkEnableItems?: (items: ContentItem[]) => Promise<void>
	bulkDisableItems?: (items: ContentItem[]) => Promise<void>
	canDeleteItem?: (item: ContentItem) => boolean
	getDeleteWarningMode?: (items: ContentItem[]) => ContentActionWarningMode
	getDisableWarningMode?: (items: ContentItem[]) => ContentActionWarningMode
	getDeleteDependencyWarning?: (
		items: ContentItem[],
	) => ContentDependencyWarning | null | Promise<ContentDependencyWarning | null>

	// Update support (optional per-platform)
	hasUpdateSupport: boolean
	updateItem?: (id: string) => void
	bulkUpdateAll?: (onProgress?: (status: BulkOperationStatus) => void) => Promise<void>
	bulkUpdateItem?: (item: ContentItem) => Promise<void>
	bulkUpdateItems?: (items: ContentItem[]) => Promise<void>

	// Modpack actions (optional)
	updateModpack?: () => void
	viewModpackContent?: () => void
	unlinkModpack?: () => void
	openSettings?: () => void

	// Switch version (optional)
	switchVersion?: (item: ContentItem) => void

	// Per-item overflow menu (optional)
	getOverflowOptions?: (item: ContentItem) => OverflowMenuOption[]

	// Share support (optional — when undefined, share button becomes hidden entirely)
	shareItems?: (items: ContentItem[], format: 'names' | 'file-names' | 'urls' | 'markdown') => void

	// Stable per-row identity. ContentItem.id can be a content hash, so it is not always unique.
	getItemId?: (item: ContentItem) => string

	// Bulk operation guard — set by layout, checked by providers to suppress refreshes
	isBulkOperating?: Ref<boolean>

	// Deletion context (controls modal variant)
	deletionContext?: 'instance' | 'server'

	// Table item mapping (link generation differs per platform)
	mapToTableItem: (item: ContentItem) => ContentCardTableItem

	// Filter persistence key — when set, selected filters are saved/restored via sessionStorage
	filterPersistKey?: string
}

export const [injectContentManager, provideContentManager] = createContext<ContentManagerContext>(
	'ContentPageLayout',
	'contentManagerContext',
)
