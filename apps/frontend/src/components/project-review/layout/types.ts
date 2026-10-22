import type { VNode } from 'vue'

export const projectReviewTabs = [
	'description',
	'gallery',
	'disclosures',
	'versions',
	'permissions',
	'tech-review',
] as const

export type ProjectReviewTab = (typeof projectReviewTabs)[number]

/**
 * The review's panels around its tabs: the project, the review's rulings and verdict, the
 * conversation about it, and the review's settings.
 */
export const projectReviewSidePanels = [
	'project',
	'rulings',
	'issues',
	're-review',
	'thread',
	'settings',
] as const

export type ProjectReviewSidePanel = (typeof projectReviewSidePanels)[number]
export type ProjectReviewSlot = ProjectReviewTab | ProjectReviewSidePanel
export type ProjectReviewSlots = {
	/** `footer` fills the workbench's toolbar, beside its layout menu. */
	[Key in Exclude<ProjectReviewSlot, 'settings'> | 'footer']?: () => VNode[]
} & {
	/** The settings, shown in their panel or, when `floating`, in a window above the review. */
	settings?: (props: { floating?: boolean }) => VNode[]
	/**
	 * Where the archive shown in the archive panels comes from, atop its details. `projectShown`
	 * tells whether the project's own panel is shown, so the project needn't be repeated.
	 */
	'archive-source'?: (props: { projectShown: boolean }) => VNode[]
}
