import type { AddPanelPositionOptions, DockviewReadyEvent, GridviewReadyEvent } from 'dockview-vue'
import type { ComputedRef, Ref, Slots } from 'vue'

import type { ButtonMenuOption } from '#ui/components/base/buttons/types'
import { createContext } from '#ui/providers/create-context'

import type { WorkbenchDocuments, WorkbenchHost, WorkbenchPanel, WorkbenchRegion } from '../types'

/** The workbench's layout state and the operations on it. */
export interface WorkbenchLayoutState {
	/** The host's slots: `panel-{id}` for each panel. */
	slots: Slots
	documents?: WorkbenchDocuments
	panels: ComputedRef<WorkbenchPanel[]>
	/** Panels the user closed, which can be opened again. */
	closedPanels: ComputedRef<WorkbenchPanel[]>
	/** The regions laid out, each kept even once emptied so panels can be dragged back into it. */
	usedRegions: ComputedRef<WorkbenchRegion[]>
	visible: Record<WorkbenchRegion, Ref<boolean>>
	/** The dock a panel is in, which it may have been moved to. */
	regionOf: (panel: WorkbenchPanel) => WorkbenchHost
	isOpen: (id: string) => boolean
	setRegionVisible: (region: WorkbenchRegion, visible: boolean) => void
	toggleRegion: (region: WorkbenchRegion) => void
	/** Moves (or reopens) a panel into a dock, and shows it. */
	movePanel: (id: string, host: WorkbenchHost, position?: AddPanelPositionOptions) => void
	/** Moves a document into another dock, once its owner confirms it may be remounted. */
	moveDocument: (id: string, host: WorkbenchHost) => void
	/** The dock holding a document, if any does. */
	hostOfDocument: (id: string) => WorkbenchHost | undefined
	/** Splits a panel off into its own group beside the rest of its group. */
	splitPanel: (id: string, side: 'right' | 'bottom') => void
	/** Whether the panel shares its group, so it can be split off. */
	canSplit: (id: string) => boolean
	closePanel: (id: string) => void
	/** Opens the panel if closed, shows its dock and brings it to the front of its group. */
	revealPanel: (id: string) => void
	/** Restores the default sizes, visibility, placement and arrangement of every panel. */
	resetLayout: () => void
	onGridReady: (event: GridviewReadyEvent) => void
	onDockReady: (host: WorkbenchHost, event: DockviewReadyEvent) => void
	onDockDispose: (host: WorkbenchHost) => void
}

export interface WorkbenchContext extends WorkbenchLayoutState {
	/** The menu of a panel's tab: moving, splitting and closing it, or opening closed panels. */
	openPanelMenu: (event: MouseEvent, id: string) => void
	/** The menu of a dock's tab bar: opening closed panels there, or hiding the region. */
	openRegionMenu: (event: MouseEvent, host: WorkbenchHost) => void
	/** The menu listing every panel, to open or close each. */
	openPanelsMenu: (event: MouseEvent) => void
	/** A "Move to" entry for a document's own tab menu, or `null` when it isn't in a dock. */
	documentMoveOption: (id: string) => ButtonMenuOption | null
}

export const [injectWorkbench, provideWorkbench] = createContext<WorkbenchContext>(
	'WorkbenchLayout',
	'workbench',
)
