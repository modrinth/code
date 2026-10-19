import type { DockviewApi, DockviewGroupPanel } from 'dockview-vue'
import type { Component } from 'vue'

/** The regions around the workbench's center, each holding its panels as dockable tabs. */
export const WORKBENCH_REGIONS = ['left', 'right', 'bottom'] as const

export type WorkbenchRegion = (typeof WORKBENCH_REGIONS)[number]

/** Every dock of the workbench: its center and the regions around it. */
export const WORKBENCH_HOSTS = ['center', ...WORKBENCH_REGIONS] as const

export type WorkbenchHost = (typeof WORKBENCH_HOSTS)[number]

/**
 * How the workbench looks: `square` for flat, editor-like tab strips, `rounded` for pill shaped
 * tabs in rounded bars, as the moderation view has.
 */
export const WORKBENCH_VARIANTS = ['square', 'rounded'] as const

export type WorkbenchVariant = (typeof WORKBENCH_VARIANTS)[number]

/** Where the region toggles sit: in the layout menu, or also in the corners of the center's tab bars. */
export type WorkbenchToggleLocation = 'menu' | 'tabs'

/** A panel of the workbench, whose content is the host's `panel-{id}` slot. */
export interface WorkbenchPanel {
	id: string
	title: string
	region: WorkbenchHost
	icon?: Component
	/** Pads the panel's content. */
	padded?: boolean
	/** Whether the panel scrolls its content (the default), rather than leaving that to it. */
	scroll?: boolean
	minimumHeight?: number
	/**
	 * Starts closed until opened (e.g. from the panels menu or by the host), for panels only
	 * wanted now and then. Needs a mode whose panels can be closed.
	 */
	closedByDefault?: boolean
}

export interface WorkbenchPopoutOptions {
	/** An empty page of the same origin that popout windows load. */
	url: string
	/** Docks whose panels can be popped out into their own windows. Defaults to the center. */
	docks?: WorkbenchHost[]
	/** Called for each popout window once it opens; the function returned runs once it closes. */
	onWindow?: (window: Window) => () => void
}

/**
 * How much of the layout users can change. Archive browsing hands them every panel to arrange
 * freely, while a review keeps its side panels fixed and only rearranges the center's tabs.
 */
export interface WorkbenchMode {
	/** Regions laid out around the center. */
	regions: WorkbenchRegion[]
	/** Panels can be moved between docks, by dragging or from their menu. */
	movePanels: boolean
	/** Panels can be closed, and reopened from the menus. */
	closePanels: boolean
	/** Docks can be split into several groups. */
	splitPanels: boolean
	/** The toolbar row, with the region toggles, panels menu and layout reset. */
	toolbar: boolean
	/** Region toggles in the outer corners of the center's tab bars, for workbenches without a toolbar. */
	cornerToggles: boolean
	/** Docks shown without tab bars, each holding a single panel. */
	headerless: WorkbenchHost[]
	/** Tabs can be selected together (shift or ctrl click) and dragged as one. */
	tabSelection: boolean
	popouts?: WorkbenchPopoutOptions
}

export const WORKBENCH_MODES = {
	explorer: {
		regions: [...WORKBENCH_REGIONS],
		movePanels: true,
		closePanels: true,
		splitPanels: true,
		toolbar: true,
		cornerToggles: false,
		headerless: [],
		tabSelection: false,
	},
	review: {
		regions: ['left', 'right'],
		movePanels: false,
		closePanels: false,
		splitPanels: true,
		toolbar: false,
		cornerToggles: true,
		headerless: ['left', 'right'],
		tabSelection: true,
	},
} satisfies Record<string, WorkbenchMode>

export type WorkbenchModeName = keyof typeof WORKBENCH_MODES

/**
 * Panels the workbench hosts on behalf of someone else, e.g. the files layout's tabs. They're
 * created and closed by their owner, which also renders their tabs, but they can live in (and be
 * dragged between) any of the workbench's docks, and are kept with its saved layout.
 */
export interface WorkbenchDocuments {
	/** Content components of the documents' panels, by name. */
	components: Record<string, Component>
	/** Tab components of the documents' panels, by name. */
	tabComponents: Record<string, Component>
	/** Shown at the end of a group's tab bar, e.g. a list of every open document. */
	headerActions?: Component
	/** Opening a new document in a group, offered first by each tab bar's add menu. */
	create?: {
		label: () => string
		icon?: Component
		open: (group: DockviewGroupPanel) => void
		/** Whether it's offered at the moment, e.g. per the user's settings. */
		shown?: () => boolean
	}
	/** Shown in the center while it holds no panel at all. */
	watermark?: Component
	/**
	 * Asked before a document moves to another dock, which remounts it. Resolves `true` once it
	 * is safe to move, e.g. after any unsaved changes were saved or discarded.
	 */
	confirmMove?: (id: string) => Promise<boolean>
	/** A dock came up (after its saved layout was restored), the center being the primary one. */
	attach: (api: DockviewApi, host: WorkbenchHost) => void
	detach: (api: DockviewApi) => void
	/** The workbench's layout was replaced, dropping documents; their panels should be re-added. */
	restore: () => void
}

export interface WorkbenchRegionSize {
	/** Width (or height, for the bottom region) the region opens with. */
	default: number
	minimum: number
}

export type WorkbenchRegionSizes = Record<WorkbenchRegion, WorkbenchRegionSize>

export const DEFAULT_WORKBENCH_REGION_SIZES: WorkbenchRegionSizes = {
	left: { default: 320, minimum: 200 },
	right: { default: 360, minimum: 220 },
	bottom: { default: 220, minimum: 100 },
}

/** Minimum size of the center, in both directions, unless a workbench sets its own. */
export const WORKBENCH_CENTER_MINIMUM = 240
