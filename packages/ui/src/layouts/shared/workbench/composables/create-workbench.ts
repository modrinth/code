import type {
	AddPanelPositionOptions,
	DockviewApi,
	DockviewDidDropEvent,
	DockviewDndOverlayEvent,
	DockviewGroupPanel,
	DockviewIDisposable,
	DockviewReadyEvent,
	GridviewApi,
	GridviewReadyEvent,
	GroupviewPanelState,
	IDockviewPanel,
	Position,
	SerializedDockview,
} from 'dockview-vue'
import { LayoutPriority } from 'dockview-vue'
import {
	computed,
	type MaybeRefOrGetter,
	onBeforeUnmount,
	type Ref,
	ref,
	shallowRef,
	type Slots,
	toValue,
	watch,
} from 'vue'

import type { ButtonMenuOption } from '#ui/components/base/buttons/types'

import type { WorkbenchContext } from '../providers/workbench'
import {
	DEFAULT_WORKBENCH_REGION_SIZES,
	WORKBENCH_CENTER_MINIMUM,
	WORKBENCH_HOSTS,
	WORKBENCH_MODES,
	WORKBENCH_REGIONS,
	type WorkbenchDocuments,
	type WorkbenchHost,
	type WorkbenchMode,
	type WorkbenchModeName,
	type WorkbenchPanel,
	type WorkbenchRegion,
	type WorkbenchRegionSizes,
	type WorkbenchToggleLocation,
	type WorkbenchVariant,
} from '../types'
import {
	evenOutDockSash,
	findCornerGroups,
	registerEvenSplits,
	registerNoSplits,
	registerPanelOnlyDrops,
	registerWrappedTabs,
} from './dock-behaviours'
import { createDockPopouts, type DockPopouts, usePopoutServices } from './dock-popouts'
import { useWorkbenchMenus } from './use-menus'
import { useWorkbenchSelection } from './use-tab-selection'
import { useWorkbenchWindows } from './use-windows'

export const WORKBENCH_DOCK_COMPONENT = 'workbenchDock'
export const WORKBENCH_PANEL_COMPONENT = 'workbenchPanel'
export const WORKBENCH_TAB_COMPONENT = 'workbenchTab'

const STORAGE_PREFIX = 'workbench-layout:'
const LAYOUT_VERSION = 4
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
	variant?: WorkbenchVariant
	/** Whether panels can be popped out, for modes that allow it. */
	popouts?: boolean
	/** Where the region toggles sit, for modes with a toolbar. */
	toggles?: WorkbenchToggleLocation
	sizes: Partial<Record<WorkbenchRegion, number>>
	visible: Partial<Record<WorkbenchRegion, boolean>>
	/** Each dock's arrangements, by the set of workbench panels it held at the time. */
	docks: Partial<Record<WorkbenchHost, Record<string, SerializedDockview>>>
	/** Panels moved away from the dock they're declared in. */
	placements: Record<string, WorkbenchHost>
	/** Panels the user closed. */
	closed: string[]
	/** Panels the layout has known, so ones closed by default only start closed the first time. */
	seen?: string[]
}

export interface WorkbenchOptions {
	panels: MaybeRefOrGetter<WorkbenchPanel[]>
	/**
	 * Identifies the layout in local storage. `null` keeps it in memory only, e.g. for a one-off
	 * view that shouldn't touch the user's own arrangement.
	 */
	storageKey: string | null
	/** How much of the layout users can change; a preset's name, or the features themselves. */
	mode?: WorkbenchModeName | WorkbenchMode
	/** The look used until the user picks one. */
	variant?: WorkbenchVariant
	sizes?: Partial<WorkbenchRegionSizes>
	/** Minimum size of the center, in both directions. */
	centerMinimum?: number
	/** Regions hidden when opened, regardless of how they were left. */
	hiddenRegions?: WorkbenchRegion[]
	documents?: WorkbenchDocuments
	/**
	 * Whether the panels are settled (e.g. their data has loaded). Until then the saved layout is
	 * neither restored nor overwritten, so it isn't lost to a partial set of panels.
	 */
	ready?: MaybeRefOrGetter<boolean>
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

/**
 * The position, unless it's beside a group the dock no longer has (e.g. one dropped on before the
 * dock loaded another arrangement), in which case the dock's default one.
 */
function usablePosition(api: DockviewApi, position: AddPanelPositionOptions | undefined) {
	const group = position && 'referenceGroup' in position ? position.referenceGroup : undefined
	if (!group || typeof group === 'string') return position
	return api.getGroup(group.id) === group ? position : defaultPosition(api)
}

/** Alongside the panels of the dock's active (or first) group in the main window. */
function defaultPosition(api: DockviewApi): AddPanelPositionOptions | undefined {
	const inGrid = (group?: DockviewGroupPanel | null) => group?.api.location.type === 'grid'
	const group = inGrid(api.activeGroup)
		? api.activeGroup
		: api.groups.find((candidate) => inGrid(candidate))
	return group ? { referenceGroup: group, direction: 'within' } : undefined
}

function isRegion(host: WorkbenchHost): host is WorkbenchRegion {
	return host !== 'center'
}

function resolveMode(mode: WorkbenchOptions['mode']): WorkbenchMode {
	if (!mode) return WORKBENCH_MODES.explorer
	return typeof mode === 'string' ? WORKBENCH_MODES[mode] : mode
}

/**
 * Lays the workbench out as a grid: its center, with resizable regions to the left, right and
 * below it. Each is its own dockview (a "dock"), so its panels can be tabbed and split within it,
 * and (as the mode allows) dragged or moved from their menu between docks, or popped out into
 * windows of their own. Besides its own panels, it hosts `documents` (e.g. file tabs) whose owner
 * creates and closes them. Region sizes and visibility, every dock's arrangement (per set of
 * panels it held), where panels were moved or closed, and the look are persisted per
 * `storageKey`.
 *
 * Create it in the host's setup and hand it to `WorkbenchLayout`, so the host can use it (e.g.
 * provide its own context from it) before the layout renders.
 */
export function createWorkbench(options: WorkbenchOptions): WorkbenchContext {
	const mode = resolveMode(options.mode)
	const sizes: WorkbenchRegionSizes = { ...DEFAULT_WORKBENCH_REGION_SIZES, ...options.sizes }
	const centerMinimum = options.centerMinimum ?? WORKBENCH_CENTER_MINIMUM
	const saved = loadLayout(options.storageKey)
	const documents = options.documents
	const regions = WORKBENCH_REGIONS.filter((region) => mode.regions.includes(region))
	const popoutHosts: WorkbenchHost[] = mode.popouts ? (mode.popouts.docks ?? ['center']) : []

	const root = shallowRef<HTMLElement | null>(null)
	const windows = useWorkbenchWindows()
	const selection = mode.tabSelection ? useWorkbenchSelection(windows) : null
	const popoutServices = popoutHosts.length ? usePopoutServices() : null

	const variant = ref<WorkbenchVariant>(saved?.variant ?? options.variant ?? 'square')
	const popoutsEnabled = ref(saved?.popouts ?? true)
	const toggleLocation = ref<WorkbenchToggleLocation>(
		saved?.toggles ?? (mode.cornerToggles ? 'tabs' : 'menu'),
	)
	/** Modes without a toolbar always keep their toggles in the tab bar, having nowhere else for them. */
	const cornerToggles = computed(() => !mode.toolbar || toggleLocation.value === 'tabs')

	const documentLoaded = ref(typeof document === 'undefined' || document.readyState === 'complete')
	function onDocumentLoad() {
		documentLoaded.value = true
	}
	if (!documentLoaded.value) window.addEventListener('load', onDocumentLoad, { once: true })

	const isReady = computed(() => (toValue(options.ready) ?? true) && documentLoaded.value)
	/** Set while the page unloads, after its layout was saved, so the teardown doesn't overwrite it. */
	let leavingPage = false

	const panels = computed(() =>
		toValue(options.panels).filter(
			(panel) => panel.region === 'center' || regions.includes(panel.region),
		),
	)
	const placements = ref<Record<string, WorkbenchHost>>({ ...saved?.placements })
	/** Panels starting closed, unless the saved layout already decided whether they're open. */
	function closedByDefault() {
		return toValue(options.panels)
			.filter((panel) => panel.closedByDefault && !saved?.seen?.includes(panel.id))
			.map((panel) => panel.id)
	}
	const closed = ref<string[]>(
		mode.closePanels ? [...new Set([...(saved?.closed ?? []), ...closedByDefault()])] : [],
	)

	function regionOf(panel: WorkbenchPanel): WorkbenchHost {
		const placed = mode.movePanels ? placements.value[panel.id] : undefined
		return placed && (placed === 'center' || regions.includes(placed)) ? placed : panel.region
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

	/** Identifies the set of workbench panels a dock holds, which its saved arrangements are kept by. */
	function panelSetKey(host: WorkbenchHost) {
		return panelsIn(host)
			.map((panel) => panel.id)
			.sort()
			.join(',')
	}

	function findPanel(id: string) {
		return panels.value.find((candidate) => candidate.id === id)
	}

	function isDocumentComponent(component: string | undefined) {
		return !!documents && !!component && component in documents.components
	}

	function isDocumentPanel(panel: IDockviewPanel) {
		return isDocumentComponent(panel.api.component)
	}

	function hasSavedDocuments(host: WorkbenchHost) {
		return Object.values(saved?.docks?.[host] ?? {}).some((layout) =>
			Object.values(layout.panels ?? {}).some((panel) =>
				isDocumentComponent(panel.contentComponent),
			),
		)
	}

	/**
	 * Every region of the mode stays laid out, so one emptied by dragging its panels away can take
	 * them back; the toggles show and hide them.
	 */
	const usedRegions = computed(() => [...regions])

	const regionSizes = Object.fromEntries(
		WORKBENCH_REGIONS.map((region) => [region, saved?.sizes[region] ?? sizes[region].default]),
	) as Record<WorkbenchRegion, number>

	const visible = Object.fromEntries(
		WORKBENCH_REGIONS.map((region) => [
			region,
			ref(
				regions.includes(region) &&
					!options.hiddenRegions?.includes(region) &&
					(saved?.visible[region] ?? (panelsIn(region).length > 0 || hasSavedDocuments(region))),
			),
		]),
	) as Record<WorkbenchRegion, Ref<boolean>>

	/** How wide (or tall, for the bottom) each region is shown, 0 while hidden. */
	const regionExtents = Object.fromEntries(
		WORKBENCH_REGIONS.map((region) => [
			region,
			ref(visible[region].value ? regionSizes[region] : 0),
		]),
	) as Record<WorkbenchRegion, Ref<number>>

	/** The center's top-left and top-right groups, whose tab bars hold the corner region toggles. */
	const cornerGroups = ref<{ left?: string; right?: string }>({})

	let grid: GridviewApi | null = null
	let gridDisposables: DockviewIDisposable[] = []

	interface Dock {
		api: DockviewApi
		disposables: DockviewIDisposable[]
		/** The panel set the dock's arrangement was last loaded or saved for. */
		key?: string
		popouts?: DockPopouts
		wrappedTabs?: DockviewIDisposable
	}

	const docks = new Map<WorkbenchHost, Dock>()
	/** Saved arrangements per dock, by panel set, updated as docks are saved. */
	const savedDockLayouts = Object.fromEntries(
		WORKBENCH_HOSTS.map((host) => [host, new Map(Object.entries(saved?.docks?.[host] ?? {}))]),
	) as Record<WorkbenchHost, Map<string, SerializedDockview>>
	/** Where panels being moved into a dock should go once it adds them. */
	const pendingPositions = new Map<string, AddPanelPositionOptions | undefined>()
	/** Documents headed for a dock that isn't set up yet. */
	const pendingDocuments = new Map<WorkbenchHost, GroupviewPanelState[]>()
	/** Removals made here rather than by the user closing the panel. */
	let ownRemovals = 0

	let saveTimer: ReturnType<typeof setTimeout> | undefined

	function isHorizontal(region: WorkbenchRegion) {
		return region !== 'bottom'
	}

	/**
	 * Whether the grid has been laid out at its real size. Until then the sizes dockview gives the
	 * regions aren't theirs, so they aren't measured (and saved over the ones they should have).
	 */
	let gridSized = false
	/** Set while the regions' sizes are applied one by one, so the in-between layouts aren't measured. */
	let applyingSizes = false

	function measureRegions() {
		if (!gridSized || applyingSizes) return
		for (const region of regions) {
			const panel = grid?.getPanel(region)
			if (!panel) continue
			const size = isHorizontal(region) ? panel.api.width : panel.api.height
			if (visible[region].value && size > 0) regionSizes[region] = size
			regionExtents[region].value = visible[region].value ? size : 0
		}
	}

	function dockLayout(dock: Dock) {
		return dock.popouts?.currentLayout() ?? dock.api.toJSON()
	}

	function persist() {
		clearTimeout(saveTimer)
		if (!isReady.value || leavingPage) return
		measureRegions()
		for (const [host, dock] of docks) {
			if (dock.key !== undefined) savedDockLayouts[host].set(dock.key, dockLayout(dock))
		}
		saveLayout(options.storageKey, {
			version: LAYOUT_VERSION,
			variant: variant.value,
			popouts: popoutsEnabled.value,
			toggles: toggleLocation.value,
			sizes: { ...regionSizes },
			visible: Object.fromEntries(regions.map((region) => [region, visible[region].value])),
			docks: Object.fromEntries(
				WORKBENCH_HOSTS.map((host) => [host, Object.fromEntries(savedDockLayouts[host])]),
			),
			placements: { ...placements.value },
			closed: [...closed.value],
			seen: panels.value.map((panel) => panel.id),
		})
	}

	function scheduleSave() {
		clearTimeout(saveTimer)
		if (leavingPage) return
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
			/*
			 * Offered space last, so the center (and the branch holding it with the bottom region)
			 * takes up the workbench being resized, rather than a region growing past its saved size.
			 */
			priority: LayoutPriority.Low,
		})
		applyRegionSize(region)
		grid.getPanel(region)?.api.setVisible(visible[region].value)
	}

	function onGridReady({ api }: GridviewReadyEvent) {
		onGridDispose()
		grid = api
		api.addPanel<WorkbenchDockParams>({
			id: 'center',
			component: WORKBENCH_DOCK_COMPONENT,
			params: { host: 'center' },
			minimumWidth: centerMinimum,
			minimumHeight: centerMinimum,
			priority: LayoutPriority.High,
		})
		for (const region of regions) addRegion(region)
		gridDisposables = [api.onDidLayoutChange(onGridLayout)]
		onGridLayout()
	}

	/** Gives the regions their sizes once the grid has its own, then keeps measuring them. */
	function onGridLayout() {
		if (!grid) return
		if (!gridSized) {
			if (grid.width <= 0 || grid.height <= 0) return
			gridSized = true
			applyRegionSizes()
		}
		measureRegions()
		scheduleSave()
	}

	function applyRegionSizes() {
		applyingSizes = true
		try {
			for (const region of regions) applyRegionSize(region)
		} finally {
			applyingSizes = false
		}
		measureRegions()
	}

	function onGridDispose() {
		for (const disposable of gridDisposables) disposable.dispose()
		gridDisposables = []
		grid = null
		gridSized = false
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

	/** Groups of headerless docks hide their tab bar. */
	function applyHeaders(host: WorkbenchHost) {
		const api = docks.get(host)?.api
		if (!api || !mode.headerless.includes(host)) return
		for (const group of api.groups) group.header.hidden = true
	}

	/**
	 * Adds the dock's missing workbench panels, drops ones it no longer holds (one popped out is
	 * released from its window, which stays open for it to return to) and updates titles.
	 */
	function syncDockPanels(host: WorkbenchHost) {
		const dock = docks.get(host)
		if (!dock || dock.key === undefined) return
		const { api } = dock
		const wanted = panelsIn(host)
		const ids = new Set(wanted.map((panel) => panel.id))

		for (const panel of [...api.panels]) {
			if (ids.has(panel.id) || isDocumentPanel(panel)) continue
			withOwnRemovals(() => {
				if (!dock.popouts?.release(panel)) api.removePanel(panel)
			})
		}
		for (const panel of wanted) {
			const existing = api.getPanel(panel.id)
			if (existing) {
				existing.api.setTitle(panel.title)
				continue
			}
			const moved = pendingPositions.has(panel.id)
			const position = moved
				? usablePosition(api, pendingPositions.get(panel.id))
				: (dock.popouts?.positionFor(panel.id) ?? defaultPosition(api))
			pendingPositions.delete(panel.id)
			api.addPanel<WorkbenchPanelParams>({
				id: panel.id,
				component: WORKBENCH_PANEL_COMPONENT,
				tabComponent: WORKBENCH_TAB_COMPONENT,
				title: panel.title,
				params: { id: panel.id },
				renderer: 'always',
				minimumHeight: panel.minimumHeight,
				inactive: !moved,
				position,
			})
		}
		if (!api.activePanel) api.panels[0]?.api.setActive()
		if (!hasGridGroup(api)) api.addGroup()
		applyHeaders(host)
	}

	/**
	 * Whether the group shows in its dock. A group popped out into a window leaves a hidden one
	 * behind in the dock, for it to return to once the window closes.
	 */
	function isShownGridGroup(group: DockviewGroupPanel) {
		return group.api.location.type === 'grid' && group.api.isVisible
	}

	function hasGridGroup(api: DockviewApi) {
		return api.groups.some(isShownGridGroup)
	}

	/** Empty groups added to keep an emptied dock's tab bar, until another group shows again. */
	const placeholderGroups = new WeakSet<DockviewGroupPanel>()

	/**
	 * Dockview drops a group once its last panel leaves, and a popped out group leaves only a
	 * hidden one behind; an emptied dock gets an empty group, so it keeps a tab bar to drop panels
	 * onto, which goes again once a popped out group returns. Checked once a move settles, since
	 * moving a whole group briefly leaves the dock without any.
	 */
	function keepGroup(api: DockviewApi) {
		queueMicrotask(() => {
			const shown = api.groups.filter(isShownGridGroup)
			if (shown.length === 0) {
				placeholderGroups.add(api.addGroup())
				return
			}
			if (shown.length < 2) return
			for (const group of shown) {
				if (placeholderGroups.has(group) && group.panels.length === 0) api.removeGroup(group)
			}
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
		if (mode.movePanels && draggedPanel(event)) event.accept()
	}

	function onForeignDrop(host: WorkbenchHost, event: DockviewDidDropEvent) {
		if (!mode.movePanels) return
		const dragged = draggedPanel(event)
		if (!dragged) return
		const position = dropPosition(event.position, event.group)
		if (dragged.document) void moveDocumentFrom(dragged.host, dragged.panel.id, host, position)
		else movePanel(dragged.panel.id, host, position)
	}

	/** A panel the user closed from its tab (rather than one moved or removed here) is remembered. */
	function onPanelRemoved(host: WorkbenchHost, id: string) {
		if (ownRemovals > 0 || !mode.closePanels) return
		queueMicrotask(() => {
			if (docks.get(host)?.api.getPanel(id)) return
			const panel = findPanel(id)
			if (!panel || regionOf(panel) !== host || closed.value.includes(id)) return
			closed.value = [...closed.value, id]
			scheduleSave()
		})
	}

	/** Whether a saved arrangement only holds documents and workbench panels the dock still has. */
	function isRestorable(layout: SerializedDockview | undefined, host: WorkbenchHost) {
		if (!layout?.grid || !layout.panels) return false
		if (layout.floatingGroups?.length) return false
		if (layout.popoutGroups?.length && !popoutHosts.includes(host)) return false
		const ids = new Set(panelsIn(host).map((panel) => panel.id))
		return Object.entries(layout.panels).every(([id, panel]) =>
			panel.contentComponent === WORKBENCH_PANEL_COMPONENT
				? ids.has(id)
				: isDocumentComponent(panel.contentComponent),
		)
	}

	/** Loads the dock's saved arrangement for its current panel set, if there's a usable one. */
	function restoreDock(host: WorkbenchHost, dock: Dock) {
		const key = dock.key ?? ''
		const layout = savedDockLayouts[host].get(key)
		if (!layout || !isRestorable(layout, host)) return false
		withOwnRemovals(() => {
			try {
				if (dock.popouts) dock.popouts.restore(layout)
				else dock.api.fromJSON(layout, { reuseExistingPanels: true })
			} catch {
				dock.api.clear()
				savedDockLayouts[host].delete(key)
			}
		})
		return true
	}

	function updateCornerGroups() {
		const api = docks.get('center')?.api
		cornerGroups.value = api ? findCornerGroups(api) : {}
	}

	/** Brings a dock up once both it and the workbench's panels are ready. */
	function initializeDock(host: WorkbenchHost) {
		const dock = docks.get(host)
		if (!dock || dock.key !== undefined || !isReady.value) return
		dock.key = panelSetKey(host)
		restoreDock(host, dock)
		for (const state of pendingDocuments.get(host) ?? []) addDocumentPanel(dock.api, state)
		pendingDocuments.delete(host)
		syncDockPanels(host)
		documents?.attach(dock.api, host)
		if (host === 'center') updateCornerGroups()
	}

	/**
	 * Follows a change of the dock's panel set: its arrangement is kept for the set it had, and the
	 * one kept for the new set (if any) is loaded, before the panels themselves are synced.
	 */
	function updateDock(host: WorkbenchHost) {
		const dock = docks.get(host)
		if (!dock) return
		if (dock.key === undefined) {
			initializeDock(host)
			return
		}
		const key = panelSetKey(host)
		if (key !== dock.key) {
			savedDockLayouts[host].set(dock.key, dockLayout(dock))
			dock.key = key
			if (!dock.popouts?.restoring() && !dock.api.getPopouts().length && restoreDock(host, dock)) {
				documents?.restore()
			}
		}
		syncDockPanels(host)
	}

	function applyVariant(dock: Dock) {
		const wrapped = variant.value === 'rounded'
		if (wrapped && !dock.wrappedTabs) dock.wrappedTabs = registerWrappedTabs(dock.api)
		if (!wrapped && dock.wrappedTabs) {
			dock.wrappedTabs.dispose()
			dock.wrappedTabs = undefined
		}
	}

	/** Bumped as docks come and go, so the combined popout state below follows them. */
	const dockVersion = ref(0)

	function onDockReady(host: WorkbenchHost, { api }: DockviewReadyEvent) {
		onDockDispose(host)
		const dock: Dock = { api, disposables: [] }
		if (popoutHosts.includes(host) && mode.popouts && popoutServices) {
			dock.popouts = createDockPopouts({
				api,
				services: popoutServices,
				options: mode.popouts,
				windows,
				root: () => root.value,
				canChange: () => isReady.value && !leavingPage,
				titleOf: (id) => findPanel(id)?.title ?? id,
				onChange: scheduleSave,
				dragPanels: selection?.getDragPanels,
				keepDrag: selection?.keepDragForPopout,
			})
		}
		dock.disposables.push(
			api.onDidLayoutChange(scheduleSave),
			api.onDidActivePanelChange(scheduleSave),
			api.onUnhandledDragOver(acceptForeignPanel),
			api.onDidDrop((event) => onForeignDrop(host, event)),
			api.onDidRemovePanel((panel) => onPanelRemoved(host, panel.id)),
			api.onDidRemoveGroup(() => keepGroup(api)),
			api.onDidLayoutChange(() => keepGroup(api)),
			api.onDidAddGroup(() => queueMicrotask(() => applyHeaders(host))),
			registerPanelOnlyDrops(api),
			registerEvenSplits(api, () => isReady.value),
		)
		if (!mode.splitPanels || mode.headerless.includes(host)) {
			dock.disposables.push(registerNoSplits(api))
		}
		if (selection) dock.disposables.push(selection.register(api))
		if (host === 'center') dock.disposables.push(api.onDidLayoutChange(updateCornerGroups))
		docks.set(host, dock)
		dockVersion.value += 1
		applyVariant(dock)
		initializeDock(host)
	}

	function onDockDispose(host: WorkbenchHost) {
		const dock = docks.get(host)
		if (!dock) return
		documents?.detach(dock.api)
		if (dock.key !== undefined && !leavingPage) {
			savedDockLayouts[host].set(dock.key, dockLayout(dock))
		}
		for (const disposable of dock.disposables) disposable.dispose()
		dock.popouts?.dispose()
		dock.wrappedTabs?.dispose()
		docks.delete(host)
		dockVersion.value += 1
	}

	watch(isReady, (ready) => {
		if (!ready) return
		for (const host of docks.keys()) initializeDock(host)
	})
	watch(openPanels, () => {
		if (!isReady.value) return
		for (const host of docks.keys()) updateDock(host)
	})
	watch(variant, () => {
		for (const dock of docks.values()) applyVariant(dock)
		scheduleSave()
	})
	watch(toggleLocation, scheduleSave)
	watch(popoutsEnabled, (enabled) => {
		if (!enabled) closePopouts()
		scheduleSave()
	})

	function setRegionVisible(region: WorkbenchRegion, value: boolean) {
		if (!regions.includes(region) || visible[region].value === value) return
		if (!value) measureRegions()
		visible[region].value = value
		grid?.getPanel(region)?.api.setVisible(value)
		if (value) applyRegionSize(region)
		measureRegions()
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
		if (!panel || (host !== 'center' && !regions.includes(host))) return
		const from = regionOf(panel)
		if (from !== host && !mode.movePanels) return
		const fromApi = docks.get(from)?.api
		if (fromApi) removeOwn(fromApi, id)

		const { [id]: _previous, ...otherPlacements } = placements.value
		placements.value = host === panel.region ? otherPlacements : { ...otherPlacements, [id]: host }
		closed.value = closed.value.filter((closedId) => closedId !== id)
		pendingPositions.set(id, position)

		showHost(host)
		updateDock(host)
		docks.get(host)?.api.getPanel(id)?.api.setActive()
		if (from !== host) updateDock(from)
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
	 * (e.g. about unsaved changes). A dock that isn't set up yet takes it once it is.
	 */
	async function moveDocumentFrom(
		from: WorkbenchHost,
		id: string,
		host: WorkbenchHost,
		position?: AddPanelPositionOptions,
	) {
		if (from === host || !mode.movePanels) return
		if (documents?.confirmMove && !(await documents.confirmMove(id))) return
		const fromApi = docks.get(from)?.api
		const panel = fromApi?.getPanel(id)
		if (!fromApi || !panel) return

		const state = panel.toJSON()
		fromApi.removePanel(panel)
		showHost(host)

		const target = docks.get(host)
		if (target && target.key !== undefined) {
			addDocumentPanel(target.api, state, position)
		} else {
			pendingDocuments.set(host, [...(pendingDocuments.get(host) ?? []), state])
		}
		scheduleSave()
	}

	function moveDocument(id: string, host: WorkbenchHost) {
		const from = hostOfDocument(id)
		if (from) void moveDocumentFrom(from, id, host)
	}

	function dockPanelOf(id: string) {
		const panel = findPanel(id)
		const dock = panel ? docks.get(regionOf(panel)) : undefined
		return dock ? { dock, panel: dock.api.getPanel(id) } : undefined
	}

	/** Splits a panel off into its own group beside the rest of its group, within its dock. */
	function splitPanel(id: string, side: 'right' | 'bottom') {
		if (!mode.splitPanels) return
		const panel = dockPanelOf(id)?.panel
		if (!panel || panel.group.panels.length < 2) return
		panel.api.moveTo({ group: panel.group, position: side })
	}

	function closePanel(id: string) {
		const panel = findPanel(id)
		if (!mode.closePanels || !panel || closed.value.includes(id)) return
		const api = docks.get(regionOf(panel))?.api
		closed.value = [...closed.value, id]
		if (api) removeOwn(api, id)
		scheduleSave()
	}

	/** Shows a panel: opening it if closed, showing its region, or bringing its popout window forward. */
	function revealPanel(id: string) {
		const panel = findPanel(id)
		if (!panel) return
		if (closed.value.includes(id)) {
			movePanel(id, regionOf(panel))
			return
		}
		const found = dockPanelOf(id)
		if (found?.panel && found.dock.popouts?.reveal(found.panel)) return
		showHost(regionOf(panel))
		found?.panel?.api.setActive()
	}

	/** Brings the first panel of every group to the front, e.g. when moving on to another subject. */
	function resetActiveTabs() {
		for (const { api } of docks.values()) {
			for (const group of api.groups) {
				const first = group.panels[0]
				if (first) group.model.openPanel(first, { skipSetGroupActive: true })
			}
		}
	}

	/** Restores the defaults, with every popout docked and every document back in the center. */
	function resetLayout() {
		for (const dock of docks.values()) {
			if (dock.popouts?.restoring() || dock.popouts?.poppingOut.value.length) return
		}
		for (const dock of docks.values()) dock.popouts?.closeAll()
		pendingPositions.clear()
		pendingDocuments.clear()
		placements.value = {}
		closed.value = mode.closePanels
			? panels.value.filter((panel) => panel.closedByDefault).map((panel) => panel.id)
			: []
		for (const host of WORKBENCH_HOSTS) savedDockLayouts[host].clear()
		for (const region of regions) {
			regionSizes[region] = sizes[region].default
			visible[region].value = true
			grid?.getPanel(region)?.api.setVisible(true)
		}
		applyRegionSizes()
		for (const [host, dock] of docks) {
			withOwnRemovals(() => dock.api.clear())
			dock.key = panelSetKey(host)
			syncDockPanels(host)
		}
		documents?.restore()
		measureRegions()
		persist()
	}

	function popoutDocks() {
		void dockVersion.value
		return [...docks.values()].flatMap((dock) => (dock.popouts ? [dock.popouts] : []))
	}

	const detachedPanels = computed(() => popoutDocks().flatMap((popouts) => popouts.detached.value))
	const hasPopouts = computed(() => popoutDocks().some((popouts) => popouts.hasPopouts.value))
	const poppingOut = computed(() => popoutDocks().flatMap((popouts) => popouts.poppingOut.value))

	function closePopouts() {
		for (const popouts of popoutDocks()) popouts.closeAll()
	}

	/**
	 * A double click on a sash evens out the groups on either side of it within a dock, or returns
	 * the region beside it in the grid to its default size.
	 */
	function onSashDoubleClick(event: MouseEvent) {
		if (!(event.target instanceof HTMLElement)) return
		const sash = event.target.closest('.dv-sash:not(.dv-disabled)')
		if (!sash) return
		const dockElement = sash.closest<HTMLElement>('[data-workbench-host]')
		if (dockElement) {
			const dock = docks.get(dockElement.dataset.workbenchHost as WorkbenchHost)
			if (!dock || !mode.splitPanels) return
			let evened = false
			withOwnRemovals(() => (evened = evenOutDockSash(dock.api, sash)))
			if (!evened) return
		} else {
			const container = sash.parentElement
			const views = container?.parentElement?.querySelector(':scope > .dv-view-container')
			if (!container || !views) return
			const index = Array.from(container.children).indexOf(sash)
			const region = [views.children[index], views.children[index + 1]]
				.map((view) => {
					const hosts = [...(view?.querySelectorAll<HTMLElement>('[data-workbench-host]') ?? [])]
					return hosts.length === 1 ? (hosts[0].dataset.workbenchHost as WorkbenchHost) : undefined
				})
				.find((host): host is WorkbenchRegion => !!host && isRegion(host))
			if (!region || !visible[region].value) return
			regionSizes[region] = sizes[region].default
			applyRegionSize(region)
		}
		event.preventDefault()
		event.stopPropagation()
		scheduleSave()
	}

	/**
	 * Saves as the page unloads, so changes made just before aren't lost to the save delay, and
	 * before Dockview's unload handlers return popped out panels to the main window.
	 */
	function onBeforeUnload() {
		persist()
		leavingPage = true
		clearTimeout(saveTimer)
	}

	function onPageShow() {
		leavingPage = false
	}

	function resumeAfterCancelledUnload(event: Event) {
		if (event.currentTarget === window) leavingPage = false
	}

	/** Refreshing a popout would reload it as the main page, so it's kept from doing so. */
	function preventPopoutRefresh(event: KeyboardEvent) {
		if (event.currentTarget === window) return
		const refresh =
			event.key === 'F5' ||
			((event.ctrlKey || event.metaKey) &&
				!event.altKey &&
				(event.code === 'KeyR' || event.key.toLowerCase() === 'r'))
		if (!refresh) return
		event.preventDefault()
		event.stopImmediatePropagation()
	}

	if (popoutHosts.length) {
		windows.listen('keydown', preventPopoutRefresh, { capture: true })
		windows.listen('pointerdown', resumeAfterCancelledUnload, { capture: true })
		windows.listen('keydown', resumeAfterCancelledUnload, { capture: true })
	} else {
		window.addEventListener('pointerdown', resumeAfterCancelledUnload, { capture: true })
		window.addEventListener('keydown', resumeAfterCancelledUnload, { capture: true })
	}
	window.addEventListener('beforeunload', onBeforeUnload, { capture: true })
	window.addEventListener('pagehide', onBeforeUnload)
	window.addEventListener('pageshow', onPageShow)

	onBeforeUnmount(() => {
		persist()
		clearTimeout(saveTimer)
		window.removeEventListener('load', onDocumentLoad)
		window.removeEventListener('beforeunload', onBeforeUnload, true)
		window.removeEventListener('pagehide', onBeforeUnload)
		window.removeEventListener('pageshow', onPageShow)
		window.removeEventListener('pointerdown', resumeAfterCancelledUnload, true)
		window.removeEventListener('keydown', resumeAfterCancelledUnload, true)
		onGridDispose()
		for (const host of [...docks.keys()]) onDockDispose(host)
	})

	let openMenu: ((event: MouseEvent, menuOptions: ButtonMenuOption[]) => void) | undefined

	const context: WorkbenchContext = {
		slots: {} as Slots,
		root,
		mode,
		variant,
		setVariant: (value) => (variant.value = value),
		canUsePopouts: popoutHosts.length > 0,
		popoutsEnabled,
		setPopoutsEnabled: (value) => (popoutsEnabled.value = value),
		toggleLocation,
		setToggleLocation: (value) => (toggleLocation.value = value),
		cornerToggles,
		documents,
		panels,
		closedPanels,
		usedRegions,
		visible,
		regionExtents,
		cornerGroups,
		selectedPanels: selection?.selectedPanels ?? shallowRef<readonly string[]>([]),
		detachedPanels,
		hasPopouts,
		poppingOut,
		popoutPanel: (id) =>
			popoutsEnabled.value ? dockPanelOf(id)?.dock.popouts?.popoutPanel(id) : undefined,
		dockPanel: (id) => dockPanelOf(id)?.dock.popouts?.dock(id),
		closePopouts,
		canPopout: (id) => {
			if (!popoutsEnabled.value) return false
			const found = dockPanelOf(id)
			return !!found?.dock.popouts && found.panel?.api.location.type === 'grid'
		},
		isDetached: (id) => detachedPanels.value.includes(id),
		regionOf,
		isOpen: (id) => !closed.value.includes(id),
		isShown: (id) => {
			const panel = findPanel(id)
			if (!panel || closed.value.includes(id)) return false
			const host = regionOf(panel)
			if (isRegion(host) && !visible[host].value) return false
			return dockPanelOf(id)?.panel?.api.isVisible ?? false
		},
		setRegionVisible,
		toggleRegion: (region) => setRegionVisible(region, !visible[region].value),
		movePanel,
		moveDocument,
		hostOfDocument,
		splitPanel,
		closePanel,
		revealPanel,
		resetActiveTabs,
		resetLayout,
		canSplit: (id) => mode.splitPanels && (dockPanelOf(id)?.panel?.group.panels.length ?? 0) > 1,
		onGridReady,
		onDockReady,
		onDockDispose,
		onSashDoubleClick,
		setMenuOpener: (opener) => (openMenu = opener),
		openPanelMenu: () => {},
		openRegionMenu: () => {},
		panelsMenuOptions: computed(() => []),
		documentMoveOption: () => null,
	}

	Object.assign(
		context,
		useWorkbenchMenus(context, (event, menuOptions) => openMenu?.(event, menuOptions)),
	)

	return context
}
