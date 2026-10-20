import type {
	DockviewApi,
	DockviewGroupPanel,
	DockviewIDisposable,
	DockviewReadyEvent,
	IDockviewPanel,
} from 'dockview-vue'
import type { ComputedRef, MaybeRefOrGetter, Ref } from 'vue'
import { computed, nextTick, ref, shallowReactive, toValue, watch } from 'vue'

import {
	infoFrom,
	isSameInfo,
	isWithinPath,
	parentInfoFrom,
	relocateInfo,
} from '#ui/layouts/shared/files-tab/utils.ts'

import type { FileEditorBridge } from '../providers/file-browser-ui'
import type { FileInfo, FileManagerContext } from '../providers/file-manager'

export const FILE_TAB_PANEL_COMPONENT = 'fileTabPanel'
export const FILE_TAB_COMPONENT = 'fileTab'

const MAX_HISTORY_LENGTH = 50
const PENDING_LOCATION_TIMEOUT = 1000
const WORKSPACE_STORAGE_PREFIX = 'files-workspace:'
const WORKSPACE_VERSION = 1
const MAX_RECENTLY_CLOSED = 10

/** Where the workspace points while every tab is closed. */
const ROOT_LOCATION = infoFrom('/')

/** A browser-like tab: its own history of visited directories and files. */
export interface FileTab {
	id: string
	history: FileInfo[]
	index: number
}

export interface FileTabPanelParams {
	tabId: string
}

/** The tabs of one workspace, as persisted to local storage. */
interface FileWorkspaceState {
	version: typeof WORKSPACE_VERSION
	tabs: FileTab[]
	activeTabId: string
}

export function currentLocation(tab: FileTab) {
	return tab.history[tab.index]
}

export interface FileTabsOptions {
	ctx: Pick<FileManagerContext, 'currentFile' | 'navigateTo'>
	/**
	 * Asks the user what to do with the given editors' unsaved changes.
	 * Resolves `true` when it is safe to discard the editors.
	 */
	confirmDiscard: (editors: FileEditorBridge[]) => Promise<boolean>
	/**
	 * Identifies the workspace the tabs are persisted under. Changing it swaps in that
	 * workspace's tabs; `null` keeps the tabs in memory only.
	 */
	workspaceId: MaybeRefOrGetter<string | null>
	/** Whether the tab strip is shown, further tabs can be opened and every tab can be closed. */
	enabled: MaybeRefOrGetter<boolean>
	/**
	 * `managed`: the tabs own a single dockview, kept to one strip of tabs and rebuilt from the
	 * tabs whenever the workspace changes. `shared`: the tabs live in dockviews laid out by a host
	 * (e.g. a workbench), which may split them or move them between its dockviews; the tabs only
	 * add the panels that are missing and drop ones for tabs that no longer exist.
	 */
	mode?: 'managed' | 'shared'
}

export interface OpenTabOptions {
	/** The group to open the tab in, rather than the focused one. */
	group?: DockviewGroupPanel
}

export interface FileTabs {
	tabs: Ref<FileTab[]>
	activeTabId: Ref<string>
	/** The active tab, or `null` while every tab is closed. */
	activeTab: ComputedRef<FileTab | null>
	activeLocation: ComputedRef<FileInfo>
	canGoBack: ComputedRef<boolean>
	canGoForward: ComputedRef<boolean>
	editors: Map<string, FileEditorBridge>
	activeEditor: ComputedRef<FileEditorBridge | null>
	hasUnsavedChanges: ComputedRef<boolean>
	getTab: (id: string) => FileTab | undefined
	navigate: (location: FileInfo) => Promise<void>
	back: () => Promise<void>
	forward: () => Promise<void>
	/** Navigates the given tab, focusing it first. */
	navigateIn: (tabId: string, location: FileInfo) => Promise<void>
	/** Moves the given tab through its history, focusing it first. */
	goIn: (tabId: string, delta: number) => Promise<void>
	canGoBackIn: (tabId: string) => boolean
	canGoForwardIn: (tabId: string) => boolean
	openTab: (location: FileInfo, options?: OpenTabOptions) => void
	activateTab: (id: string) => void
	closeTab: (id: string) => Promise<void>
	closeOtherTabs: (id: string) => Promise<void>
	closeAllTabs: () => Promise<void>
	/** Whether the tab can be closed; the last tab only can while tabs are enabled. */
	canClose: ComputedRef<boolean>
	/** Recently closed tabs, most recent first, with their history intact. */
	recentlyClosed: Ref<FileTab[]>
	reopenTab: (tab: FileTab) => void
	closeFile: (tabId: string) => void
	/** Prompts about unsaved changes in editors of files at or beneath any of `paths`. */
	confirmDiscardWithin: (paths: string[]) => Promise<boolean>
	/** Makes every tab follow `from` being moved or renamed to `to`. */
	relocate: (from: FileInfo, to: FileInfo) => void
	/** Sends every tab within the deleted `file` to its parent directory. */
	forget: (file: FileInfo) => void
	registerEditor: (tabId: string, bridge: FileEditorBridge) => void
	unregisterEditor: (tabId: string, bridge: FileEditorBridge) => void
	/** Whether a dockview panel is one of the tabs. */
	isTabPanel: (panel: Pick<IDockviewPanel, 'id'>) => boolean
	/**
	 * Asks about the tab's unsaved changes before it moves to another dockview, which remounts
	 * its editor. Resolves `true` once it is safe to move.
	 */
	confirmMove: (tabId: string) => Promise<boolean>
	/** Hosts the tabs in a dockview; the primary one is where new tabs go when none is focused. */
	attach: (dockview: DockviewApi, options?: { primary?: boolean }) => void
	detach: (dockview: DockviewApi) => void
	/** Adds the panels of tabs no dockview holds (e.g. after the host reset its layout). */
	syncPanels: () => void
	/** Attaches the managed dockview. */
	onReady: (event: DockviewReadyEvent) => void
	onDispose: () => void
}

function isFileInfo(value: unknown): value is FileInfo {
	if (typeof value !== 'object' || value === null) return false
	const info = value as Record<string, unknown>
	return (
		typeof info.path === 'string' &&
		typeof info.name === 'string' &&
		(info.type === 'file' || info.type === 'directory')
	)
}

function isFileTab(value: unknown): value is FileTab {
	if (typeof value !== 'object' || value === null) return false
	const tab = value as Record<string, unknown>
	return (
		typeof tab.id === 'string' &&
		Array.isArray(tab.history) &&
		tab.history.length > 0 &&
		tab.history.every(isFileInfo) &&
		typeof tab.index === 'number' &&
		tab.index >= 0 &&
		tab.index < tab.history.length
	)
}

function loadWorkspace(workspaceId: string | null): FileWorkspaceState | null {
	if (!workspaceId) return null
	try {
		const raw = localStorage.getItem(WORKSPACE_STORAGE_PREFIX + workspaceId)
		if (!raw) return null
		const state = JSON.parse(raw) as Partial<FileWorkspaceState>
		if (state.version !== WORKSPACE_VERSION || !Array.isArray(state.tabs)) return null
		const tabs = state.tabs.filter(isFileTab)
		const activeTabId = tabs.some((tab) => tab.id === state.activeTabId)
			? state.activeTabId!
			: (tabs[0]?.id ?? '')
		return { version: WORKSPACE_VERSION, tabs, activeTabId }
	} catch {
		return null
	}
}

function saveWorkspace(workspaceId: string | null, tabs: FileTab[], activeTabId: string) {
	if (!workspaceId) return
	const state: FileWorkspaceState = {
		version: WORKSPACE_VERSION,
		tabs: tabs.map((tab) => ({
			id: tab.id,
			index: tab.index,
			history: tab.history.map(({ name, type, path }) => ({ name, type, path })),
		})),
		activeTabId,
	}
	try {
		localStorage.setItem(WORKSPACE_STORAGE_PREFIX + workspaceId, JSON.stringify(state))
	} catch {
		// Storage may be full or unavailable; the workspace then only lives in memory.
	}
}

function isRootDirectory(location: FileInfo) {
	return location.type === 'directory' && location.path === '/'
}

/**
 * Browser-like file tabs. Each tab keeps its own history of directories and files, and the
 * active tab's current location is mirrored into the file manager context, so the listing,
 * editor and host URL keep working off the context. Any navigation that reaches the context
 * from elsewhere (e.g. the browser's back button) is recorded into the active tab's history.
 *
 * The tabs are rendered by dockview, one panel per tab, which also keeps each tab's open
 * editor alive while other tabs are browsed. Tabs are persisted per workspace, so each server
 * or instance reopens with the tabs it was left with.
 */
export function useFileTabs(options: FileTabsOptions): FileTabs {
	const { ctx } = options
	const managed = (options.mode ?? 'managed') === 'managed'

	const contextLocation = computed<FileInfo>(() => ctx.currentFile.value)

	function createTab(location: FileInfo): FileTab {
		return {
			id: `file-tab-${Math.random().toString(36).slice(2, 9)}`,
			history: [location],
			index: 0,
		}
	}

	const initialTab = createTab(contextLocation.value)
	const tabs = ref<FileTab[]>([initialTab])
	const activeTabId = ref(initialTab.id)

	const recentlyClosed = ref<FileTab[]>([])

	const activeTab = computed(
		() => tabs.value.find((tab) => tab.id === activeTabId.value) ?? tabs.value[0] ?? null,
	)
	const activeLocation = computed(() =>
		activeTab.value ? currentLocation(activeTab.value) : ROOT_LOCATION,
	)
	const canGoBack = computed(() => (activeTab.value?.index ?? 0) > 0)
	const canGoForward = computed(() => {
		const tab = activeTab.value
		return !!tab && tab.index < tab.history.length - 1
	})
	const canClose = computed(() => tabs.value.length > 1 || toValue(options.enabled))

	const editors = shallowReactive(new Map<string, FileEditorBridge>())
	const activeEditor = computed(() => editors.get(activeTabId.value) ?? null)
	const hasUnsavedChanges = computed(() =>
		[...editors.values()].some((editor) => editor.hasUnsavedChanges.value),
	)

	/** The dockviews hosting the tabs, with their event subscriptions. */
	const docks = new Map<DockviewApi, DockviewIDisposable[]>()
	let primaryDock: DockviewApi | null = null
	/** The dockview the active tab was last focused in, where new tabs open. */
	let focusedDock: DockviewApi | null = null
	/** Set while panels are being (re)built, so dockview's events don't feed back into the tabs. */
	let renderingPanels = false

	function getTab(id: string) {
		return tabs.value.find((tab) => tab.id === id)
	}

	function isTabPanel(panel: Pick<IDockviewPanel, 'id'>) {
		return getTab(panel.id) != null
	}

	/** The tab's panel and the dockview holding it. */
	function findPanel(id: string) {
		for (const dock of docks.keys()) {
			const panel = dock.getPanel(id)
			if (panel) return { dock, panel }
		}
		return null
	}

	function dockOfGroup(group: DockviewGroupPanel) {
		for (const dock of docks.keys()) if (dock.getGroup(group.id) === group) return dock
		return null
	}

	function targetDock() {
		if (focusedDock && docks.has(focusedDock)) return focusedDock
		if (primaryDock && docks.has(primaryDock)) return primaryDock
		return docks.keys().next().value ?? null
	}

	function pushLocation(tab: FileTab, location: FileInfo) {
		if (isSameInfo(currentLocation(tab), location)) return
		tab.history.splice(tab.index + 1)
		tab.history.push(location)
		if (tab.history.length > MAX_HISTORY_LENGTH) tab.history.shift()
		tab.index = tab.history.length - 1
	}

	/**
	 * The location we last pushed into the context. Hosts may apply it asynchronously (e.g. via
	 * the router), so context changes are only recorded once it has been reached.
	 */
	let pendingLocation: FileInfo | null = null
	let pendingTimeout: ReturnType<typeof setTimeout> | undefined

	function applyToContext(info: FileInfo) {
		if (isSameInfo(contextLocation.value, info)) return

		pendingLocation = info
		clearTimeout(pendingTimeout)
		pendingTimeout = setTimeout(() => (pendingLocation = null), PENDING_LOCATION_TIMEOUT)

		ctx.navigateTo(info)
	}

	watch(
		contextLocation,
		(location, previous) => {
			// Hosts may hand back an equal location as a new object (e.g. echoed from the router).
			if (previous && isSameInfo(location, previous)) return
			if (pendingLocation) {
				if (!isSameInfo(location, pendingLocation)) return
				pendingLocation = null
				clearTimeout(pendingTimeout)
				return
			}
			if (activeTab.value) {
				pushLocation(activeTab.value, location)
			} else {
				openTab(location)
			}
		},
		{ flush: 'post' },
	)

	/**
	 * Swaps in a workspace's persisted tabs, or a single tab at the context's location when there
	 * are none. A location the host was opened at (e.g. a deep link) wins over the restored one.
	 */
	function restoreWorkspace(state: FileWorkspaceState | null) {
		if (!state) {
			const tab = createTab(contextLocation.value)
			tabs.value = [tab]
			activeTabId.value = tab.id
			return
		}

		tabs.value = state.tabs
		activeTabId.value = state.activeTabId

		const location = contextLocation.value
		const tab = activeTab.value
		if (!isRootDirectory(location)) {
			if (tab) {
				pushLocation(tab, location)
			} else {
				const opened = createTab(location)
				tabs.value = [opened]
				activeTabId.value = opened.id
			}
		} else if (tab) {
			applyToContext(activeLocation.value)
		}
	}

	restoreWorkspace(loadWorkspace(toValue(options.workspaceId)))

	watch(
		() => toValue(options.workspaceId),
		(workspaceId) => {
			restoreWorkspace(loadWorkspace(workspaceId))
			renderPanels()
		},
	)

	watch(
		[tabs, activeTabId],
		() => saveWorkspace(toValue(options.workspaceId), tabs.value, activeTabId.value),
		{ deep: true },
	)

	async function confirmLeave(tab: FileTab) {
		const editor = editors.get(tab.id)
		if (currentLocation(tab).type !== 'file' || !editor) return true
		return options.confirmDiscard([editor])
	}

	async function navigate(location: FileInfo) {
		const tab = activeTab.value
		if (!tab) {
			openTab(location)
			return
		}
		if (!isSameInfo(currentLocation(tab), location)) {
			if (!(await confirmLeave(tab))) return
			pushLocation(tab, location)
		}
		applyToContext(location)
	}

	async function go(delta: number) {
		const tab = activeTab.value
		if (!tab) return
		const target = tab.index + delta
		if (target < 0 || target >= tab.history.length) return
		if (!(await confirmLeave(tab))) return
		tab.index = target
		applyToContext(currentLocation(tab))
	}

	async function navigateIn(tabId: string, location: FileInfo) {
		activateTab(tabId)
		if (activeTabId.value === tabId) await navigate(location)
	}

	async function goIn(tabId: string, delta: number) {
		activateTab(tabId)
		if (activeTabId.value === tabId) await go(delta)
	}

	function canGoBackIn(tabId: string) {
		return (getTab(tabId)?.index ?? 0) > 0
	}

	function canGoForwardIn(tabId: string) {
		const tab = getTab(tabId)
		return !!tab && tab.index < tab.history.length - 1
	}

	function applyHeaderVisibility() {
		if (!managed) return
		const hidden = !toValue(options.enabled)
		for (const group of primaryDock?.groups ?? []) group.header.hidden = hidden
	}

	watch(() => toValue(options.enabled), applyHeaderVisibility)

	/**
	 * Adds the tab's panel to `group`, else the focused dockview's active (or first) group.
	 * Without an explicit group, panels added while no group is active (e.g. `inactive` ones)
	 * each get a new group, splitting the view.
	 */
	function addPanel(tab: FileTab, inactive = false, group?: DockviewGroupPanel) {
		const dock = group ? dockOfGroup(group) : targetDock()
		if (!dock) return
		const reference = group ?? dock.activeGroup ?? dock.groups[0]
		dock.addPanel<FileTabPanelParams>({
			id: tab.id,
			component: FILE_TAB_PANEL_COMPONENT,
			tabComponent: FILE_TAB_COMPONENT,
			title: currentLocation(tab).path,
			params: { tabId: tab.id },
			position: reference ? { referenceGroup: reference, direction: 'within' } : undefined,
			inactive,
		})
		applyHeaderVisibility()
	}

	/**
	 * Managed: rebuilds the dockview from the tabs. Shared: drops panels of tabs that no longer
	 * exist (or that another dockview already holds) and adds the missing ones.
	 */
	function renderPanels() {
		renderingPanels = true
		try {
			if (managed) {
				if (!primaryDock) return
				primaryDock.clear()
				for (const tab of tabs.value) addPanel(tab, true)
				if (tabs.value.length === 0) primaryDock.addGroup()
				applyHeaderVisibility()
			} else {
				const seen = new Set<string>()
				for (const dock of docks.keys()) {
					for (const panel of [...dock.panels]) {
						if (panel.api.component !== FILE_TAB_PANEL_COMPONENT) continue
						if (!getTab(panel.id) || seen.has(panel.id)) dock.removePanel(panel)
						else seen.add(panel.id)
					}
				}
				for (const tab of tabs.value) if (!seen.has(tab.id)) addPanel(tab, true)
			}
			findPanel(activeTabId.value)?.panel.api.setActive()
		} finally {
			renderingPanels = false
		}
	}

	let renderScheduled = false

	/** Shared dockviews come up one by one, so the missing panels are added once all have. */
	function scheduleRender() {
		if (renderScheduled) return
		renderScheduled = true
		nextTick(() => {
			renderScheduled = false
			renderPanels()
		})
	}

	/** Keeps `tabs` in the order the user dragged them into, so it is restored that way. */
	function syncTabOrder() {
		const order = [...docks.keys()].flatMap((dock) =>
			dock.groups.flatMap((group) => group.panels.map((panel) => panel.id)),
		)
		const position = (tab: FileTab) => {
			const index = order.indexOf(tab.id)
			return index === -1 ? Number.MAX_SAFE_INTEGER : index
		}
		const sorted = [...tabs.value].sort((a, b) => position(a) - position(b))
		if (sorted.some((tab, index) => tab !== tabs.value[index])) tabs.value = sorted
	}

	function activateTab(id: string) {
		if (!getTab(id)) return
		const found = findPanel(id)
		if (found) focusedDock = found.dock
		if (found && !found.panel.api.isActive) found.panel.api.setActive()
		if (activeTabId.value === id) return
		activeTabId.value = id
		applyToContext(activeLocation.value)
	}

	function openTab(location: FileInfo, openOptions?: OpenTabOptions) {
		if (!toValue(options.enabled)) {
			navigate(location)
			return
		}
		const tab = createTab(location)
		tabs.value.push(tab)
		addPanel(tab, false, openOptions?.group)
		activateTab(tab.id)
	}

	/** Removes a tab without asking about unsaved changes, remembering it to reopen later. */
	function removeTab(tab: FileTab) {
		const index = tabs.value.indexOf(tab)
		if (index === -1) return

		tabs.value.splice(index, 1)
		editors.delete(tab.id)
		findPanel(tab.id)?.panel.api.close()
		recentlyClosed.value = [tab, ...recentlyClosed.value].slice(0, MAX_RECENTLY_CLOSED)

		if (activeTabId.value !== tab.id) return
		const next = tabs.value[index] ?? tabs.value[index - 1]
		if (next) {
			activateTab(next.id)
		} else {
			activeTabId.value = ''
			applyToContext(ROOT_LOCATION)
		}
	}

	/** Closes the given tabs, asking about all of their unsaved changes at once. */
	async function closeTabs(ids: string[]) {
		const closing = tabs.value.filter((tab) => ids.includes(tab.id))
		if (closing.length === 0) return
		if (closing.length === tabs.value.length && !toValue(options.enabled)) return

		const dirty = closing.flatMap((tab) => {
			const editor = editors.get(tab.id)
			return currentLocation(tab).type === 'file' && editor ? [editor] : []
		})
		if (!(await options.confirmDiscard(dirty))) return

		for (const tab of closing) removeTab(tab)
	}

	async function closeTab(id: string) {
		const tab = getTab(id)
		if (!tab || !canClose.value || !(await confirmLeave(tab))) return
		removeTab(tab)
	}

	async function closeOtherTabs(id: string) {
		await closeTabs(tabs.value.filter((tab) => tab.id !== id).map((tab) => tab.id))
		activateTab(id)
	}

	async function closeAllTabs() {
		await closeTabs(tabs.value.map((tab) => tab.id))
	}

	function reopenTab(tab: FileTab) {
		recentlyClosed.value = recentlyClosed.value.filter((closed) => closed !== tab)
		if (!toValue(options.enabled)) {
			navigate(currentLocation(tab))
			return
		}
		tabs.value.push(tab)
		addPanel(tab)
		activateTab(tab.id)
	}

	function closeFile(tabId: string) {
		const tab = getTab(tabId)
		if (!tab) return
		const location = currentLocation(tab)
		if (location.type !== 'file') return

		const parent: FileInfo = parentInfoFrom(location)
		pushLocation(tab, parent)
		if (tab.id === activeTabId.value) applyToContext(parent)
	}

	async function confirmDiscardWithin(paths: string[]) {
		const affected = tabs.value.flatMap((tab) => {
			const location = currentLocation(tab)
			const editor = editors.get(tab.id)
			return location.type === 'file' &&
				editor &&
				paths.some((path) => isWithinPath(location.path, path))
				? [editor]
				: []
		})
		return options.confirmDiscard(affected)
	}

	/** Rewrites every tab's history through `map`, collapsing neighbouring entries that end up the same. */
	function remapLocations(map: (location: FileInfo) => FileInfo) {
		for (const tab of [...tabs.value, ...recentlyClosed.value]) {
			const history: FileInfo[] = []
			let index = 0
			tab.history.forEach((entry, entryIndex) => {
				const mapped = map(entry)
				const previous = history[history.length - 1]
				if (!previous || !isSameInfo(previous, mapped)) history.push(mapped)
				if (entryIndex === tab.index) index = history.length - 1
			})
			tab.history = history
			tab.index = index
		}
		if (activeTab.value) applyToContext(activeLocation.value)
	}

	function relocate(from: FileInfo, to: FileInfo) {
		remapLocations((location) => relocateInfo(location, from.path, to.path))
	}

	function forget(file: FileInfo) {
		const parent = parentInfoFrom(file)
		remapLocations((location) => (isWithinPath(location.path, file.path) ? parent : location))
	}

	function registerEditor(tabId: string, bridge: FileEditorBridge) {
		editors.set(tabId, bridge)
	}

	function unregisterEditor(tabId: string, bridge: FileEditorBridge) {
		if (editors.get(tabId) === bridge) editors.delete(tabId)
	}

	async function confirmMove(tabId: string) {
		const editor = editors.get(tabId)
		return editor ? options.confirmDiscard([editor]) : true
	}

	function attach(dockview: DockviewApi, attachOptions?: { primary?: boolean }) {
		detach(dockview)
		if (attachOptions?.primary) primaryDock = dockview

		const disposables: DockviewIDisposable[] = [
			dockview.onDidActivePanelChange(({ panel }) => {
				if (!panel || renderingPanels || !getTab(panel.id)) return
				focusedDock = dockview
				activateTab(panel.id)
			}),
			dockview.onDidMovePanel(() => {
				if (!renderingPanels) syncTabOrder()
			}),
		]
		if (managed) {
			disposables.push(
				dockview.onWillShowOverlay((event) => {
					if (event.kind === 'content' || event.kind === 'edge') event.preventDefault()
				}),
			)
		}
		docks.set(dockview, disposables)

		if (managed) renderPanels()
		else scheduleRender()
	}

	/** Re-adds missing panels where new tabs go by default, rather than the last focused dockview. */
	function syncPanels() {
		focusedDock = null
		if (managed) renderPanels()
		else scheduleRender()
	}

	function detach(dockview: DockviewApi) {
		const disposables = docks.get(dockview)
		if (!disposables) return
		for (const disposable of disposables) disposable.dispose()
		docks.delete(dockview)
		if (primaryDock === dockview) primaryDock = null
		if (focusedDock === dockview) focusedDock = null
	}

	function onReady({ api: dockview }: DockviewReadyEvent) {
		attach(dockview, { primary: true })
	}

	function onDispose() {
		if (primaryDock) detach(primaryDock)
	}

	return {
		tabs,
		activeTabId,
		activeTab,
		activeLocation,
		canGoBack,
		canGoForward,
		editors,
		activeEditor,
		hasUnsavedChanges,
		getTab,
		navigate,
		back: () => go(-1),
		forward: () => go(1),
		navigateIn,
		goIn,
		canGoBackIn,
		canGoForwardIn,
		openTab,
		activateTab,
		closeTab,
		closeOtherTabs,
		closeAllTabs,
		canClose,
		recentlyClosed,
		reopenTab,
		closeFile,
		confirmDiscardWithin,
		relocate,
		forget,
		registerEditor,
		unregisterEditor,
		isTabPanel,
		confirmMove,
		attach,
		detach,
		syncPanels,
		onReady,
		onDispose,
	}
}
