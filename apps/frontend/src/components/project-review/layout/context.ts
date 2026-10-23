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
	/** Item counts shown in the panels' tabs, e.g. how many issues there are. */
	counts: ComputedRef<Partial<Record<ProjectReviewSlot, number>>>
	/** Brings a panel to the front of its group, showing its region (or its window, if popped out). */
	revealSlot: (slot: ProjectReviewSlot) => void
	/** Hides a panel shown in view, or else reveals it. */
	toggleSlot: (slot: ProjectReviewSlot) => void
	/**
	 * Toggles the settings: their panel while it's open, or else their floating window, which
	 * leaves whichever panel is in front where it is.
	 */
	toggleSettings: () => void
	/** Moves the settings out of their floating window into their panel. */
	settingsAsPanel: () => void
	/** Moves the settings out of their panel into their floating window. */
	settingsAsWindow: () => void
	openTab: (tab: ProjectReviewTab) => void
	/** Brings the first tab of every group to the front. */
	resetActiveTabs: () => void
	selectedTabs: Ref<readonly string[]>
	detachedTabs: ComputedRef<string[]>
	popoutTab: (tab: ProjectReviewTab) => void
	dockTab: (tab: ProjectReviewTab) => void
}

export const [injectProjectReviewContext, provideProjectReviewContext] =
	createContext<ProjectReviewContext>('ProjectReview')

export const [injectReviewSlot, provideReviewSlot] =
	createContext<Ref<ProjectReviewSlot>>('ProjectReviewSlot')

/** Whether the surrounding panel is currently shown, e.g. it is the active tab in its group. */
export const [injectReviewPanelVisible, provideReviewPanelVisible] = createContext<
	Readonly<Ref<boolean>>
>('ProjectReviewPanelVisible')
