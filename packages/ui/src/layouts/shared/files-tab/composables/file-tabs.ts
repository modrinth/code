import type { DockviewApi, DockviewReadyEvent, IDisposable } from 'dockview-vue'
import type { ComputedRef, MaybeRefOrGetter, Ref } from 'vue'
import { computed, ref, shallowReactive, shallowRef, toValue, watch } from 'vue'

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
	openTab: (location: FileInfo) => void
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

	const api = shallowRef<DockviewApi | null>(null)
	let disposables: IDisposable[] = []
	/** Set while panels are being (re)built, so dockview's events don't feed back into the tabs. */
	let renderingPanels = false

	function getTab(id: string) {
		return tabs.value.find((tab) => tab.id === id)
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
			if (api.value) renderPanels(api.value)
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

	function applyHeaderVisibility() {
		const hidden = !toValue(options.enabled)
		for (const group of api.value?.groups ?? []) group.header.hidden = hidden
	}

	watch(() => toValue(options.enabled), applyHeaderVisibility)

	/**
	 * Adds the tab's panel to the single tab group. Without an explicit group, panels added while
	 * no group is active (e.g. `inactive` ones) each get a new group, splitting the view.
	 */
	function addPanel(tab: FileTab, inactive = false) {
		const group = api.value?.groups[0]
		api.value?.addPanel<FileTabPanelParams>({
			id: tab.id,
			component: FILE_TAB_PANEL_COMPONENT,
			tabComponent: FILE_TAB_COMPONENT,
			title: currentLocation(tab).path,
			params: { tabId: tab.id },
			position: group ? { referenceGroup: group, direction: 'within' } : undefined,
			inactive,
		})
		applyHeaderVisibility()
	}

	function renderPanels(dockview: DockviewApi) {
		renderingPanels = true
		try {
			dockview.clear()
			for (const tab of tabs.value) addPanel(tab, true)
			if (tabs.value.length === 0) dockview.addGroup()
			applyHeaderVisibility()
			dockview.getPanel(activeTabId.value)?.api.setActive()
		} finally {
			renderingPanels = false
		}
	}

	/** Keeps `tabs` in the order the user dragged them into, so it is restored that way. */
	function syncTabOrder(dockview: DockviewApi) {
		const order = dockview.groups.flatMap((group) => group.panels.map((panel) => panel.id))
		const position = (tab: FileTab) => {
			const index = order.indexOf(tab.id)
			return index === -1 ? Number.MAX_SAFE_INTEGER : index
		}
		const sorted = [...tabs.value].sort((a, b) => position(a) - position(b))
		if (sorted.some((tab, index) => tab !== tabs.value[index])) tabs.value = sorted
	}

	function activateTab(id: string) {
		if (!getTab(id)) return
		const panel = api.value?.getPanel(id)
		if (panel && !panel.api.isActive) panel.api.setActive()
		if (activeTabId.value === id) return
		activeTabId.value = id
		applyToContext(activeLocation.value)
	}

	function openTab(location: FileInfo) {
		if (!toValue(options.enabled)) {
			navigate(location)
			return
		}
		const tab = createTab(location)
		tabs.value.push(tab)
		addPanel(tab)
		activateTab(tab.id)
	}

	/** Removes a tab without asking about unsaved changes, remembering it to reopen later. */
	function removeTab(tab: FileTab) {
		const index = tabs.value.indexOf(tab)
		if (index === -1) return

		tabs.value.splice(index, 1)
		editors.delete(tab.id)
		api.value?.getPanel(tab.id)?.api.close()
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

	function onReady({ api: dockview }: DockviewReadyEvent) {
		onDispose()
		api.value = dockview

		renderPanels(dockview)

		disposables = [
			dockview.onDidActivePanelChange(({ panel }) => {
				if (panel && !renderingPanels) activateTab(panel.id)
			}),
			dockview.onDidMovePanel(() => {
				if (!renderingPanels) syncTabOrder(dockview)
			}),
			dockview.onWillShowOverlay((event) => {
				if (event.kind === 'content' || event.kind === 'edge') event.preventDefault()
			}),
		]
	}

	function onDispose() {
		for (const disposable of disposables) disposable.dispose()
		disposables = []
		api.value = null
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
		onReady,
		onDispose,
	}
}
