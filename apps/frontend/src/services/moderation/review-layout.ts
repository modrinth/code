import { moderationSettings } from '@modrinth/moderation'
import { useLocalStorage } from '@vueuse/core'
import { computed, type ComputedRef, type Ref } from 'vue'

import { useModerationSettings } from '~/composables/moderation.ts'

/**
 * Shared state for the moderation "review view" — the VS Code / Slicer style shell that
 * replaces the project page chrome while a moderator is actively reviewing a project.
 *
 * The state is intentionally **not keyed by project id** so that the set of open tabs, the
 * PiP dock contents and the sidebar collapse state all persist as a moderator moves between
 * projects (e.g. stepping through the moderation queue). It is backed by localStorage so it
 * also survives the `[type]/[project].vue` remount that happens on navigation.
 */

export type ReviewTabId =
	| 'description'
	| 'gallery'
	| 'changelog'
	| 'versions'
	| 'tags'
	| 'license'
	| 'links'
	| 'disclosures'
	| 'permissions'
	| 'thread'
	| 'settings'
	| 'moderation_settings'
	| 'tech_review'

export type ReviewDockId = 'main' | 'pip'
export type SplitDirection = 'row' | 'column'

/** Canonical tab order, used when rendering tab strips. */
export const REVIEW_TAB_ORDER: readonly ReviewTabId[] = [
	'description',
	'gallery',
	'changelog',
	'versions',
	'tags',
	'license',
	'links',
	'disclosures',
	'permissions',
	'thread',
	'settings',
	'moderation_settings',
	'tech_review',
]

/**
 * A saved, named split — "which tabs, laid out which way" — that can be restored in one click
 * instead of re-adding each tab to the split every time. Restoring a group doesn't stop you from
 * switching away to look at one other tab; the group's own membership is untouched until you
 * explicitly save over it or delete it, so it's always there to jump back to.
 */
export interface TabGroup {
	id: string
	label: string
	tabs: ReviewTabId[]
	direction: SplitDirection
}

interface DockState {
	/** Open tabs in this dock, in the order they were opened. */
	tabs: ReviewTabId[]
	/** Subset of `tabs` currently rendered (multi-select split view; usually one). */
	active: ReviewTabId[]
	/** Per-tab flex weight for the split view; missing entries default to 1. */
	weights: Partial<Record<ReviewTabId, number>>
	/** Split-view orientation: side-by-side ('row') or stacked ('column'). */
	splitDirection: SplitDirection
}

interface ReviewLayoutState {
	sidebarCollapsed: boolean
	/** When false, the checklist stops driving which tab is focused ("disconnect mode"). */
	checklistConnected: boolean
	pipOpen: boolean
	docks: Record<ReviewDockId, DockState>
	/** Which sub-section the Settings tab shows. */
	settingsSection: string
	/** The version the "version" tab shows (opened by clicking a version in the Versions tab). */
	selectedVersionId: string | null
	/** Left project-info sidebar width in px (when not collapsed). */
	sidebarWidth: number
	/** Right checklist module width in px. */
	checklistPanelWidth: number
	/** Whether the bottom checklist walkthrough widget is collapsed to its slim bar. */
	walkthroughCollapsed: boolean
	/** Saved splits, restorable in either dock regardless of which dock they were saved from. */
	tabGroups: TabGroup[]
}

export const SIDEBAR_WIDTH_MIN = 220
export const SIDEBAR_WIDTH_MAX = 560
export const CHECKLIST_PANEL_WIDTH_MIN = 280
export const CHECKLIST_PANEL_WIDTH_MAX = 760

function clamp(value: number, min: number, max: number): number {
	return Math.min(Math.max(value, min), max)
}

const STORAGE_KEY = 'moderation-review-layout-v1'

const settings = useModerationSettings()

function defaultState(): ReviewLayoutState {
	return {
		sidebarCollapsed: false,
		checklistConnected: !settings.value.get(
			moderationSettings.Experimental.UnlinkChecklistInReview,
		),
		pipOpen: false,
		docks: {
			main: {
				tabs: ['description'],
				active: ['description'],
				weights: {},
				splitDirection: 'row',
			},
			pip: { tabs: [], active: [], weights: {}, splitDirection: 'row' },
		},
		settingsSection: 'general',
		selectedVersionId: null,
		sidebarWidth: 304,
		checklistPanelWidth: 400,
		walkthroughCollapsed: false,
		tabGroups: [],
	}
}

export interface OpenTabOptions {
	dock?: ReviewDockId
	/** Focus (solo-activate) the tab after opening it. Defaults to true. */
	focus?: boolean
	/** Set when the call originates from the checklist; ignored while disconnected. */
	fromChecklist?: boolean
	/** Should be opened if not present  */
	important?: boolean
}

export interface ModerationReviewLayout {
	state: Ref<ReviewLayoutState>
	sidebarCollapsed: ComputedRef<boolean>
	checklistConnected: ComputedRef<boolean>
	pipOpen: ComputedRef<boolean>
	mainDock: ComputedRef<DockState>
	pipDock: ComputedRef<DockState>
	dockOf: (tab: ReviewTabId) => ReviewDockId | null
	openTab: (tab: ReviewTabId, options?: OpenTabOptions) => void
	closeTab: (tab: ReviewTabId) => void
	focusTab: (tab: ReviewTabId, dock?: ReviewDockId) => void
	toggleActive: (tab: ReviewTabId, dock: ReviewDockId) => void
	reorderTabs: (dock: ReviewDockId, orderedIds: ReviewTabId[]) => void
	moveTab: (tab: ReviewTabId, toDock: ReviewDockId) => void
	panelWeight: (dock: ReviewDockId, tab: ReviewTabId) => number
	setPanelWeights: (dock: ReviewDockId, weights: Partial<Record<ReviewTabId, number>>) => void
	toggleSplitDirection: (dock: ReviewDockId) => void
	setSidebarCollapsed: (value: boolean) => void
	toggleSidebar: () => void
	sidebarWidth: ComputedRef<number>
	setSidebarWidth: (px: number) => void
	checklistPanelWidth: ComputedRef<number>
	setChecklistPanelWidth: (px: number) => void
	walkthroughCollapsed: ComputedRef<boolean>
	toggleWalkthroughCollapsed: () => void
	settingsSection: ComputedRef<string>
	setSettingsSection: (section: string) => void
	selectedVersionId: ComputedRef<string | null>
	setSelectedVersion: (id: string | null) => void
	setChecklistConnected: (value: boolean) => void
	openPip: () => void
	closePip: () => void
	reclaimPipTabs: () => void
	tabGroups: ComputedRef<TabGroup[]>
	/** Save the dock's current split (2+ active tabs) as a group. Returns its id, or null if the
	 *  dock isn't currently split. */
	saveTabGroup: (dock: ReviewDockId, label: string) => string | null
	/** Open every tab in the group (in `dock`, pulling them out of the other dock if needed) as
	 *  its saved split — the group itself is left untouched, so it stays restorable. */
	activateTabGroup: (dock: ReviewDockId, groupId: string) => void
	renameTabGroup: (groupId: string, label: string) => void
	deleteTabGroup: (groupId: string) => void
	reset: () => void
}

let singleton: ModerationReviewLayout | null = null

function create(): ModerationReviewLayout {
	const KNOWN_TABS = new Set<string>(REVIEW_TAB_ORDER)

	function sanitizeDock(dock: DockState, fallback: ReviewTabId[]): DockState {
		let tabs = dock.tabs.filter((t) => KNOWN_TABS.has(t))
		if (tabs.length === 0) tabs = [...fallback]
		let active = dock.active.filter((t) => tabs.includes(t))
		if (active.length === 0 && tabs.length > 0) active = [tabs[tabs.length - 1]]
		return { ...dock, tabs, active }
	}

	/** Drop stale tab ids and any group left with fewer than 2 valid tabs — a "group" of one
	 *  wouldn't be a split. */
	function sanitizeTabGroups(groups: TabGroup[] | undefined): TabGroup[] {
		if (!groups) return []
		return groups
			.map((g) => ({ ...g, tabs: g.tabs.filter((t) => KNOWN_TABS.has(t)) }))
			.filter((g) => g.tabs.length >= 2)
	}

	const state = useLocalStorage<ReviewLayoutState>(STORAGE_KEY, defaultState(), {
		mergeDefaults: (storageValue, defaults) => {
			const stored = storageValue as Partial<ReviewLayoutState> | null
			return {
				...defaults,
				...stored,
				docks: {
					main: sanitizeDock({ ...defaults.docks.main, ...stored?.docks?.main }, ['description']),
					pip: sanitizeDock({ ...defaults.docks.pip, ...stored?.docks?.pip }, []),
				},
				tabGroups: sanitizeTabGroups(stored?.tabGroups),
			}
		},
	})

	function dockContaining(tab: ReviewTabId): ReviewDockId | null {
		if (state.value.docks.main.tabs.includes(tab)) return 'main'
		if (state.value.docks.pip.tabs.includes(tab)) return 'pip'
		return null
	}

	function panelWeight(dock: ReviewDockId, tab: ReviewTabId): number {
		const w = state.value.docks[dock].weights?.[tab]
		return typeof w === 'number' && w > 0 ? w : 1
	}

	function setPanelWeights(dock: ReviewDockId, weights: Partial<Record<ReviewTabId, number>>) {
		state.value.docks[dock].weights = { ...state.value.docks[dock].weights, ...weights }
	}

	function toggleSplitDirection(dock: ReviewDockId) {
		const d = state.value.docks[dock]
		d.splitDirection = d.splitDirection === 'row' ? 'column' : 'row'
	}

	/** Guarantee a dock always renders at least one of its open tabs. */
	function normalizeActive(id: ReviewDockId) {
		const d = state.value.docks[id]
		d.active = d.active.filter((t) => d.tabs.includes(t))
		if (d.active.length === 0 && d.tabs.length > 0) {
			d.active = [d.tabs[d.tabs.length - 1]]
		}
		// A solo panel's stored split ratio is meaningless once nothing shares the row/column
		// with it. Drop it here so the next time this dock splits, it starts from an even
		// distribution instead of resuming whatever ratio was left over from an earlier split.
		if (d.active.length <= 1) d.weights = {}
	}

	function removeFromDock(tab: ReviewTabId, id: ReviewDockId) {
		const d = state.value.docks[id]
		d.tabs = d.tabs.filter((t) => t !== tab)
		d.active = d.active.filter((t) => t !== tab)
		normalizeActive(id)
	}

	function openTab(tab: ReviewTabId, options: OpenTabOptions = {}) {
		const { dock = 'main', focus = true, fromChecklist = false, important = false } = options

		// A tab lives in exactly one dock at a time.
		const existing = dockContaining(tab)

		if ((fromChecklist && !state.value.checklistConnected) || (important && existing)) return

		if (existing && existing !== dock) {
			removeFromDock(tab, existing)
		}

		const target = state.value.docks[dock]
		if (!target.tabs.includes(tab)) target.tabs.push(tab)

		if (focus) {
			target.active = [tab]
		} else if (!target.active.includes(tab) && target.active.length === 0) {
			target.active = [tab]
		}
		normalizeActive(dock)
	}

	function closeTab(tab: ReviewTabId) {
		const id = dockContaining(tab)
		if (id) removeFromDock(tab, id)
	}

	function focusTab(tab: ReviewTabId, dock?: ReviewDockId) {
		const id = dock ?? dockContaining(tab)
		if (!id) return
		if (!state.value.docks[id].tabs.includes(tab)) return
		state.value.docks[id].active = [tab]
		normalizeActive(id)
	}

	function toggleActive(tab: ReviewTabId, dock: ReviewDockId) {
		const d = state.value.docks[dock]
		if (!d.tabs.includes(tab)) return
		if (d.active.includes(tab)) {
			if (d.active.length > 1) d.active = d.active.filter((t) => t !== tab)
		} else {
			// Display order is derived from `tabs`, so membership is all that matters here.
			d.active = [...d.active, tab]
		}
		normalizeActive(dock)
	}

	/** Apply a new tab order for a dock (from drag-and-drop). Missing tabs are appended. */
	function reorderTabs(dock: ReviewDockId, orderedIds: ReviewTabId[]) {
		const d = state.value.docks[dock]
		const present = new Set(d.tabs)
		const next = orderedIds.filter((id) => present.has(id))
		for (const id of d.tabs) if (!next.includes(id)) next.push(id)
		d.tabs = next
	}

	function moveTab(tab: ReviewTabId, toDock: ReviewDockId) {
		const from = dockContaining(tab)
		if (from === toDock) return
		if (from) removeFromDock(tab, from)
		openTab(tab, { dock: toDock, focus: true })
		if (toDock === 'pip') state.value.pipOpen = true
	}

	function setSidebarCollapsed(value: boolean) {
		state.value.sidebarCollapsed = value
	}

	function toggleSidebar() {
		state.value.sidebarCollapsed = !state.value.sidebarCollapsed
	}

	function setChecklistConnected(value: boolean) {
		state.value.checklistConnected = value
		settings.value.set(moderationSettings.Experimental.UnlinkChecklistInReview, !value)
	}

	function setSidebarWidth(px: number) {
		state.value.sidebarWidth = clamp(Math.round(px), SIDEBAR_WIDTH_MIN, SIDEBAR_WIDTH_MAX)
	}

	function setChecklistPanelWidth(px: number) {
		state.value.checklistPanelWidth = clamp(
			Math.round(px),
			CHECKLIST_PANEL_WIDTH_MIN,
			CHECKLIST_PANEL_WIDTH_MAX,
		)
	}

	function toggleWalkthroughCollapsed() {
		state.value.walkthroughCollapsed = !state.value.walkthroughCollapsed
	}

	function setSettingsSection(section: string) {
		state.value.settingsSection = section
	}

	function setSelectedVersion(id: string | null) {
		state.value.selectedVersionId = id
	}

	function openPip() {
		state.value.pipOpen = true
	}

	function closePip() {
		state.value.pipOpen = false
	}

	/** Pull every PiP tab back into the main dock without disturbing the main dock's focus. */
	function reclaimPipTabs() {
		const pip = state.value.docks.pip
		const main = state.value.docks.main
		for (const tab of pip.tabs) {
			if (!main.tabs.includes(tab)) main.tabs.push(tab)
		}
		pip.tabs = []
		pip.active = []
		normalizeActive('main')
		state.value.pipOpen = false
	}

	function saveTabGroup(dock: ReviewDockId, label: string): string | null {
		const d = state.value.docks[dock]
		if (d.active.length < 2) return null
		const id = `group-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 7)}`
		state.value.tabGroups = [
			...state.value.tabGroups,
			{
				id,
				label: label.trim() || 'Untitled group',
				tabs: [...d.active],
				direction: d.splitDirection,
			},
		]
		return id
	}

	function activateTabGroup(dock: ReviewDockId, groupId: string) {
		const group = state.value.tabGroups.find((g) => g.id === groupId)
		if (!group || group.tabs.length === 0) return
		for (const tab of group.tabs) {
			const existing = dockContaining(tab)
			if (existing && existing !== dock) removeFromDock(tab, existing)
			const target = state.value.docks[dock]
			if (!target.tabs.includes(tab)) target.tabs.push(tab)
		}
		const d = state.value.docks[dock]
		d.active = group.tabs.filter((t) => d.tabs.includes(t))
		d.splitDirection = group.direction
		normalizeActive(dock)
	}

	function renameTabGroup(groupId: string, label: string) {
		const trimmed = label.trim()
		if (!trimmed) return
		state.value.tabGroups = state.value.tabGroups.map((g) =>
			g.id === groupId ? { ...g, label: trimmed } : g,
		)
	}

	function deleteTabGroup(groupId: string) {
		state.value.tabGroups = state.value.tabGroups.filter((g) => g.id !== groupId)
	}

	function reset() {
		state.value = defaultState()
	}

	return {
		state,
		sidebarCollapsed: computed(() => state.value.sidebarCollapsed),
		checklistConnected: computed(() => state.value.checklistConnected),
		pipOpen: computed(() => state.value.pipOpen),
		mainDock: computed(() => state.value.docks.main),
		pipDock: computed(() => state.value.docks.pip),
		dockOf: dockContaining,
		openTab,
		closeTab,
		focusTab,
		toggleActive,
		reorderTabs,
		moveTab,
		panelWeight,
		setPanelWeights,
		toggleSplitDirection,
		setSidebarCollapsed,
		toggleSidebar,
		sidebarWidth: computed(() => state.value.sidebarWidth),
		setSidebarWidth,
		checklistPanelWidth: computed(() => state.value.checklistPanelWidth),
		setChecklistPanelWidth,
		walkthroughCollapsed: computed(() => state.value.walkthroughCollapsed),
		toggleWalkthroughCollapsed,
		settingsSection: computed(() => state.value.settingsSection),
		setSettingsSection,
		selectedVersionId: computed(() => state.value.selectedVersionId),
		setSelectedVersion,
		setChecklistConnected,
		openPip,
		closePip,
		reclaimPipTabs,
		tabGroups: computed(() => state.value.tabGroups),
		saveTabGroup,
		activateTabGroup,
		renameTabGroup,
		deleteTabGroup,
		reset,
	}
}

export function useModerationReviewLayout(): ModerationReviewLayout {
	if (!singleton) singleton = create()
	return singleton
}
