import { createContext } from '@modrinth/ui'

import type { InstanceCard } from '@/features/instances/use-instance-actions'
import type { UninstalledSharedInstance } from '@/features/library/shared-instances/types'

import type { useInstanceGroupDrag } from './composables/use-instance-group-drag'
import type { useLibraryGroups } from './composables/use-library-groups'
import type { useLibrarySelection } from './composables/use-library-selection'
import type { useLibraryView } from './composables/use-library-view'

export type LibraryViewContext = ReturnType<typeof useLibraryView>
export type LibrarySelectionContext = ReturnType<typeof useLibrarySelection>
export type LibraryGroupsContext = ReturnType<typeof useLibraryGroups>
export type InstanceGroupDragContext = ReturnType<typeof useInstanceGroupDrag>

export interface LibraryActionsContext {
	openGroupInstancesModal: (groupId: string) => void
	openInstanceContextMenu: (event: MouseEvent, item: InstanceCard, groupId: string) => void
	openSharedInstanceContextMenu: (event: MouseEvent, instance: UninstalledSharedInstance) => void
}

export const [injectLibraryView, provideLibraryView] = createContext<LibraryViewContext>(
	'LibrarySection',
	'libraryView',
)

export const [injectLibrarySelection, provideLibrarySelection] =
	createContext<LibrarySelectionContext>('LibrarySection', 'librarySelection')

export const [injectLibraryGroups, provideLibraryGroups] = createContext<LibraryGroupsContext>(
	'LibrarySection',
	'libraryGroups',
)

export const [injectInstanceGroupDrag, provideInstanceGroupDrag] =
	createContext<InstanceGroupDragContext>('LibrarySection', 'libraryInstanceGroupDrag')

export const [injectLibraryActions, provideLibraryActions] = createContext<LibraryActionsContext>(
	'LibrarySection',
	'libraryActions',
)
