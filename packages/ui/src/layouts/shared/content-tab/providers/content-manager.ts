import type { Ref } from 'vue'
import type { RouteLocationRaw } from 'vue-router'

import type { ButtonMenuOption } from '#ui/components/base/buttons'
import type { UpdateAllSelection } from '#ui/components/modal/update-all-modal/update-all-modal-types'
import { createContext } from '#ui/providers/create-context'

import type {
	ContentActionWarning,
	ContentConfirmResult,
	ContentItem,
	ContentOwner,
	ContentPermissions,
	ContentSide,
	ContentType,
	ManagedContent,
} from '../types'

export interface ContentDependencyWarning {
	items: ContentItem[]
	dependents: Array<{
		item: ContentItem
		dependencies: ContentItem[]
	}>
}

export interface ContentManagerContext {
	items: Readonly<Ref<ContentItem[]>>
	loading: Readonly<Ref<boolean>>
	error: Readonly<Ref<Error | null>>
	refresh: () => Promise<void>

	/** One tab per entry, in this order. */
	types: Readonly<Ref<ContentType[]>>
	target: Readonly<Ref<{ kind: 'instance' | 'server'; gameVersion: string; loader: string }>>
	/** Whole-target lock, such as an instance mid-install. Null when idle. */
	busy: Readonly<Ref<{ message?: string } | null>>

	permissions: (item: ContentItem) => ContentPermissions
	canAdd: Readonly<Ref<boolean>>
	addDeniedReason?: string
	warn?: (action: 'remove' | 'disable', items: ContentItem[]) => ContentActionWarning | null
	confirm?: (
		action: 'enable' | 'disable' | 'remove',
		items: ContentItem[],
	) => Promise<ContentConfirmResult>
	skipOptionalConfirms?: Readonly<Ref<boolean>>

	setEnabled: (items: ContentItem[], enabled: boolean) => Promise<void>
	remove: (items: ContentItem[]) => Promise<void>
	findDependents?: (items: ContentItem[]) => Promise<ContentDependencyWarning | null>
	setEnabledFor?: (item: ContentItem, side: ContentSide, enabled: boolean) => Promise<void>

	/** Each opens the shell's own version picker. */
	updateItem?: (item: ContentItem) => void
	switchVersion?: (item: ContentItem) => void
	/** Applies the choices made in the shared update-all modal. */
	applyUpdates?: (
		selections: UpdateAllSelection[],
		onProgress?: (completed: number) => void,
	) => Promise<void>
	updatesRunInBackground?: boolean

	browse: (type?: ContentType) => void
	upload: () => void

	managed: Readonly<Ref<ManagedContent | null>>

	links: {
		project: (item: ContentItem) => string | RouteLocationRaw | undefined
		version: (item: ContentItem) => string | RouteLocationRaw | undefined
		owner: (owner: ContentOwner) => ContentOwner['link']
	}
	menu?: (item: ContentItem) => ButtonMenuOption[]
	share?: (items: ContentItem[], format: 'names' | 'file-names' | 'urls' | 'markdown') => void
	persistKey?: string

	/** Noun for the empty state, selection bar and confirm modals. */
	contentTypeLabel: Readonly<Ref<string>>
	/** Route query param the search box syncs to, so back navigation restores the search. */
	searchQueryParam?: string
	showSharedContentFilter?: Readonly<Ref<boolean>>
	/** When false, rows stay interactive while their own or a bulk action is in flight. */
	disableWhileMutating?: boolean
}

export const [injectContentManager, provideContentManager] = createContext<ContentManagerContext>(
	'ContentPageLayout',
	'contentManagerContext',
)
