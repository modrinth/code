import type { DockviewApi } from 'dockview-vue'
import type { Component } from 'vue'

/** The regions around the workbench's center, each holding its panels as dockable tabs. */
export const WORKBENCH_REGIONS = ['left', 'right', 'bottom'] as const

export type WorkbenchRegion = (typeof WORKBENCH_REGIONS)[number]

/** Every dock of the workbench: its center and the regions around it. */
export const WORKBENCH_HOSTS = ['center', ...WORKBENCH_REGIONS] as const

export type WorkbenchHost = (typeof WORKBENCH_HOSTS)[number]

/** A panel of the workbench, whose content is the host's `panel-{id}` slot. */
export interface WorkbenchPanel {
	id: string
	title: string
	region: WorkbenchHost
	icon?: Component
}

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
	/** Shown at the end of a group's tab bar, e.g. to open another document there. */
	headerActions?: Component
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
	/** The workbench's layout was reset, dropping every document; their panels should be re-added. */
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

/** Minimum size of the center, in both directions. */
export const WORKBENCH_CENTER_MINIMUM = 240
