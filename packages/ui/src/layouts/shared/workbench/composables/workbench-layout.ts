import type {
	AddPanelPositionOptions,
	DockviewApi,
	DockviewDidDropEvent,
	DockviewDndOverlayEvent,
	DockviewGroupPanel,
	DockviewReadyEvent,
	GridviewApi,
	GridviewReadyEvent,
	GroupviewPanelState,
	IDisposable,
	IDockviewPanel,
	Position,
	SerializedDockview,
} from 'dockview-vue'
import { LayoutPriority } from 'dockview-vue'
import {
	computed,
	type MaybeRefOrGetter,
	onBeforeUnmount,
	ref,
	type Ref,
	type Slots,
	toValue,
	watch,
} from 'vue'

import type { WorkbenchLayoutState } from '../providers/workbench'
import {
	DEFAULT_WORKBENCH_REGION_SIZES,
	WORKBENCH_CENTER_MINIMUM,
	WORKBENCH_REGIONS,
	type WorkbenchDocuments,
	type WorkbenchHost,
	type WorkbenchPanel,
	type WorkbenchRegion,
	type WorkbenchRegionSizes,
} from '../types'

export const WORKBENCH_DOCK_COMPONENT = 'workbenchDock'
export const WORKBENCH_PANEL_COMPONENT = 'workbenchPanel'
export const WORKBENCH_TAB_COMPONENT = 'workbenchTab'

const STORAGE_PREFIX = 'workbench-layout:'
const LAYOUT_VERSION = 3
const SAVE_DELAY = 200

export interface WorkbenchDockParams {
	host: WorkbenchHost
}

export interface WorkbenchPanelParams {
	id: string
}

/** A workbench's layout, as persisted to local storage. */
interface SavedWorkbenchLayout {
	version: typeof LAYOUT_VERSION
	sizes: Partial<Record<WorkbenchRegion, number>>
	visible: Partial<Record<WorkbenchRegion, boolean>>
	docks: Partial<Record<WorkbenchHost, SerializedDockview>>
	/** Panels moved away from the dock they're declared in. */
	placements: Record<string, WorkbenchHost>
	/** Panels the user closed. */
	closed: string[]
}

export interface WorkbenchLayoutOptions {
	slots: Slots
	panels: MaybeRefOrGetter<WorkbenchPanel[]>
	/**
	 * Identifies the layout in local storage. `null` keeps it in memory only, e.g. for a one-off
	 * view that shouldn't touch the user's own arrangement.
	 */
	storageKey: string | null
	sizes?: Partial<WorkbenchRegionSizes>
	/** Regions hidden when opened, regardless of how they were left. */
	hiddenRegions?: WorkbenchRegion[]
	documents?: WorkbenchDocuments
}

function loadLayout(storageKey: string | null): SavedWorkbenchLayout | null {
	if (!storageKey) return null
	try {
		const saved = JSON.parse(localStorage.getItem(STORAGE_PREFIX + storageKey) ?? 'null')
		return saved?.version === LAYOUT_VERSION ? (saved as SavedWorkbenchLayout) : null
	} catch {
		return null
	}
}

function saveLayout(storageKey: string | null, layout: SavedWorkbenchLayout) {
	if (!storageKey) return
	try {
		localStorage.setItem(STORAGE_PREFIX + storageKey, JSON.stringify(layout))
	} catch {
		// Storage may be full or unavailable; the layout then only lives in memory.
	}
}

const DROP_DIRECTIONS = {
	top: 'above',
	bottom: 'below',
	left: 'left',
	right: 'right',
	center: 'within',
} as const satisfies Record<Position, string>

/** Where a panel dropped at `position` (of `group`, or of the whole dock without one) goes. */
function dropPosition(
	position: Position,
	group?: DockviewGroupPanel,
): AddPanelPositionOptions | undefined {
	if (group) return { referenceGroup: group, direction: DROP_DIRECTIONS[position] }
	if (position === 'center') return undefined
	return { direction: DROP_DIRECTIONS[position] }
}

/** Alongside the panels of the dock's active (or first) group. */
function defaultPosition(api: DockviewApi): AddPanelPositionOptions | undefined {
	const group = api.activeGroup ?? api.groups[0]
	return group ? { referenceGroup: group, direction: 'within' } : undefined
}

function isRegion(host: WorkbenchHost): host is WorkbenchRegion {
	return host !== 'center'
}

/**
 * Lays the workbench out as a grid: its center, with resizable regions to the left, right and
 * below it. Each is its own dockview (a "dock"), so its panels can be tabbed and split within it,
 * and panels can be dragged (or moved from their menu) from one dock to another. Besides its own
 * panels, the workbench hosts `documents` (e.g. file tabs) whose owner creates and closes them.
 * Region sizes and visibility, every dock's arrangement, and where panels were moved or closed
 * are persisted per `storageKey`.
 */
export function useWorkbenchLayout(options: WorkbenchLayoutOptions): WorkbenchLayoutState {
	const sizes: WorkbenchRegionSizes = { ...DEFAULT_WORKBENCH_REGION_SIZES, ...options.sizes }
	const saved = loadLayout(options.storageKey)
	const documents = options.documents

	const panels = computed(() => toValue(options.panels))
	const placements = ref<Record<string, WorkbenchHost>>({ ...saved?.placements })
	const closed = ref<string[]>([...(saved?.closed ?? [])])

	function regionOf(panel: WorkbenchPanel): WorkbenchHost {
		return placements.value[panel.id] ?? panel.region
	}

	const openPanels = computed(() =>
		panels.value.filter((panel) => !closed.value.includes(panel.id)),
	)
	const closedPanels = computed(() =>
		panels.value.filter((panel) => closed.value.includes(panel.id)),
	)

	function panelsIn(host: WorkbenchHost) {
		return openPanels.value.filter((panel) => regionOf(panel) === host)
	}

	function isDocumentComponent(component: string | undefined) {
		return !!documents && !!component && component in documents.components
	}

	function isDocumentPanel(panel: IDockviewPanel) {
		return isDocumentComponent(panel.api.component)
	}

	function savedDocumentCount(layout: SerializedDockview | undefined) {
		return Object.values(layout?.panels ?? {}).filter((panel) =>
			isDocumentComponent(panel.contentComponent),
		).length
	}

	/** Documents headed for a dock that isn't laid out yet. */
	const pendingDocuments = new Map<WorkbenchHost, GroupviewPanelState[]>()

	/**
	 * Every region stays laid out, so one emptied by dragging its panels away can take them back;
	 * the toggles show and hide them.
	 */
	const usedRegions = computed(() => [...WORKBENCH_REGIONS])

	/** Whether a region holds anything when the workbench opens; empty ones start hidden. */
	function hasInitialContent(region: WorkbenchRegion) {
		return panelsIn(region).length > 0 || savedDocumentCount(saved?.docks?.[region]) > 0
	}

	const regionSizes = Object.fromEntries(
		WORKBENCH_REGIONS.map((region) => [region, saved?.sizes[region] ?? sizes[region].default]),
	) as Record<WorkbenchRegion, number>

	const visible = Object.fromEntries(
		WORKBENCH_REGIONS.map((region) => [
			region,
			ref(
				!options.hiddenRegions?.includes(region) &&
					(saved?.visible[region] ?? hasInitialContent(region)),
			),
		]),
	) as Record<WorkbenchRegion, Ref<boolean>>

	let grid: GridviewApi | null = null
	let gridDisposables: IDisposable[] = []
	const docks = new Map<WorkbenchHost, { api: DockviewApi; disposables: IDisposable[] }>()
	/** Saved arrangements are only restored the first time each dock comes up. */
	const pendingRestores = new Map(Object.entries(saved?.docks ?? {})) as Map<
		WorkbenchHost,
		SerializedDockview
	>
	/** Where panels being moved into a dock that isn't laid out yet should go once it is. */
	const pendingPositions = new Map<string, AddPanelPositionOptions | undefined>()
	/** Removals made here rather than by the user closing the panel. */
	let ownRemovals = 0

	let saveTimer: ReturnType<typeof setTimeout> | undefined

	function isHorizontal(region: WorkbenchRegion) {
		return region !== 'bottom'
	}

	function measureRegions() {
		for (const region of usedRegions.value) {
			const panel = grid?.getPanel(region)
			if (!panel || !visible[region].value) continue
			const size = isHorizontal(region) ? panel.api.width : panel.api.height
			if (size > 0) regionSizes[region] = size
		}
	}

	function persist() {
		clearTimeout(saveTimer)
		measureRegions()
		const savedDocks: SavedWorkbenchLayout['docks'] = Object.fromEntries(pendingRestores)
		for (const [host, { api }] of docks) savedDocks[host] = api.toJSON()
		saveLayout(options.storageKey, {
			version: LAYOUT_VERSION,
			sizes: { ...regionSizes },
			visible: Object.fromEntries(
				WORKBENCH_REGIONS.map((region) => [region, visible[region].value]),
			),
			docks: savedDocks,
			placements: { ...placements.value },
			closed: [...closed.value],
		})
	}

	function scheduleSave() {
		clearTimeout(saveTimer)
		saveTimer = setTimeout(persist, SAVE_DELAY)
	}

	function applyRegionSize(region: WorkbenchRegion) {
		const panel = grid?.getPanel(region)
		if (!panel) return
		const size = regionSizes[region]
		panel.api.setSize(isHorizontal(region) ? { width: size } : { height: size })
	}

	function addRegion(region: WorkbenchRegion) {
		if (!grid || grid.getPanel(region)) return
		const { minimum } = sizes[region]
		grid.addPanel<WorkbenchDockParams>({
			id: region,
			component: WORKBENCH_DOCK_COMPONENT,
			params: { host: region },
			position: { direction: region === 'bottom' ? 'below' : region, referencePanel: 'center' },
			...(isHorizontal(region) ? { minimumWidth: minimum } : { minimumHeight: minimum }),
		})
		applyRegionSize(region)
		grid.getPanel(region)?.api.setVisible(visible[region].value)
	}

	/** Lays out every region. Left and right go in before the bottom one, so it only spans the center. */
	function addRegions() {
		for (const region of WORKBENCH_REGIONS) addRegion(region)
	}

	function onGridReady({ api }: GridviewReadyEvent) {
		onGridDispose()
		grid = api
		api.addPanel<WorkbenchDockParams>({
			id: 'center',
			component: WORKBENCH_DOCK_COMPONENT,
			params: { host: 'center' },
			minimumWidth: WORKBENCH_CENTER_MINIMUM,
			minimumHeight: WORKBENCH_CENTER_MINIMUM,
			priority: LayoutPriority.High,
		})
		addRegions()
		gridDisposables = [api.onDidLayoutChange(scheduleSave)]
	}

	function onGridDispose() {
		for (const disposable of gridDisposables) disposable.dispose()
		gridDisposables = []
		grid = null
	}

	function withOwnRemovals(action: () => void) {
		ownRemovals += 1
		try {
			action()
		} finally {
			ownRemovals -= 1
		}
	}

	function removeOwn(api: DockviewApi, id: string) {
		const panel = api.getPanel(id)
		if (panel) withOwnRemovals(() => api.removePanel(panel))
	}

	/** Adds the dock's missing workbench panels, drops ones it no longer holds and updates titles. */
	function syncDockPanels(host: WorkbenchHost) {
		const api = docks.get(host)?.api
		if (!api) return
		const wanted = panelsIn(host)
		const ids = new Set(wanted.map((panel) => panel.id))

		for (const panel of [...api.panels]) {
			if (!ids.has(panel.id) && !isDocumentPanel(panel)) removeOwn(api, panel.id)
		}
		for (const panel of wanted) {
			const existing = api.getPanel(panel.id)
			if (existing) {
				existing.api.setTitle(panel.title)
				continue
			}
			const moved = pendingPositions.has(panel.id)
			const position = moved ? pendingPositions.get(panel.id) : defaultPosition(api)
			pendingPositions.delete(panel.id)
			api.addPanel<WorkbenchPanelParams>({
				id: panel.id,
				component: WORKBENCH_PANEL_COMPONENT,
				tabComponent: WORKBENCH_TAB_COMPONENT,
				title: panel.title,
				params: { id: panel.id },
				inactive: !moved,
				position,
			})
		}
		if (!api.activePanel) api.panels[0]?.api.setActive()
	}

	/**
	 * Dockview drops a group once its last panel leaves; an emptied dock gets an empty group back,
	 * so it keeps a tab bar to drop panels onto. Checked once a move settles, since moving a whole
	 * group briefly leaves the dock without any.
	 */
	function keepGroup(api: DockviewApi) {
		queueMicrotask(() => {
			if (api.groups.length === 0) api.addGroup()
		})
	}

	function addDocumentPanel(
		api: DockviewApi,
		state: GroupviewPanelState,
		position?: AddPanelPositionOptions,
	) {
		if (!state.contentComponent || api.getPanel(state.id)) return
		api.addPanel({
			id: state.id,
			component: state.contentComponent,
			tabComponent: state.tabComponent,
			title: state.title,
			params: state.params,
			position: position ?? defaultPosition(api),
		})
		api.getPanel(state.id)?.api.setActive()
	}

	/** The dock a dockview belongs to, by its id. */
	function hostWithView(viewId: string) {
		for (const [host, { api }] of docks) if (api.id === viewId) return host
		return undefined
	}

	function findPanel(id: string) {
		return panels.value.find((candidate) => candidate.id === id)
	}

	/** The dragged panel, when it's one of this workbench's panels or documents in another dock. */
	function draggedPanel(event: Pick<DockviewDndOverlayEvent, 'getData'>) {
		const data = event.getData()
		if (data?.panelId == null) return null
		const host = hostWithView(data.viewId)
		const panel = host ? docks.get(host)?.api.getPanel(data.panelId) : undefined
		if (!host || !panel) return null
		if (findPanel(panel.id)) return { host, panel, document: false }
		if (isDocumentPanel(panel)) return { host, panel, document: true }
		return null
	}

	function acceptForeignPanel(event: DockviewDndOverlayEvent) {
		if (draggedPanel(event)) event.accept()
	}

	function onForeignDrop(host: WorkbenchHost, event: DockviewDidDropEvent) {
		const dragged = draggedPanel(event)
		if (!dragged) return
		const position = dropPosition(event.position, event.group)
		if (dragged.document) void moveDocumentFrom(dragged.host, dragged.panel.id, host, position)
		else movePanel(dragged.panel.id, host, position)
	}

	/** A panel the user closed from its tab (rather than one moved or removed here) is remembered. */
	function onPanelRemoved(host: WorkbenchHost, id: string) {
		if (ownRemovals > 0) return
		queueMicrotask(() => {
			if (docks.get(host)?.api.getPanel(id)) return
			const panel = findPanel(id)
			if (!panel || regionOf(panel) !== host || closed.value.includes(id)) return
			closed.value = [...closed.value, id]
			scheduleSave()
		})
	}

	/** Whether a saved dock only holds documents and workbench panels it still has. */
	function isRestorable(layout: SerializedDockview | undefined, host: WorkbenchHost) {
		if (!layout?.grid || !layout.panels) return false
		if (layout.floatingGroups?.length || layout.popoutGroups?.length) return false
		const ids = new Set(panelsIn(host).map((panel) => panel.id))
		return Object.entries(layout.panels).every(([id, panel]) =>
			panel.contentComponent === WORKBENCH_PANEL_COMPONENT
				? ids.has(id)
				: isDocumentComponent(panel.contentComponent),
		)
	}

	function onDockReady(host: WorkbenchHost, { api }: DockviewReadyEvent) {
		onDockDispose(host)
		const restore = pendingRestores.get(host)
		pendingRestores.delete(host)
		if (isRestorable(restore, host)) {
			withOwnRemovals(() => {
				try {
					api.fromJSON(restore!)
				} catch {
					api.clear()
				}
			})
		}
		for (const state of pendingDocuments.get(host) ?? []) addDocumentPanel(api, state)
		pendingDocuments.delete(host)

		docks.set(host, {
			api,
			disposables: [
				api.onDidLayoutChange(scheduleSave),
				api.onUnhandledDragOver(acceptForeignPanel),
				api.onDidDrop((event) => onForeignDrop(host, event)),
				api.onDidRemovePanel((panel) => onPanelRemoved(host, panel.id)),
				api.onDidRemoveGroup(() => keepGroup(api)),
			],
		})
		syncDockPanels(host)
		if (api.groups.length === 0) api.addGroup()
		documents?.attach(api, host)
	}

	function onDockDispose(host: WorkbenchHost) {
		const entry = docks.get(host)
		if (!entry) return
		documents?.detach(entry.api)
		for (const disposable of entry.disposables) disposable.dispose()
		docks.delete(host)
	}

	watch(openPanels, () => {
		for (const host of docks.keys()) syncDockPanels(host)
	})

	function setRegionVisible(region: WorkbenchRegion, value: boolean) {
		if (visible[region].value === value) return
		if (!value) measureRegions()
		visible[region].value = value
		grid?.getPanel(region)?.api.setVisible(value)
		if (value) applyRegionSize(region)
		scheduleSave()
	}

	function showHost(host: WorkbenchHost) {
		if (isRegion(host)) setRegionVisible(host, true)
	}

	/**
	 * Moves (or reopens) a workbench panel into `host`, at `position` within it or else alongside
	 * its active group's panels, and shows it.
	 */
	function movePanel(id: string, host: WorkbenchHost, position?: AddPanelPositionOptions) {
		const panel = findPanel(id)
		if (!panel) return
		const from = regionOf(panel)
		const fromApi = docks.get(from)?.api
		if (fromApi) removeOwn(fromApi, id)

		const nextPlacements = { ...placements.value }
		if (host === panel.region) delete nextPlacements[id]
		else nextPlacements[id] = host
		placements.value = nextPlacements
		closed.value = closed.value.filter((closedId) => closedId !== id)
		pendingPositions.set(id, position)

		showHost(host)
		syncDockPanels(host)
		docks.get(host)?.api.getPanel(id)?.api.setActive()
		if (from !== host && fromApi) syncDockPanels(from)
		scheduleSave()
	}

	function hostOfDocument(id: string) {
		for (const [host, { api }] of docks) {
			const panel = api.getPanel(id)
			if (panel && isDocumentPanel(panel)) return host
		}
		return undefined
	}

	/**
	 * Moves a document to another dock, which remounts it, so its owner gets to confirm first
	 * (e.g. about unsaved changes). A dock that isn't laid out yet takes it once it is.
	 */
	async function moveDocumentFrom(
		from: WorkbenchHost,
		id: string,
		host: WorkbenchHost,
		position?: AddPanelPositionOptions,
	) {
		if (from === host) return
		if (documents?.confirmMove && !(await documents.confirmMove(id))) return
		const fromApi = docks.get(from)?.api
		const panel = fromApi?.getPanel(id)
		if (!fromApi || !panel) return

		const state = panel.toJSON()
		fromApi.removePanel(panel)
		showHost(host)

		const target = docks.get(host)?.api
		if (target) {
			addDocumentPanel(target, state, position)
		} else {
			pendingDocuments.set(host, [...(pendingDocuments.get(host) ?? []), state])
		}
		scheduleSave()
	}

	function moveDocument(id: string, host: WorkbenchHost) {
		const from = hostOfDocument(id)
		if (from) void moveDocumentFrom(from, id, host)
	}

	/** Splits a panel off into its own group beside the rest of its group, within its dock. */
	function splitPanel(id: string, side: 'right' | 'bottom') {
		const panel = findPanel(id)
		if (!panel) return
		const dockPanel = docks.get(regionOf(panel))?.api.getPanel(id)
		if (!dockPanel || dockPanel.group.panels.length < 2) return
		dockPanel.api.moveTo({ group: dockPanel.group, position: side })
	}

	function closePanel(id: string) {
		const panel = findPanel(id)
		if (!panel || closed.value.includes(id)) return
		const api = docks.get(regionOf(panel))?.api
		closed.value = [...closed.value, id]
		if (api) removeOwn(api, id)
		scheduleSave()
	}

	function revealPanel(id: string) {
		const panel = findPanel(id)
		if (!panel) return
		if (closed.value.includes(id)) {
			movePanel(id, regionOf(panel))
			return
		}
		showHost(regionOf(panel))
		docks.get(regionOf(panel))?.api.getPanel(id)?.api.setActive()
	}

	/** Restores the defaults, with every document back in the center. */
	function resetLayout() {
		pendingRestores.clear()
		pendingPositions.clear()
		pendingDocuments.clear()
		placements.value = {}
		closed.value = []
		for (const region of WORKBENCH_REGIONS) {
			regionSizes[region] = sizes[region].default
			visible[region].value = true
			grid?.getPanel(region)?.api.setVisible(true)
			applyRegionSize(region)
		}
		for (const [host, { api }] of docks) {
			withOwnRemovals(() => api.clear())
			syncDockPanels(host)
			if (api.groups.length === 0) api.addGroup()
		}
		documents?.restore()
		persist()
	}

	onBeforeUnmount(() => {
		persist()
		onGridDispose()
		for (const host of [...docks.keys()]) onDockDispose(host)
	})

	return {
		slots: options.slots,
		documents,
		panels,
		closedPanels,
		usedRegions,
		visible,
		regionOf,
		isOpen: (id) => !closed.value.includes(id),
		setRegionVisible,
		toggleRegion: (region) => setRegionVisible(region, !visible[region].value),
		movePanel,
		moveDocument,
		hostOfDocument,
		splitPanel,
		closePanel,
		revealPanel,
		resetLayout,
		canSplit: (id) => {
			const panel = findPanel(id)
			const dockPanel = panel ? docks.get(regionOf(panel))?.api.getPanel(id) : undefined
			return (dockPanel?.group.panels.length ?? 0) > 1
		},
		onGridReady,
		onDockReady,
		onDockDispose,
	}
}
