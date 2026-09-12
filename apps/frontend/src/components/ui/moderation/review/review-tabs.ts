import {
	BoxIcon,
	CopyrightIcon,
	FileTextIcon,
	HistoryIcon,
	ImagesIcon,
	KeyIcon,
	LinkIcon,
	MessageIcon,
	ScaleIcon,
	SettingsIcon,
	ShieldIcon,
	TagsIcon,
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
	/** Only offered for modpack projects (matches the checklist stage's own gating). */
	modpackOnly?: boolean
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
		component: defineAsyncComponent(() => import('./ReviewVersionsTab.vue')),
		requiresVersions: true,
	},
	tags: {
		id: 'tags',
		label: 'Tags',
		icon: TagsIcon,
		component: defineAsyncComponent(() => import('~/pages/[type]/[project]/settings/tags.vue')),
	},
	license: {
		id: 'license',
		label: 'License',
		icon: CopyrightIcon,
		component: defineAsyncComponent(() => import('~/pages/[type]/[project]/settings/license.vue')),
	},
	links: {
		id: 'links',
		label: 'Links',
		icon: LinkIcon,
		component: defineAsyncComponent(() => import('~/pages/[type]/[project]/settings/links.vue')),
	},
	disclosures: {
		id: 'disclosures',
		label: 'Disclosures',
		icon: ShieldIcon,
		component: defineAsyncComponent(
			() => import('~/pages/[type]/[project]/settings/disclosures.vue'),
		),
	},
	permissions: {
		id: 'permissions',
		label: 'Permissions',
		icon: KeyIcon,
		component: defineAsyncComponent(
			() => import('~/pages/[type]/[project]/settings/permissions.vue'),
		),
		modpackOnly: true,
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
	moderation_settings: {
		id: 'moderation_settings',
		label: 'Moderation Settings',
		icon: ScaleIcon,
		component: defineAsyncComponent(() => import('../settings/ModerationSettings.vue'))
	}
}

export function isReviewTab(id: string): id is ReviewTabId {
	return id in REVIEW_TABS
}

/** Safe lookup — falls back to Description for stale/unknown ids left in persisted state. */
export function reviewTab(id: string): ReviewTabDef {
	return REVIEW_TABS[id as ReviewTabId] ?? REVIEW_TABS.description
}

/**
 * Tab ids offered in the tab-strip menus / collapsed rail for this project. Drops
 * `modpackOnly` tabs for non-modpacks (they can still be opened by the checklist when their
 * stage is visible, and stay valid in persisted state).
 */
export function selectableReviewTabs(order: readonly ReviewTabId[], isModpack: boolean) {
	return order.filter((id) => !REVIEW_TABS[id].modpackOnly || isModpack)
}
