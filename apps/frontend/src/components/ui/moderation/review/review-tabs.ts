import {
	BoxIcon,
	FileTextIcon,
	HistoryIcon,
	ImagesIcon,
	MessageIcon,
	SettingsIcon,
} from '@modrinth/assets'
import { type Component, defineAsyncComponent } from 'vue'

import type { ReviewTabId } from '~/services/moderation/review-layout'

import ReviewDescriptionTab from './ReviewDescriptionTab.vue'

export interface ReviewTabDef {
	id: ReviewTabId
	label: string
	icon: Component
	/** The section component rendered inside the panel. */
	component: Component
	/** Context helper to lazily load whenever this tab is opened. */
	requiresVersions?: boolean
}

/**
 * Maps each review tab to the existing project-section component. These are the same SFCs
 * the project page renders via `<NuxtPage>`; they read their data from
 * `injectProjectPageContext()`, which is provided in `[type]/[project].vue`, so they work
 * unchanged when rendered as descendants inside a review panel.
 */
export const REVIEW_TABS: Record<ReviewTabId, ReviewTabDef> = {
	description: {
		id: 'description',
		label: 'Description',
		icon: FileTextIcon,
		component: ReviewDescriptionTab,
	},
	gallery: {
		id: 'gallery',
		label: 'Gallery',
		icon: ImagesIcon,
		component: defineAsyncComponent(() => import('~/pages/[type]/[project]/gallery.vue')),
	},
	changelog: {
		id: 'changelog',
		label: 'Changelog',
		icon: HistoryIcon,
		component: defineAsyncComponent(() => import('~/pages/[type]/[project]/changelog.vue')),
		requiresVersions: true,
	},
	versions: {
		id: 'versions',
		label: 'Versions',
		icon: BoxIcon,
		component: defineAsyncComponent(() => import('~/pages/[type]/[project]/versions.vue')),
		requiresVersions: true,
	},
	thread: {
		id: 'thread',
		label: 'Moderation',
		icon: MessageIcon,
		component: defineAsyncComponent(() => import('~/pages/[type]/[project]/moderation.vue')),
	},
	settings: {
		id: 'settings',
		label: 'Settings',
		icon: SettingsIcon,
		component: defineAsyncComponent(() => import('./ReviewSettingsTab.vue')),
	},
}

export function reviewTab(id: ReviewTabId): ReviewTabDef {
	return REVIEW_TABS[id]
}
