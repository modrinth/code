import type { Component, MaybeRef, Ref } from 'vue'
import type { RouteLocationRaw } from 'vue-router'

import { createContext } from '#ui/providers/create-context'
import type { FilterValue, ServerContentEnvironment } from '#ui/utils/search'

import type { BrowseViewState } from '../composables/use-browse-view-state'
import type {
	BrowseDisplayMode,
	BrowseHit,
	BrowseInstallCapability,
	BrowseLockedFilterMessages,
	BrowseProjectTypeLink,
	BrowseSelection,
	BrowseTags,
	BrowseToggle,
	CardAction,
	ServerModpackContent,
} from '../types'

export interface BrowseManagerContext {
	view: BrowseViewState
	tags: Readonly<Ref<BrowseTags>>
	projectType: Readonly<Ref<string>>
	projectTypes?: Readonly<Ref<BrowseProjectTypeLink[]>>

	selection?: BrowseSelection
	install?: BrowseInstallCapability
	actions?: (hit: BrowseHit) => CardAction[]

	providedFilters?: Readonly<Ref<FilterValue[]>>
	hiddenFilterTypes?: Readonly<Ref<string[]>>
	lockedFilterMessages?: MaybeRef<BrowseLockedFilterMessages>
	toggles?: Readonly<Ref<BrowseToggle[]>>
	serverEnvironment?: {
		selected: Ref<ServerContentEnvironment | null>
		shown: Readonly<Ref<boolean>>
	}

	links: {
		project: (hit: BrowseHit) => string | RouteLocationRaw
		server: (hit: BrowseHit) => string | RouteLocationRaw
		organization: (organizationId: string) => string | RouteLocationRaw
	}

	serverPings?: Readonly<Ref<Record<string, number | undefined>>>
	serverModpackContent?: (hit: BrowseHit) => ServerModpackContent | undefined

	display?: {
		mode: Readonly<Ref<BrowseDisplayMode>>
		cycle: () => void
	}
	preferences?: {
		advancedFiltersCollapsed: Ref<boolean>
		dismissedPhotosensitivityWarning: Ref<boolean>
	}
	filtersMenuOpen?: Ref<boolean>
	offline?: Readonly<Ref<boolean>>
	loadingComponent?: Component

	onHover?: (hit: BrowseHit) => void
	onHoverEnd?: () => void
	onContextMenu?: (event: MouseEvent, hit: BrowseHit) => void
}

export const [injectBrowseManager, provideBrowseManager] = createContext<BrowseManagerContext>(
	'BrowsePageLayout',
	'browseManagerContext',
)
