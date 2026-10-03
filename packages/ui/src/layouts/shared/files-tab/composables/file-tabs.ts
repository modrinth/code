import type { DockviewApi, DockviewReadyEvent, IDisposable } from 'dockview-vue'
import type { ComputedRef, Ref } from 'vue'
import { computed, ref, shallowReactive, shallowRef, watch } from 'vue'

import {isSameInfo, parentInfoFrom} from "#ui/layouts/shared/files-tab/utils.ts";

import type { FileEditorBridge } from '../providers/file-browser-ui'
import type { FileInfo, FileManagerContext } from '../providers/file-manager'

export const FILE_TAB_PANEL_COMPONENT = 'fileTabPanel'
export const FILE_TAB_COMPONENT = 'fileTab'

const MAX_HISTORY_LENGTH = 50
const PENDING_LOCATION_TIMEOUT = 1000

/** A browser-like tab: its own history of visited directories and files. */
export interface FileTab {
	id: string
	history: FileInfo[]
	index: number
}

export interface FileTabPanelParams {
	tabId: string
}

export function currentLocation(tab: FileTab) {
	return tab.history[tab.index]
}

export interface FileTabsOptions {
	ctx: Pick<FileManagerContext, 'currentDirectory' | 'currentFile' | 'navigateTo'>
	/**
	 * Asks the user what to do with the given editors' unsaved changes.
	 * Resolves `true` when it is safe to discard the editors.
	 */
	confirmDiscard: (editors: FileEditorBridge[]) => Promise<boolean>
}

export interface FileTabs {
	tabs: Ref<FileTab[]>
	activeTabId: Ref<string>
	activeTab: ComputedRef<FileTab>
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
	closeFile: (tabId: string) => void
	registerEditor: (tabId: string, bridge: FileEditorBridge) => void
	unregisterEditor: (tabId: string, bridge: FileEditorBridge) => void
	onReady: (event: DockviewReadyEvent) => void
	onDispose: () => void
}

/**
 * Browser-like file tabs. Each tab keeps its own history of directories and files, and the
 * active tab's current location is mirrored into the file manager context (`currentPath` /
 * `editingFile`), so the listing, editor and host URL keep working off the context. Any
 * navigation that reaches the context from elsewhere (e.g. the browser's back button) is
 * recorded into the active tab's history.
 *
 * The tabs are rendered by dockview, one panel per tab, which also keeps each tab's open
 * editor alive while other tabs are browsed.
 */
export function useFileTabs(options: FileTabsOptions): FileTabs {
	const { ctx } = options

	const contextLocation = computed<FileInfo>(() => ctx.currentFile.value);

	let nextTabId = 0
	function createTab(location: FileInfo): FileTab {
		return { id: `file-tab-${nextTabId++}`, history: [location], index: 0 }
	}

	const initialTab = createTab(contextLocation.value)
	const tabs = ref<FileTab[]>([initialTab])
	const activeTabId = ref(initialTab.id)

	const activeTab = computed(
		() => tabs.value.find((tab) => tab.id === activeTabId.value) ?? tabs.value[0],
	)
	const activeLocation = computed(() => currentLocation(activeTab.value))
	const canGoBack = computed(() => activeTab.value.index > 0)
	const canGoForward = computed(
		() => activeTab.value.index < activeTab.value.history.length - 1,
	)

	const editors = shallowReactive(new Map<string, FileEditorBridge>())
	const activeEditor = computed(() => editors.get(activeTabId.value) ?? null)
	const hasUnsavedChanges = computed(() =>
		[...editors.values()].some((editor) => editor.hasUnsavedChanges.value),
	)

	const api = shallowRef<DockviewApi | null>(null)
	let disposables: IDisposable[] = []

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

		ctx.navigateTo(info);
	}

	watch(
		contextLocation,
		(location) => {
			if (pendingLocation) {
				if (!isSameInfo(location, pendingLocation)) return
				pendingLocation = null
				clearTimeout(pendingTimeout)
			}
			pushLocation(activeTab.value, location)
		},
		{ flush: 'post' },
	)

	async function confirmLeave(tab: FileTab) {
		const editor = editors.get(tab.id)
		if (currentLocation(tab).type !== 'file' || !editor) return true
		return options.confirmDiscard([editor])
	}

	async function navigate(location: FileInfo) {
		const tab = activeTab.value
		if (!isSameInfo(currentLocation(tab), location)) {
			if (!(await confirmLeave(tab))) return
			pushLocation(tab, location)
		}
		applyToContext(location)
	}

	async function go(delta: number) {
		const tab = activeTab.value
		const target = tab.index + delta
		if (target < 0 || target >= tab.history.length) return
		if (!(await confirmLeave(tab))) return
		tab.index = target
		applyToContext(currentLocation(tab))
	}

	function addPanel(tab: FileTab, inactive = false) {
		api.value?.addPanel<FileTabPanelParams>({
			id: tab.id,
			component: FILE_TAB_PANEL_COMPONENT,
			tabComponent: FILE_TAB_COMPONENT,
			title: currentLocation(tab).path,
			params: { tabId: tab.id },
			inactive,
		})
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
		const tab = createTab(location)
		tabs.value.push(tab)
		addPanel(tab)
		activateTab(tab.id)
	}

	async function closeTab(id: string) {
		if (tabs.value.length <= 1) return
		const index = tabs.value.findIndex((tab) => tab.id === id)
		const tab = tabs.value[index]
		if (!tab || !(await confirmLeave(tab))) return

		tabs.value.splice(index, 1)
		editors.delete(id)
		api.value?.getPanel(id)?.api.close()

		if (activeTabId.value === id) {
			activateTab((tabs.value[index] ?? tabs.value[index - 1]).id)
		}
	}

	function closeFile(tabId: string) {
		const tab = getTab(tabId)
		if (!tab) return
		const location = currentLocation(tab)
		if (location.type !== 'file') return

		const parent: FileInfo = parentInfoFrom(location);
		pushLocation(tab, parent)
		if (tab.id === activeTabId.value) applyToContext(parent)
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

		for (const tab of tabs.value) addPanel(tab, true)
		dockview.getPanel(activeTabId.value)?.api.setActive()

		disposables = [
			dockview.onDidActivePanelChange(({ panel }) => {
				if (panel) activateTab(panel.id)
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
		closeFile,
		registerEditor,
		unregisterEditor,
		onReady,
		onDispose,
	}
}
