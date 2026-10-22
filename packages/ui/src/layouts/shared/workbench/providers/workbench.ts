import type { AddPanelPositionOptions, DockviewReadyEvent, GridviewReadyEvent } from 'dockview-vue'
import type { Component, ComputedRef, Ref, ShallowRef, Slots } from 'vue'

import type { ButtonMenuOption } from '#ui/components/base/buttons/types'
import { createContext } from '#ui/providers/create-context'

import type {
	WorkbenchDocuments,
	WorkbenchHost,
	WorkbenchLink,
	WorkbenchLinkOrientation,
	WorkbenchMode,
	WorkbenchPanel,
	WorkbenchPopoutRestore,
	WorkbenchRegion,
	WorkbenchSplitSide,
	WorkbenchToggleAction,
	WorkbenchToggleLocation,
	WorkbenchVariant,
} from '../types'

/** A tab of a dock: one of the workbench's panels, or a document. */
export interface WorkbenchDockTab {
	id: string
	title: string
	icon?: Component
	/** Whether it's the front tab of its group. */
	active: boolean
}

/** The workbench's layout state and the operations on it. */
export interface WorkbenchContext {
	/**
	 * The layout's slots: `panel-{id}` for each panel, `tab` for custom tab content and the
	 * toolbar's. Set by `WorkbenchLayout` when it renders.
	 */
	slots: Slots
	/** The workbench's root element, set by `WorkbenchLayout`. */
	root: ShallowRef<HTMLElement | null>
	mode: WorkbenchMode
	variant: Ref<WorkbenchVariant>
	setVariant: (variant: WorkbenchVariant) => void
	/** Whether the mode lets panels be popped out at all. */
	canUsePopouts: boolean
	/** Whether the user lets panels be popped out, where the mode allows it. */
	popoutsEnabled: Ref<boolean>
	setPopoutsEnabled: (enabled: boolean) => void
	/** Whether groups holding a single tab hide their tab bar, behind a handle pulling it down. */
	hideLoneTabBars: Ref<boolean>
	setHideLoneTabBars: (hide: boolean) => void
	/** Hides the tab bar of a panel's group, behind a handle pulling it down again. */
	hideTabBar: (id: string) => void
	canHideTabBar: (host: WorkbenchHost) => boolean
	/** Turns the order of a dock's groups around. */
	reverseGroups: (host: WorkbenchHost) => void
	canReverseGroups: (host: WorkbenchHost) => boolean
	/** What becomes of popped out panels when the page is opened again. */
	popoutRestore: Ref<WorkbenchPopoutRestore>
	setPopoutRestore: (restore: WorkbenchPopoutRestore) => void
	/** Where the user keeps the region toggles: in the layout menu only, or also in the tab bar. */
	toggleLocation: Ref<WorkbenchToggleLocation>
	setToggleLocation: (location: WorkbenchToggleLocation) => void
	/** Whether the region toggles hide their regions or collapse them to their tabs. */
	toggleAction: Ref<WorkbenchToggleAction>
	setToggleAction: (action: WorkbenchToggleAction) => void
	/** Whether the left and right region toggles show in the corners of the center's tab bars. */
	cornerToggles: ComputedRef<boolean>
	documents?: WorkbenchDocuments
	panels: ComputedRef<WorkbenchPanel[]>
	/** Panels the user closed, which can be opened again. */
	closedPanels: ComputedRef<WorkbenchPanel[]>
	/** The regions laid out, each kept even once emptied so panels can be dragged back into it. */
	usedRegions: ComputedRef<WorkbenchRegion[]>
	visible: Record<WorkbenchRegion, Ref<boolean>>
	/** How wide (or tall, for the bottom) each region is shown, 0 while hidden. */
	regionExtents: Record<WorkbenchRegion, Ref<number>>
	/** The center's top-left and top-right groups, whose tab bars hold the corner region toggles. */
	cornerGroups: Ref<{ left?: string; right?: string }>
	/** Tabs selected together, which are dragged as one. */
	selectedPanels: Ref<readonly string[]>
	/** Panels popped out into windows of their own. */
	detachedPanels: ComputedRef<string[]>
	hasPopouts: ComputedRef<boolean>
	/** Panels whose windows are being opened. */
	poppingOut: ComputedRef<string[]>
	popoutPanel: (id: string) => Promise<void> | undefined
	/** Returns a popped out panel to the main window. */
	dockPanel: (id: string) => void
	/** Returns every popped out panel to the main window. */
	closePopouts: () => void
	canPopout: (id: string) => boolean
	isDetached: (id: string) => boolean
	/** The dock a panel is in, which it may have been moved to. */
	regionOf: (panel: WorkbenchPanel) => WorkbenchHost
	isOpen: (id: string) => boolean
	/** Whether the panel is in view: open, in a shown region, and the front tab of its group. */
	isShown: (id: string) => boolean
	setRegionVisible: (region: WorkbenchRegion, visible: boolean) => void
	/** Brings a hidden or collapsed region back, or else hides or collapses it, as the settings choose. */
	toggleRegion: (region: WorkbenchRegion) => void
	/** Regions collapsed to a strip of their tabs. */
	collapsed: Record<WorkbenchRegion, Ref<boolean>>
	setRegionCollapsed: (region: WorkbenchRegion, collapsed: boolean) => void
	/** Each region's top-right group, whose tab bar holds the button collapsing the region. */
	regionCornerGroups: Ref<Partial<Record<WorkbenchRegion, string>>>
	/** The tabs a dock holds, in order, e.g. for its strip while collapsed. */
	dockTabs: (host: WorkbenchHost) => WorkbenchDockTab[]
	/** Brings a dock's tab to the front, showing (and expanding) its dock. */
	activateTab: (host: WorkbenchHost, id: string) => void
	/** Moves (or reopens) a panel into a dock, and shows it. */
	movePanel: (id: string, host: WorkbenchHost, position?: AddPanelPositionOptions) => void
	/** Moves a document into another dock, once its owner confirms it may be remounted. */
	moveDocument: (id: string, host: WorkbenchHost) => void
	/** The dock holding a document, if any does. */
	hostOfDocument: (id: string) => WorkbenchHost | undefined
	/** Splits a panel off into its own group beside the rest of its group. */
	/** Splits panels of a group off into a group of their own beside it. */
	splitPanels: (ids: string[], side: WorkbenchSplitSide) => void
	/** Whether the panel shares its group, so it can be split off. */
	canSplit: (ids: string[]) => boolean
	/** Pops several panels out together, into one window. */
	popoutPanels: (ids: string[]) => Promise<void> | undefined
	/** Closes a panel; a linked one is taken out of its linked tab first. */
	closePanel: (id: string) => void
	/** Panels linked into single tabs, each split between its panels. */
	links: ComputedRef<WorkbenchLink[]>
	/** Links a panel (or linked tab) with another, as one tab split between them. */
	linkPanels: (id: string, otherId: string) => void
	/** Takes a panel out of its linked tab, beside it. */
	unlinkPanel: (id: string) => void
	/** Undoes a link, its panels each becoming a tab of their own. */
	unlinkAll: (linkId: string) => void
	setLinkOrientation: (linkId: string, orientation: WorkbenchLinkOrientation) => void
	/** Turns the order of a linked tab's panels around. */
	reverseLink: (linkId: string) => void
	/** Keeps the shares of a linked tab its panels were resized to. */
	setLinkSizes: (linkId: string, sizes: number[]) => void
	/** Opens the panel if closed, shows its dock or window and brings it to the front of its group. */
	revealPanel: (id: string) => void
	/** Brings the first panel of every group to the front. */
	resetActiveTabs: () => void
	/** Restores the default sizes, visibility, placement and arrangement of every panel. */
	resetLayout: () => void
	onGridReady: (event: GridviewReadyEvent) => void
	onDockReady: (host: WorkbenchHost, event: DockviewReadyEvent) => void
	onDockDispose: (host: WorkbenchHost) => void
	onSashDoubleClick: (event: MouseEvent) => void
	/** Sets how the workbench's menus are shown, i.e. `WorkbenchLayout`'s context menu. */
	setMenuOpener: (open: (event: MouseEvent, options: ButtonMenuOption[]) => void) => void
	/** The menu of a panel's tab: moving, splitting, popping out and closing it. */
	openPanelMenu: (event: MouseEvent, id: string) => void
	/** The menu of a dock's tab bar: opening closed panels there, or hiding the region. */
	openRegionMenu: (event: MouseEvent, host: WorkbenchHost, panelId?: string) => void
	/** Options listing every panel, to open or close each. */
	panelsMenuOptions: ComputedRef<ButtonMenuOption[]>
	/** A "Move to" entry for a document's own tab menu, or `null` when it can't be moved. */
	documentMoveOption: (id: string) => ButtonMenuOption | null
}

export const [injectWorkbench, provideWorkbench] = createContext<WorkbenchContext>(
	'WorkbenchLayout',
	'workbench',
)

/** The dock a component is rendered in (its tabs, panels, header actions and watermark). */
export const [injectWorkbenchDock, provideWorkbenchDock] = createContext<WorkbenchHost>(
	'WorkbenchDock',
	'workbenchDock',
)

/** The workbench panel a component is rendered in. */
export interface WorkbenchPanelContext {
	id: string
	/** Whether the panel is shown, i.e. the active tab of its group, in a shown region. */
	visible: Readonly<Ref<boolean>>
	/** The window showing the panel: the main one, or the popout it was moved into. */
	window: Readonly<ShallowRef<Window>>
}

export const [injectWorkbenchPanel, provideWorkbenchPanel] = createContext<WorkbenchPanelContext>(
	'WorkbenchPanel',
	'workbenchPanel',
)
