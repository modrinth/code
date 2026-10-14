import type { SerializedDockview } from 'dockview-vue'

import { type ProjectReviewTab, projectReviewTabs } from './types'

const STORAGE_KEY = 'moderation.project-review.layout.v1'

export const workspacePanelSizes = {
	left: { default: 400, minimum: 200 },
	right: { default: 400, minimum: 200 },
	bottom: { default: 200, minimum: 50 },
	center: { minimum: 20 },
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
	tabLayouts?: Record<string, SerializedDockview>
}

export function workspaceTabLayoutKey(tabs: readonly ProjectReviewTab[]) {
	return projectReviewTabs.filter((tab) => tabs.includes(tab)).join(',')
}

function isValidTabLayout(layout: SerializedDockview) {
	if (
		!layout?.grid ||
		!layout.panels ||
		Object.keys(layout.panels).some(
			(tab) => !projectReviewTabs.includes(tab as ProjectReviewTab),
		) ||
		layout.floatingGroups?.length ||
		layout.popoutGroups?.length ||
		layout.edgeGroups
	) {
		return false
	}

	for (const tab of projectReviewTabs) {
		const panel = layout.panels[tab]
		if (tab === 'permissions' && !panel) continue
		if (
			panel?.id !== tab ||
			panel.contentComponent !== 'ProjectReviewPanel' ||
			panel.params?.tab !== tab ||
			panel.params?.slot !== tab
		) {
			return false
		}
	}

	return true
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
			!isValidTabLayout(saved.tabs) ||
			(saved.tabLayouts !== undefined &&
				(typeof saved.tabLayouts !== 'object' ||
					saved.tabLayouts === null ||
					Object.entries(saved.tabLayouts).some(
						([key, layout]) =>
							!isValidTabLayout(layout) ||
							key !== workspaceTabLayoutKey(Object.keys(layout.panels) as ProjectReviewTab[]),
					)))
		) {
			return
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

/** Save detached panels in the grid so restoring a layout never opens browser windows. */
export function dockPopoutLayout(layout: SerializedDockview): SerializedDockview {
	const result: SerializedDockview = JSON.parse(JSON.stringify(layout))
	type GridNode = SerializedDockview['grid']['root']
	type Group = Exclude<GridNode['data'], GridNode[]>
	function findGroupPath(node: GridNode, id: string): GridNode[] | undefined {
		if (!Array.isArray(node.data)) return node.data.id === id ? [node] : undefined
		for (const child of node.data) {
			const path = findGroupPath(child, id)
			if (path) return [node, ...path]
		}
	}
	function restoreGroup(group: Group, reference?: string) {
		const path = reference ? (findGroupPath(result.grid.root, reference) ?? []) : []
		const existing = path.at(-1)
		if (existing && !Array.isArray(existing.data)) {
			existing.data.views.push(...group.views)
			existing.data.activeView ??= group.activeView
			for (const node of path) node.visible = true
			return
		}
		const leaf: GridNode = { type: 'leaf', data: group }
		if (Array.isArray(result.grid.root.data)) result.grid.root.data.push(leaf)
		else result.grid.root = { type: 'branch', data: [result.grid.root, leaf] }
	}
	function restoreGrid(node: GridNode, reference?: string) {
		if (Array.isArray(node.data)) {
			for (const child of node.data) restoreGrid(child, reference)
		} else restoreGroup(node.data, reference)
	}
	for (const popout of result.popoutGroups ?? []) {
		if (popout.data) restoreGroup(popout.data, popout.gridReferenceGroup)
		if (popout.grid) restoreGrid(popout.grid.root, popout.gridReferenceGroup)
	}
	delete result.popoutGroups
	if (result.activeGroup && !findGroupPath(result.grid.root, result.activeGroup)) {
		delete result.activeGroup
	}
	return result
}
