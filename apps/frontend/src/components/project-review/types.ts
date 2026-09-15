import type { VNode } from 'vue'

export const projectReviewTabs = [
	'description',
	'gallery',
	'disclosures',
	'versions',
	'history',
	'tech-review',
] as const

export type ProjectReviewTab = (typeof projectReviewTabs)[number]
export type ProjectReviewSlot = ProjectReviewTab | 'left' | 'right' | 'bottom'
export type ProjectReviewSlots = { [Key in ProjectReviewSlot]?: () => VNode[] }
