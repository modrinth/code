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
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import type { WorkbenchContext } from '../providers/workbench'
import {
	DEFAULT_WORKBENCH_REGION_SIZES,
	WORKBENCH_CENTER_MINIMUM,
	WORKBENCH_HOSTS,
	WORKBENCH_MODES,
	WORKBENCH_REGIONS,
	type WorkbenchDocuments,
	type WorkbenchHost,
	type WorkbenchLink,
	type WorkbenchLinkOrientation,
	type WorkbenchMode,
	type WorkbenchModeName,
	type WorkbenchPanel,
	type WorkbenchPopoutRestore,
	type WorkbenchRegion,
	type WorkbenchRegionSizes,
	type WorkbenchSplitSide,
	type WorkbenchToggleAction,
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
import { registerTabBars } from './tab-bars'
import { useWorkbenchMenus } from './use-menus'
import { useWorkbenchSelection } from './use-tab-selection'
import { useWorkbenchWindows } from './use-windows'

const messages = defineMessages({
	showTabs: { id: 'workbench.lone-tab.show-tabs', defaultMessage: 'Show tabs' },
	hideTabs: { id: 'workbench.lone-tab.hide-tabs', defaultMessage: 'Hide tabs' },
})

export const WORKBENCH_DOCK_COMPONENT = 'workbenchDock'
export const WORKBENCH_PANEL_COMPONENT = 'workbenchPanel'
export const WORKBENCH_TAB_COMPONENT = 'workbenchTab'

const STORAGE_PREFIX = 'workbench-layout:'
const LINK_PREFIX = 'link:'
const LAYOUT_VERSION = 4
const SAVE_DELAY = 200

/** How wide (or tall, for the bottom) a collapsed region's strip of tabs is. */
export const WORKBENCH_COLLAPSED_SIZE = 36

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
	/** Whether the region toggles hide their regions or collapse them. */
	toggleAction?: WorkbenchToggleAction
	/** Whether groups holding a single tab (outside the center) hide their tab bar. */
	hideLoneTabs?: boolean
	/** Panels whose groups' tab bars the user hid. */
	hiddenBars?: string[]
	/** What becomes of popped out panels when the page is opened again. */
	popoutRestore?: WorkbenchPopoutRestore
	sizes: Partial<Record<WorkbenchRegion, number>>
	visible: Partial<Record<WorkbenchRegion, boolean>>
	/** Regions collapsed to a strip of their tabs. */
	collapsed?: Partial<Record<WorkbenchRegion, boolean>>
	/** Each dock's arrangements, by the set of workbench panels it held at the time. */
	docks: Partial<Record<WorkbenchHost, Record<string, SerializedDockview>>>
	/** Panels moved away from the dock they're declared in. */
	placements: Record<string, WorkbenchHost>
	/** Panels the user closed. */
	closed: string[]
	/** Panels the layout has known, so ones closed by default only start closed the first time. */
	seen?: string[]
	/** Panels linked into single tabs. */
	links?: WorkbenchLink[]
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

/**
 * Where a panel the layout hasn't placed yet starts: with the placed panels of its `group`, or
 * else in a group of its own below the dock's others (see `WorkbenchPanel.group`).
 */
function startingPosition(
	api: DockviewApi,
	panel: WorkbenchPanel,
	wanted: WorkbenchPanel[],
): AddPanelPositionOptions | undefined {
	if (!panel.group) return defaultPosition(api)
	const sibling = wanted.find(
		(other) => other.id !== panel.id && other.group === panel.group && api.getPanel(other.id),
	)
	if (sibling) return { referencePanel: sibling.id, direction: 'within' }
	return api.panels.length > 0 ? { direction: 'below' } : defaultPosition(api)
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
	const { formatMessage } = useVIntl()
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
	const hideLoneTabBars = ref(saved?.hideLoneTabs ?? false)
	const hiddenBars = ref<string[]>(saved?.hiddenBars ?? [])
	const popoutRestore = ref<WorkbenchPopoutRestore>(saved?.popoutRestore ?? 'prompt')
	const toggleLocation = ref<WorkbenchToggleLocation>(
		saved?.toggles ?? (mode.cornerToggles ? 'tabs' : 'menu'),
	)
	const toggleAction = ref<WorkbenchToggleAction>(saved?.toggleAction ?? 'hide')
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

	/** The host's panels, before any are linked into single tabs. */
	const ownPanels = computed(() =>
		toValue(options.panels).filter(
			(panel) => panel.region === 'center' || regions.includes(panel.region),
		),
	)
	const links = ref<WorkbenchLink[]>(saved?.links ?? [])
	/** Links of two or more panels the host still offers, holding only those. */
	const activeLinks = computed(() =>
		links.value
			.map((link) => ({
				...link,
				members: link.members.filter((id) => ownPanels.value.some((panel) => panel.id === id)),
			}))
			.filter((link) => link.members.length > 1),
	)

	function linkOf(id: string) {
		return activeLinks.value.find((link) => link.members.includes(id))
	}

	/** The panel holding a panel in view: its linked tab, if it's linked, or else itself. */
	function hostPanelId(id: string) {
		return linkOf(id)?.id ?? id
	}

	/** A linked tab, standing in for its members where the first of them was offered. */
	function linkedTab(link: WorkbenchLink): WorkbenchPanel {
		const members = link.members.flatMap((id) => {
			const panel = ownPanels.value.find((candidate) => candidate.id === id)
			return panel ? [panel] : []
		})
		const [first] = members
		return {
			id: link.id,
			title: members.map((member) => member.title).join(' · '),
			icon: first.icon,
			region: first.region,
			group: first.group,
			scroll: false,
			minimumHeight: Math.max(0, ...members.map((member) => member.minimumHeight ?? 0)),
			members,
		}
	}

	/** The panels as the docks hold them, with linked panels replaced by their linked tabs. */
	const panels = computed(() =>
		ownPanels.value.flatMap((panel) => {
			const link = linkOf(panel.id)
			if (!link) return [panel]
			return link.members[0] === panel.id ? [linkedTab(link)] : []
		}),
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

	/** The dock a panel is in: the one its linked tab is in, if it's linked. */
	function regionOf(panel: WorkbenchPanel): WorkbenchHost {
		const link = linkOf(panel.id)
		const id = link?.id ?? panel.id
		const declared = link ? linkedTab(link).region : panel.region
		const placed = mode.movePanels ? placements.value[id] : undefined
		return placed && (placed === 'center' || regions.includes(placed)) ? placed : declared
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

	/** Regions collapsed to a strip of their tabs, which brings any of them back. */
	const collapsed = Object.fromEntries(
		WORKBENCH_REGIONS.map((region) => [
			region,
			ref(regions.includes(region) && !!saved?.collapsed?.[region]),
		]),
	) as Record<WorkbenchRegion, Ref<boolean>>

	/** How wide (or tall) a region is laid out while shown: its size, or its strip's if collapsed. */
	function shownSize(region: WorkbenchRegion) {
		return collapsed[region].value ? WORKBENCH_COLLAPSED_SIZE : regionSizes[region]
	}

	/** How wide (or tall, for the bottom) each region is shown, 0 while hidden. */
	const regionExtents = Object.fromEntries(
		WORKBENCH_REGIONS.map((region) => [region, ref(visible[region].value ? shownSize(region) : 0)]),
	) as Record<WorkbenchRegion, Ref<number>>

	/** Top-right group of each region's dock, whose tab bar holds the button collapsing the region. */
	const regionCornerGroups = ref<Partial<Record<WorkbenchRegion, string>>>({})

	/** Bumped as docks rearrange, so what's derived from their arrangement (e.g. strips) follows. */
	const arrangement = ref(0)

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
		tabBars?: ReturnType<typeof registerTabBars>
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
			if (visible[region].value && !collapsed[region].value && size > 0) {
				regionSizes[region] = size
			}
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
			toggleAction: toggleAction.value,
			hideLoneTabs: hideLoneTabBars.value,
			hiddenBars: hiddenBars.value,
			popoutRestore: popoutRestore.value,
			sizes: { ...regionSizes },
			visible: Object.fromEntries(regions.map((region) => [region, visible[region].value])),
			collapsed: Object.fromEntries(regions.map((region) => [region, collapsed[region].value])),
			docks: Object.fromEntries(
				WORKBENCH_HOSTS.map((host) => [host, Object.fromEntries(savedDockLayouts[host])]),
			),
			placements: { ...placements.value },
			closed: [...closed.value],
			seen: [...ownPanels.value, ...panels.value].map((panel) => panel.id),
			links: links.value,
		})
	}

	function scheduleSave() {
		clearTimeout(saveTimer)
		if (leavingPage) return
		saveTimer = setTimeout(persist, SAVE_DELAY)
	}

	/** A collapsed region is held at its strip's size; otherwise it can be resized past its minimum. */
	function regionConstraints(region: WorkbenchRegion) {
		const fixed = collapsed[region].value ? WORKBENCH_COLLAPSED_SIZE : undefined
		const minimum = fixed ?? sizes[region].minimum
		const maximum = fixed ?? Number.MAX_SAFE_INTEGER
		return isHorizontal(region)
			? { minimumWidth: minimum, maximumWidth: maximum }
			: { minimumHeight: minimum, maximumHeight: maximum }
	}

	function applyRegionSize(region: WorkbenchRegion) {
		const panel = grid?.getPanel(region)
		if (!panel) return
		const size = shownSize(region)
		panel.api.setConstraints(regionConstraints(region))
		panel.api.setSize(isHorizontal(region) ? { width: size } : { height: size })
	}

	function addRegion(region: WorkbenchRegion) {
		if (!grid || grid.getPanel(region)) return
		grid.addPanel<WorkbenchDockParams>({
			id: region,
			component: WORKBENCH_DOCK_COMPONENT,
			params: { host: region },
			position: { direction: region === 'bottom' ? 'below' : region, referencePanel: 'center' },
			...regionConstraints(region),
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
		for (const group of api.groups) {
			if (group.header.hidden) continue
			group.header.hidden = true
			group.relayout()
		}
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

		const leaving = api.panels.filter((panel) => !ids.has(panel.id) && !isDocumentPanel(panel))
		for (const panel of wanted) {
			const existing = api.getPanel(panel.id)
			if (existing) {
				existing.api.setTitle(panel.title)
				continue
			}
			const moved = pendingPositions.has(panel.id)
			const position = moved
				? usablePosition(api, pendingPositions.get(panel.id))
				: (dock.popouts?.positionFor(panel.id) ?? startingPosition(api, panel, wanted))
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
		/* Dropped once the new ones are in, so those can be placed where the leaving ones were. */
		for (const panel of leaving) {
			withOwnRemovals(() => {
				if (!dock.popouts?.release(panel)) api.removePanel(panel)
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

	function updateRegionCornerGroup(region: WorkbenchRegion) {
		const api = docks.get(region)?.api
		const group = api ? findCornerGroups(api).right : undefined
		if (regionCornerGroups.value[region] !== group) {
			regionCornerGroups.value = { ...regionCornerGroups.value, [region]: group }
		}
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
		else updateRegionCornerGroup(host)
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
				restoreMode: () => popoutRestore.value,
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
		if (!mode.headerless.includes(host)) {
			dock.tabBars = registerTabBars(api, {
				hideLone: () => host !== 'center' && hideLoneTabBars.value,
				hiddenByUser: (group) => group.panels.some((panel) => hiddenBars.value.includes(panel.id)),
				showHidden: (group) => {
					const ids = group.panels.map((panel) => panel.id)
					hiddenBars.value = hiddenBars.value.filter((id) => !ids.includes(id))
				},
				label: (hidden) => formatMessage(hidden ? messages.showTabs : messages.hideTabs),
			})
			dock.disposables.push(dock.tabBars)
		}
		if (host === 'center') dock.disposables.push(api.onDidLayoutChange(updateCornerGroups))
		else dock.disposables.push(api.onDidLayoutChange(() => updateRegionCornerGroup(host)))
		dock.disposables.push(
			api.onDidLayoutChange(() => (arrangement.value += 1)),
			api.onDidActivePanelChange(() => (arrangement.value += 1)),
		)
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
	/** Switching what the toggles do turns regions they'd hidden into collapsed ones, and back. */
	watch(toggleAction, (action) => {
		for (const region of regions) {
			if (action === 'collapse' && !visible[region].value && panelsIn(region).length) {
				setRegionCollapsed(region, true)
			} else if (action === 'hide' && visible[region].value && collapsed[region].value) {
				setRegionVisible(region, false)
				setRegionCollapsed(region, false)
			}
		}
		scheduleSave()
	})
	watch(popoutRestore, scheduleSave)
	watch([hideLoneTabBars, hiddenBars], () => {
		for (const dock of docks.values()) dock.tabBars?.update()
		scheduleSave()
	})
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

	/**
	 * Collapses a region to a strip of its tabs, or brings it back to its size. A collapsed region
	 * is shown, so its strip is too.
	 */
	function setRegionCollapsed(region: WorkbenchRegion, value: boolean) {
		if (!regions.includes(region)) return
		if (value) setRegionVisible(region, true)
		if (collapsed[region].value === value) return
		if (value) measureRegions()
		collapsed[region].value = value
		applyRegionSize(region)
		measureRegions()
		scheduleSave()
	}

	/** Shows a dock, bringing its region back from being hidden or collapsed. */
	function showHost(host: WorkbenchHost) {
		if (!isRegion(host)) return
		setRegionVisible(host, true)
		setRegionCollapsed(host, false)
	}

	/**
	 * Brings a hidden or collapsed region back in full; otherwise hides it, or collapses it to its
	 * tabs, as the settings choose.
	 */
	function toggleRegion(region: WorkbenchRegion) {
		if (!visible[region].value) setRegionVisible(region, true)
		else if (collapsed[region].value) setRegionCollapsed(region, false)
		else if (toggleAction.value === 'collapse') setRegionCollapsed(region, true)
		else setRegionVisible(region, false)
	}

	/**
	 * The tabs a dock holds, in order (its workbench panels and documents alike), e.g. for its
	 * strip while collapsed.
	 */
	function dockTabs(host: WorkbenchHost) {
		void arrangement.value
		const api = docks.get(host)?.api
		if (!api) return []
		return api.groups
			.filter((group) => group.api.location.type === 'grid')
			.flatMap((group) =>
				group.panels.map((panel) => ({
					id: panel.id,
					title: panel.title ?? findPanel(panel.id)?.title ?? panel.id,
					icon: findPanel(panel.id)?.icon,
					active: group.activePanel === panel,
				})),
			)
	}

	/** Brings a dock's tab to the front, showing the dock (e.g. from its collapsed strip). */
	function activateTab(host: WorkbenchHost, id: string) {
		showHost(host)
		const panel = docks.get(host)?.api.getPanel(id)
		if (!panel) return
		panel.group.model.openPanel(panel)
		panel.api.setActive()
	}

	/**
	 * Moves (or reopens) a workbench panel into `host`, at `position` within it or else alongside
	 * its active group's panels, and shows it.
	 */
	function movePanel(requested: string, host: WorkbenchHost, position?: AddPanelPositionOptions) {
		const id = hostPanelId(requested)
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

	/** The panels a dock panel shows: a linked tab's members, or else the panel itself. */
	function membersOf(id: string) {
		return activeLinks.value.find((link) => link.id === id)?.members ?? [id]
	}

	/** Updates every dock a change to the links touched, once they've been set. */
	function applyLinks(hosts: Iterable<WorkbenchHost>) {
		for (const host of new Set(hosts)) updateDock(host)
		scheduleSave()
	}

	/**
	 * Links a panel (or linked tab) with another, making them one tab where the first is, split
	 * between them. Linking with a linked tab brings in all of its panels.
	 */
	function linkPanels(id: string, otherId: string) {
		if (!mode.movePanels) return
		const target = hostPanelId(id)
		const other = hostPanelId(otherId)
		const targetPanel = findPanel(target)
		const otherPanel = findPanel(other)
		if (!targetPanel || !otherPanel || target === other) return
		const hosts = [regionOf(targetPanel), regionOf(otherPanel)]
		const adding = membersOf(other)
		const existing = links.value.find((link) => link.id === target)
		const remaining = links.value
			.filter((link) => link.id !== other && link.id !== target)
			.map((link) => ({
				...link,
				members: link.members.filter((member) => !adding.includes(member)),
			}))
		if (existing) {
			links.value = [
				...remaining,
				{ ...existing, members: [...existing.members, ...adding], sizes: undefined },
			]
		} else {
			const link: WorkbenchLink = {
				id: `${LINK_PREFIX}${target}`,
				members: [target, ...adding],
				orientation: 'vertical',
			}
			placements.value = { ...placements.value, [link.id]: regionOf(targetPanel) }
			closed.value = closed.value.filter((closedId) => closedId !== link.id)
			pendingPositions.set(link.id, { referencePanel: target, direction: 'within' })
			links.value = [...remaining, link]
		}
		applyLinks(hosts)
	}

	/** Takes a panel out of its linked tab, beside it; a link left with a single panel is undone. */
	function unlinkPanel(id: string) {
		const link = linkOf(id)
		if (!link) return
		const tab = findPanel(link.id)
		const host = tab ? regionOf(tab) : 'center'
		const members = link.members.filter((member) => member !== id)
		const released = members.length > 1 ? [id] : link.members
		for (const member of released) {
			placements.value = { ...placements.value, [member]: host }
			pendingPositions.set(member, { referencePanel: link.id, direction: 'within' })
		}
		links.value = links.value.flatMap((candidate) => {
			if (candidate.id !== link.id) return [candidate]
			return members.length > 1 ? [{ ...candidate, members, sizes: undefined }] : []
		})
		applyLinks([host])
	}

	/** Undoes a link, its panels each becoming a tab of their own where it was. */
	function unlinkAll(linkId: string) {
		const link = activeLinks.value.find((candidate) => candidate.id === linkId)
		if (link) unlinkPanel(link.members[0])
		if (activeLinks.value.some((candidate) => candidate.id === linkId)) unlinkAll(linkId)
	}

	function updateLink(linkId: string, change: Partial<WorkbenchLink>) {
		links.value = links.value.map((link) => (link.id === linkId ? { ...link, ...change } : link))
		scheduleSave()
	}

	function setLinkOrientation(linkId: string, orientation: WorkbenchLinkOrientation) {
		updateLink(linkId, { orientation })
	}

	function setLinkSizes(linkId: string, sizes: number[]) {
		updateLink(linkId, { sizes })
	}

	/**
	 * Turns the order of a linked tab's panels around, keeping each one's share. The tab stays in
	 * its dock, though the panel now first may have been offered in another.
	 */
	function reverseLink(linkId: string) {
		const link = links.value.find((candidate) => candidate.id === linkId)
		const tab = findPanel(linkId)
		if (!link || !tab) return
		placements.value = { ...placements.value, [linkId]: regionOf(tab) }
		updateLink(linkId, {
			members: [...link.members].reverse(),
			sizes: link.sizes ? [...link.sizes].reverse() : undefined,
		})
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

	function dockPanelOf(requested: string) {
		const id = hostPanelId(requested)
		const panel = findPanel(id)
		const dock = panel ? docks.get(regionOf(panel)) : undefined
		return dock ? { dock, panel: dock.api.getPanel(id) } : undefined
	}

	/** The dock panels showing the given panels, each once. */
	function dockPanelsOf(ids: string[]) {
		const found = ids.flatMap((id) => {
			const panel = dockPanelOf(id)?.panel
			return panel ? [panel] : []
		})
		return [...new Set(found)]
	}

	/** Whether the panels share a group they don't fill, so they can be split off from it. */
	function canSplit(ids: string[]) {
		const found = dockPanelsOf(ids)
		const group = found[0]?.group
		return (
			mode.splitPanels &&
			!!group &&
			found.every((panel) => panel.group === group) &&
			group.panels.length > found.length
		)
	}

	/** Splits panels off into a group of their own beside the rest of their group, within its dock. */
	function splitPanels(ids: string[], side: WorkbenchSplitSide) {
		if (!canSplit(ids)) return
		const [first, ...rest] = dockPanelsOf(ids)
		first.api.moveTo({ group: first.group, position: side })
		for (const panel of rest) panel.api.moveTo({ group: first.group, position: 'center' })
		first.api.setActive()
	}

	/** Hides the tab bar of the panel's group, until it's pulled down again from its handle. */
	function hideTabBar(id: string) {
		const group = dockPanelOf(id)?.panel?.group
		if (!group) return
		const ids = group.panels.map((panel) => panel.id)
		hiddenBars.value = [...new Set([...hiddenBars.value, ...ids])]
	}

	type GridNode = SerializedDockview['grid']['root']

	/**
	 * The dock's layout, with the outermost split of its groups (in its own window), when it has
	 * one; a split holding a single other split is looked through.
	 */
	function outermostSplit(host: WorkbenchHost) {
		const dock = docks.get(host)
		if (!dock) return undefined
		const layout = dock.api.toJSON()
		let node: GridNode = layout.grid.root
		let horizontal = layout.grid.orientation === 'HORIZONTAL'
		while (Array.isArray(node.data) && node.data.length === 1) {
			node = node.data[0]
			horizontal = !horizontal
		}
		if (!Array.isArray(node.data)) return undefined
		const children = node.data
		/** Groups left hidden by popped out ones, waiting for them to return, keep their place. */
		const shown = children.filter((child) => child.visible !== false)
		if (shown.length < 2) return undefined
		const groups = shown.flatMap((child) => {
			const data = child.data
			const group = Array.isArray(data)
				? undefined
				: dock.api.groups.find((candidate) => candidate.id === data.id)
			return group ? [group] : []
		})
		/*
		 * Groups side by side are moved around one by one, which leaves popped out windows open; a
		 * split of splits is laid out anew, which can only be done without any.
		 */
		const movable = groups.length === shown.length
		if (!movable && dock.api.getPopouts().length) return undefined
		return { dock, layout, children, groups: movable ? groups : undefined, horizontal }
	}

	/** Turns the order of a dock's groups around, e.g. to have its last group on top. */
	function reverseGroups(host: WorkbenchHost) {
		const split = outermostSplit(host)
		if (!split) return
		const { dock, layout, children, groups, horizontal } = split
		withOwnRemovals(() => {
			if (groups) {
				for (let index = 1; index < groups.length; index++) {
					groups[index].api.moveTo({
						group: groups[index - 1],
						position: horizontal ? 'left' : 'top',
					})
				}
			} else {
				children.reverse()
				dock.api.fromJSON(layout, { reuseExistingPanels: true })
			}
		})
		updateCornerGroups()
		scheduleSave()
	}

	function closePanel(requested: string) {
		if (linkOf(requested)) unlinkPanel(requested)
		const id = hostPanelId(requested)
		const panel = findPanel(id)
		if (!mode.closePanels || !panel || closed.value.includes(id)) return
		const api = docks.get(regionOf(panel))?.api
		closed.value = [...closed.value, id]
		if (api) removeOwn(api, id)
		scheduleSave()
	}

	/** Shows a panel: opening it if closed, showing its region, or bringing its popout window forward. */
	function revealPanel(requested: string) {
		const id = hostPanelId(requested)
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

	/**
	 * Restores the defaults, with every popout docked and every document back in the center. Regions
	 * left without panels stay hidden, as they would on a first visit.
	 */
	function resetLayout() {
		if (!isReady.value) return
		for (const dock of docks.values()) {
			if (dock.popouts?.restoring() || dock.popouts?.poppingOut.value.length) return
		}
		for (const dock of docks.values()) dock.popouts?.closeAll()
		pendingPositions.clear()
		pendingDocuments.clear()
		placements.value = {}
		links.value = []
		closed.value = mode.closePanels
			? panels.value.filter((panel) => panel.closedByDefault).map((panel) => panel.id)
			: []
		for (const host of WORKBENCH_HOSTS) savedDockLayouts[host].clear()
		for (const region of regions) {
			const shown = !options.hiddenRegions?.includes(region) && panelsIn(region).length > 0
			regionSizes[region] = sizes[region].default
			collapsed[region].value = false
			visible[region].value = shown
			grid?.getPanel(region)?.api.setVisible(shown)
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
			if (!region || !visible[region].value || collapsed[region].value) return
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
		hideLoneTabBars,
		setHideLoneTabBars: (value) => (hideLoneTabBars.value = value),
		hideTabBar,
		canHideTabBar: (host) => !mode.headerless.includes(host),
		reverseGroups,
		canReverseGroups: (host) => !!outermostSplit(host),
		popoutRestore,
		setPopoutRestore: (value) => (popoutRestore.value = value),
		toggleLocation,
		setToggleLocation: (value) => (toggleLocation.value = value),
		toggleAction,
		setToggleAction: (value) => (toggleAction.value = value),
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
			popoutsEnabled.value
				? dockPanelOf(id)?.dock.popouts?.popoutPanel(hostPanelId(id))
				: undefined,
		dockPanel: (id) => dockPanelOf(id)?.dock.popouts?.dock(hostPanelId(id)),
		closePopouts,
		canPopout: (id) => {
			if (!popoutsEnabled.value) return false
			const found = dockPanelOf(id)
			return !!found?.dock.popouts && found.panel?.api.location.type === 'grid'
		},
		isDetached: (id) => detachedPanels.value.includes(hostPanelId(id)),
		regionOf,
		isOpen: (id) => !closed.value.includes(hostPanelId(id)),
		isShown: (requested) => {
			const id = hostPanelId(requested)
			const panel = findPanel(id)
			if (!panel || closed.value.includes(id)) return false
			const host = regionOf(panel)
			if (isRegion(host) && (!visible[host].value || collapsed[host].value)) return false
			return dockPanelOf(id)?.panel?.api.isVisible ?? false
		},
		setRegionVisible,
		toggleRegion,
		collapsed,
		setRegionCollapsed,
		regionCornerGroups,
		dockTabs,
		activateTab,
		movePanel,
		moveDocument,
		hostOfDocument,
		splitPanels,
		closePanel,
		revealPanel,
		links: activeLinks,
		linkPanels,
		unlinkPanel,
		unlinkAll,
		setLinkOrientation,
		reverseLink,
		setLinkSizes,
		resetActiveTabs,
		resetLayout,
		canSplit,
		popoutPanels: (ids) => {
			if (!popoutsEnabled.value || !ids.length) return undefined
			return dockPanelOf(ids[0])?.dock.popouts?.popoutPanels(ids.map(hostPanelId))
		},
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
