import type { SerializedDockview } from 'dockview-vue'

import { type ProjectReviewTab, projectReviewTabs } from './types'

const STORAGE_KEY = 'moderation.project-review.layout.v1'
export const reviewPopoutUrl = '/project-review-popout.html'

export const workspacePanelSizes = {
	left: { default: 400, minimum: 200 },
	right: { default: 400, minimum: 200 },
	center: { minimum: 20 },
} as const

export interface SavedWorkspaceLayout {
	leftWidth: number
	rightWidth: number
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
		(layout.popoutGroups !== undefined && !Array.isArray(layout.popoutGroups)) ||
		layout.edgeGroups
	) {
		return false
	}

	const panelIds = new Set<string>()
	const groupIds = new Set<string>()
	type GridNode = SerializedDockview['grid']['root']
	function validateGroup(group: Exclude<GridNode['data'], GridNode[]>) {
		if (
			!group ||
			typeof group.id !== 'string' ||
			groupIds.has(group.id) ||
			!Array.isArray(group.views) ||
			(group.activeView !== undefined && !group.views.includes(group.activeView))
		)
			return false
		groupIds.add(group.id)
		for (const id of group.views) {
			if (typeof id !== 'string' || !Object.hasOwn(layout.panels, id) || panelIds.has(id))
				return false
			panelIds.add(id)
		}
		return true
	}
	function validateGrid(node: GridNode): boolean {
		if (!node) return false
		if (node.type === 'branch') return Array.isArray(node.data) && node.data.every(validateGrid)
		return node.type === 'leaf' && !Array.isArray(node.data) && validateGroup(node.data)
	}
	if (!validateGrid(layout.grid.root)) return false
	for (const popout of layout.popoutGroups ?? []) {
		if (
			!popout ||
			(popout.url !== undefined && popout.url !== reviewPopoutUrl) ||
			!!popout.data === !!popout.grid ||
			(popout.position !== null &&
				(!popout.position ||
					![popout.position.left, popout.position.top].every(Number.isFinite) ||
					![popout.position.width, popout.position.height].every(
						(size) => Number.isFinite(size) && size > 0,
					))) ||
			!(popout.grid ? validateGrid(popout.grid.root) : popout.data && validateGroup(popout.data))
		)
			return false
	}
	if (panelIds.size !== Object.keys(layout.panels).length) return false

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
			![saved.leftWidth, saved.rightWidth].every(
				(size) => typeof size === 'number' && Number.isFinite(size) && size > 0,
			) ||
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

		for (const layout of [saved.tabs, ...Object.values(saved.tabLayouts ?? {})]) {
			for (const popout of layout.popoutGroups ?? []) popout.url = reviewPopoutUrl
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

/** Keep saved popout panels accessible in the workspace until the user restores their windows. */
export function dockPopoutLayout(layout: SerializedDockview): SerializedDockview {
	const result: SerializedDockview = structuredClone(layout)
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
		const path = reference ? findGroupPath(result.grid.root, reference) : undefined
		const existing = path?.at(-1)
		if (existing && !Array.isArray(existing.data)) {
			existing.data.views.push(...group.views)
			existing.data.activeView ??= group.activeView
			for (const node of path ?? []) node.visible = true
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
	if (result.activeGroup && !findGroupPath(result.grid.root, result.activeGroup))
		delete result.activeGroup
	return result
}
