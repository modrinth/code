import type { ComputedRef, Ref } from 'vue'
import type { RouteLocationRaw } from 'vue-router'

import type { Option as OverflowMenuOption } from '../components/base/OverflowMenu.vue'
import type {
	ContentCardTableItem,
	ContentItem,
	ContentModpackCardCategory,
	ContentModpackCardProject,
	ContentModpackCardVersion,
	ContentOwner,
} from '../components/instances/types'
import { createContext } from '.'

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

export interface UploadState {
	isUploading: boolean
	currentFileName: string | null
	currentFileProgress: number
	completedFiles: number
	totalFiles: number
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

	// Identity & labelling
	getItemId: (item: ContentItem) => string
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

	// Update support (optional per-platform)
	hasUpdateSupport: boolean
	updateItem?: (id: string) => void
	bulkUpdateItem?: (item: ContentItem) => Promise<void>

	// Modpack actions (optional)
	updateModpack?: () => void
	viewModpackContent?: () => void
	unlinkModpack?: () => void

	// Per-item overflow menu (optional)
	getOverflowOptions?: (item: ContentItem) => OverflowMenuOption[]

	// Share support (optional — when undefined, share button becomes hidden entirely)
	shareItems?: (items: ContentItem[], format: 'names' | 'file-names' | 'urls' | 'markdown') => void

	// Upload progress (optional)
	uploadState?: Ref<UploadState> | ComputedRef<UploadState>

	// Deletion context (controls modal variant)
	deletionContext?: 'instance' | 'server'

	// Table item mapping (link generation differs per platform)
	mapToTableItem: (item: ContentItem) => ContentCardTableItem
}

export const [injectContentManager, provideContentManager] = createContext<ContentManagerContext>(
	'ContentPageLayout',
	'contentManagerContext',
)
