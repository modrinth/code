import type { Labrinth } from '@modrinth/api-client'
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

	types: Readonly<Ref<ContentType[]>>
	target: Readonly<Ref<{ kind: 'instance' | 'server'; gameVersion: string; loader: string }>>
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

	installVersion?: (item: ContentItem, version: Labrinth.Versions.v2.Version) => Promise<void>
	fetchProjectVersions?: (projectId: string) => Promise<Labrinth.Versions.v2.Version[]>
	fetchVersion?: (versionId: string) => Promise<Labrinth.Versions.v2.Version>
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

	searchQueryParam?: string
	showSharedContentFilter?: Readonly<Ref<boolean>>
	disableWhileMutating?: boolean
}

export const [injectContentManager, provideContentManager] = createContext<ContentManagerContext>(
	'ContentPageLayout',
	'contentManagerContext',
)
