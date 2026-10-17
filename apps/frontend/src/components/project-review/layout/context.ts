import { createContext } from '@modrinth/ui'
import type { ComputedRef, Ref } from 'vue'

import type { ProjectReviewSlot, ProjectReviewSlots, ProjectReviewTab } from './types'

/** The review's layout: its side panels and tabs, as laid out by the shared workbench. */
export interface ProjectReviewContext {
	slots: ProjectReviewSlots
	leftVisible: Ref<boolean>
	rightVisible: Ref<boolean>
	/** How wide the right panel is shown, 0 while hidden. */
	visibleRightWidth: Ref<number>
	toggleSidebar: (side: 'left' | 'right') => void
	/** Shows a side panel, or brings a tab to the front (in its own window, if popped out). */
	revealSlot: (slot: ProjectReviewSlot) => void
	openTab: (tab: ProjectReviewTab) => void
	/** Brings the first tab of every group to the front. */
	resetActiveTabs: () => void
	resetPanelLayout: () => void
	selectedTabs: Ref<readonly string[]>
	hasPopouts: ComputedRef<boolean>
	detachedTabs: ComputedRef<string[]>
	poppingOut: ComputedRef<string[]>
	popoutTab: (tab: ProjectReviewTab) => void
	dockTab: (tab: ProjectReviewTab) => void
	/** Returns every popped out tab to the main window. */
	closeExternalPanels: () => void
}

export const [injectProjectReviewContext, provideProjectReviewContext] =
	createContext<ProjectReviewContext>('ProjectReview')

export const [injectReviewSlot, provideReviewSlot] =
	createContext<Ref<ProjectReviewSlot>>('ProjectReviewSlot')

/** Whether the surrounding panel is currently shown, e.g. it is the active tab in its group. */
export const [injectReviewPanelVisible, provideReviewPanelVisible] = createContext<
	Readonly<Ref<boolean>>
>('ProjectReviewPanelVisible')
