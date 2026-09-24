import type { SerializedDockview } from 'dockview-vue'

import { projectReviewTabs } from './types'

const STORAGE_KEY = 'moderation.project-review.layout.v1'

export const workspacePanelSizes = {
	left: { default: 400, minimum: 200 },
	right: { default: 400, minimum: 200 },
	bottom: { default: 200, minimum: 50 },
	center: { minimum: 100 },
	tabs: { minimum: 100 },
} as const

export interface SavedWorkspaceLayout {
	leftWidth: number
	rightWidth: number
	bottomHeight: number
	bottomVisible?: boolean
	leftVisible: boolean
	rightVisible: boolean
	tabs: SerializedDockview
}

export function readWorkspaceLayout(): SavedWorkspaceLayout | undefined {
	try {
		const saved: SavedWorkspaceLayout | null = JSON.parse(
			localStorage.getItem(STORAGE_KEY) ?? 'null',
		)
		if (
			!saved ||
			![saved.leftWidth, saved.rightWidth, saved.bottomHeight].every(
				(size) => typeof size === 'number' && Number.isFinite(size) && size > 0,
			) ||
			(saved.bottomVisible !== undefined && typeof saved.bottomVisible !== 'boolean') ||
			typeof saved.leftVisible !== 'boolean' ||
			typeof saved.rightVisible !== 'boolean' ||
			!saved.tabs?.grid ||
			!saved.tabs.panels ||
			Object.keys(saved.tabs.panels).some(
				(tab) => !projectReviewTabs.includes(tab as (typeof projectReviewTabs)[number]),
			) ||
			saved.tabs.floatingGroups?.length ||
			saved.tabs.popoutGroups?.length ||
			saved.tabs.edgeGroups
		) {
			return
		}

		for (const tab of projectReviewTabs) {
			const panel = saved.tabs.panels[tab]
			if (tab === 'permissions' && !panel) continue
			if (
				panel?.id !== tab ||
				panel.contentComponent !== 'ProjectReviewPanel' ||
				panel.params?.tab !== tab ||
				panel.params?.slot !== tab
			) {
				return
			}
		}

		return saved
	} catch {
		return
	}
}

export function saveWorkspaceLayout(layout: SavedWorkspaceLayout) {
	try {
		localStorage.setItem(STORAGE_KEY, JSON.stringify(layout))
	} catch {
		// no-op
	}
}
