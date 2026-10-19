import { injectNotificationManager, useVIntl } from '@modrinth/ui'
import { useIntervalFn } from '@vueuse/core'
import type {
	DockviewApi,
	DockviewGroupPanel,
	DockviewIDisposable,
	DockviewPopoutGroupOptions,
	DockviewReadyEvent,
	DockviewWillDropEvent,
	DockviewWillShowOverlayLocationEvent,
	SerializedDockview,
	SplitviewApi,
	SplitviewReadyEvent,
} from 'dockview-vue'
import { getGridLocation, LayoutPriority, Orientation } from 'dockview-vue'
import { onBeforeUnmount, ref, watchEffect } from 'vue'

import { projectReviewMessages } from '../messages'
import { injectReviewContext } from '../review-panel/context'
import { injectReviewShortcuts } from '../shortcuts'
import { registerDragPopouts } from './drag-popout'
import {
	dockPopoutLayout,
	readWorkspaceLayout,
	reviewPopoutUrl,
	saveWorkspaceLayout,
	workspacePanelSizes,
	workspaceTabLayoutKey,
} from './layout-storage'
import {
	configureReviewPopout,
	openReviewPopout,
	removeReviewPopoutPanel,
	restoreReviewPopout,
	type ReviewPopoutLayout,
	type ReviewPopoutTarget,
} from './popout-window'
import { useTabSelection } from './tab-selection'
import { type ProjectReviewSlot, type ProjectReviewTab, projectReviewTabs } from './types'
import { registerWrappedTabs } from './wrapped-tabs'

export function useProjectReviewLayout(
	getTitle: (tab: ProjectReviewTab) => string,
	getTabs: () => readonly ProjectReviewTab[],
	getReady: () => boolean,
) {
	const shortcuts = injectReviewShortcuts()
	const review = injectReviewContext()
	const tabSelection = useTabSelection()
	const detachedTabs = ref<ProjectReviewTab[]>([])
	const hasPopouts = ref(false)
	const { addNotification, removeNotification } = injectNotificationManager()
	const { formatMessage } = useVIntl()
	const poppingOut = ref<string[]>([])
	const popoutSources = new Map<string, string>()
	const unavailablePopoutTabs = new Map<
		ProjectReviewTab,
		{ group: DockviewGroupPanel; index: number }
	>()
	const popoutCleanups = new Map<Window, () => void>()
	const savedLayout = readWorkspaceLayout()
	const leftVisible = ref(savedLayout?.leftVisible ?? true)
	const rightVisible = ref(savedLayout?.rightVisible ?? true)
	let leftWidth = savedLayout?.leftWidth ?? workspacePanelSizes.left.default
	let rightWidth = savedLayout?.rightWidth ?? workspacePanelSizes.right.default
	const visibleRightWidth = ref(rightVisible.value ? rightWidth : 0)
	const topLeftGroupId = ref<string>()
	const topRightGroupId = ref<string>()
	let columns: SplitviewApi | undefined
	let tabsReady = false
	let leavingPage = false
	let blockedPopoutLayout: SerializedDockview | undefined
	let blockedPopoutBaseline: string | undefined
	const pendingPopouts = new Map<
		ReviewPopoutLayout,
		{ notificationId?: string | number; opening: boolean }
	>()
	const documentLoaded = ref(document.readyState === 'complete')
	let saveTimer: ReturnType<typeof setTimeout> | undefined
	let tabs: DockviewApi | undefined
	let currentTabLayoutKey: string | undefined
	const tabLayouts = new Map(Object.entries(savedLayout?.tabLayouts ?? {}))
	if (savedLayout) {
		tabLayouts.set(
			workspaceTabLayoutKey(Object.keys(savedLayout.tabs.panels) as ProjectReviewTab[]),
			savedLayout.tabs,
		)
	}
	const subscriptions: DockviewIDisposable[] = []
	const { pause: stopCheckingWindows, resume: checkWindows } = useIntervalFn(
		() => {
			for (const tab of detachedTabs.value) {
				const panel = tabs?.getPanel(tab)
				if (panel?.api.location.type === 'popout' && panel.api.getWindow().closed) dockTab(tab)
			}
		},
		500,
		{ immediate: false },
	)

	function updateDetachedTabs() {
		hasPopouts.value = !!tabs?.getPopouts().length
		detachedTabs.value = getTabs().filter(
			(tab) => tabs?.getPanel(tab)?.api.location.type === 'popout',
		)
		if (detachedTabs.value.length) checkWindows()
		else stopCheckingWindows()
	}

	function currentTabLayout(api: DockviewApi) {
		const layout = api.toJSON()
		if (
			blockedPopoutLayout &&
			!restoringPopouts() &&
			blockedPopoutBaseline !== JSON.stringify(layout)
		) {
			blockedPopoutLayout = undefined
			blockedPopoutBaseline = undefined
		}
		return blockedPopoutLayout ?? layout
	}

	function saveLayout() {
		clearTimeout(saveTimer)
		if (!columns || !tabs || !tabsReady || leavingPage || !getReady() || !documentLoaded.value)
			return
		const currentLeftWidth = columns.getPanel('left')?.api.width ?? 0
		const currentRightWidth = columns.getPanel('right')?.api.width ?? 0
		if (leftVisible.value && currentLeftWidth > 0) leftWidth = currentLeftWidth
		if (rightVisible.value && currentRightWidth > 0) rightWidth = currentRightWidth
		const currentTabs = currentTabLayout(tabs)
		if (currentTabLayoutKey) tabLayouts.set(currentTabLayoutKey, currentTabs)
		saveWorkspaceLayout({
			leftWidth,
			rightWidth,
			leftVisible: leftVisible.value,
			rightVisible: rightVisible.value,
			tabs: currentTabs,
			tabLayouts: Object.fromEntries(tabLayouts),
		})
	}

	function scheduleSave() {
		clearTimeout(saveTimer)
		if (leavingPage) return
		saveTimer = setTimeout(saveLayout, 200)
	}

	function dismissRestoreNotifications() {
		for (const pending of pendingPopouts.values()) {
			if (pending.notificationId !== undefined) removeNotification(pending.notificationId)
		}
		pendingPopouts.clear()
	}

	function restoringPopouts() {
		return [...pendingPopouts.values()].some((pending) => pending.opening)
	}

	function popoutPanelIds(popout: ReviewPopoutLayout) {
		const ids: string[] = []
		function collect(node: SerializedDockview['grid']['root']) {
			if (Array.isArray(node.data)) node.data.forEach(collect)
			else ids.push(...node.data.views)
		}
		if (popout.grid) collect(popout.grid.root)
		else if (popout.data) ids.push(...popout.data.views)
		return ids
	}

	function notifyPopoutRestoration(popout: ReviewPopoutLayout) {
		const pending = pendingPopouts.get(popout)
		if (!pending || pending.notificationId !== undefined) return
		const panels = popoutPanelIds(popout)
			.map((id) => getTitle(id as ProjectReviewTab))
			.join(', ')
		pending.notificationId = addNotification({
			id: crypto.randomUUID(),
			type: 'neutral',
			containerClass: '!py-1.5 !pr-0.5 !pb-2 !rounded-xl',
			title: formatMessage(projectReviewMessages.restorePopouts),
			text: formatMessage(projectReviewMessages.restorePopoutsDescription, {
				panels,
			}),
			autoCloseMs: null,
			noIcon: true,
			buttons: [
				{
					label: formatMessage(projectReviewMessages.reopenPopouts),
					keepOpen: true,
					color: 'green',
					action: () => reopenPopout(popout),
				},
				{
					label: formatMessage(projectReviewMessages.keepPopoutClosed),
					keepOpen: true,
					action: () => keepPopoutClosed(popout),
				},
			],
		}).id
	}

	function keepPopoutClosed(popout: ReviewPopoutLayout) {
		const pending = pendingPopouts.get(popout)
		if (!pending || pending.opening) return
		if (pending.notificationId !== undefined) removeNotification(pending.notificationId)
		pendingPopouts.delete(popout)
		if (blockedPopoutLayout) {
			const remainingPopouts = blockedPopoutLayout.popoutGroups?.filter((group) => group !== popout)
			blockedPopoutLayout = dockPopoutLayout({
				...blockedPopoutLayout,
				popoutGroups: [popout],
			})
			blockedPopoutLayout.popoutGroups = remainingPopouts
		}
		if (!pendingPopouts.size) {
			blockedPopoutLayout = undefined
			blockedPopoutBaseline = undefined
		}
		scheduleSave()
	}

	async function reopenPopout(popout: ReviewPopoutLayout) {
		const pending = pendingPopouts.get(popout)
		if (!tabs || !pending || pending.opening || !tabsReady || leavingPage || !getReady()) return
		currentTabLayout(tabs)
		pending.opening = true
		const opened = await restoreReviewPopout(tabs, popout)
		if (pendingPopouts.get(popout) !== pending) return
		pending.opening = false
		if (opened) {
			if (pending.notificationId !== undefined) removeNotification(pending.notificationId)
			pendingPopouts.delete(popout)
			if (!pendingPopouts.size) blockedPopoutLayout = undefined
		} else notifyPopoutRestoration(popout)
		syncTabs()
		if (blockedPopoutLayout && !restoringPopouts()) {
			blockedPopoutBaseline = JSON.stringify(tabs.toJSON())
		}
		scheduleSave()
	}

	function restoreTabLayout(layout: SerializedDockview) {
		if (!tabs) return
		dismissRestoreNotifications()
		blockedPopoutLayout = undefined
		blockedPopoutBaseline = undefined
		if (layout.popoutGroups?.length) {
			tabs.fromJSON(dockPopoutLayout(layout), { reuseExistingPanels: true })
			blockedPopoutLayout = layout
			blockedPopoutBaseline = JSON.stringify(tabs.toJSON())
			for (const popout of layout.popoutGroups) pendingPopouts.set(popout, { opening: false })
			for (const popout of layout.popoutGroups) void reopenPopout(popout)
			return
		}
		tabs.fromJSON(layout, { reuseExistingPanels: true })
	}

	function updateRightPanelWidth() {
		visibleRightWidth.value = rightVisible.value
			? (columns?.getPanel('right')?.api.width ?? rightWidth)
			: 0
	}

	function onColumnsReady({ api }: SplitviewReadyEvent) {
		columns = api
		api.addPanel({
			id: 'left',
			component: 'ProjectReviewPanel',
			params: { slot: 'left' },
			minimumSize: workspacePanelSizes.left.minimum,
			size: leftWidth,
		})
		api.addPanel({
			id: 'center',
			component: 'ProjectReviewCenter',
			minimumSize: workspacePanelSizes.center.minimum,
			priority: LayoutPriority.High,
		})
		api.addPanel({
			id: 'right',
			component: 'ProjectReviewPanel',
			params: { slot: 'right' },
			minimumSize: workspacePanelSizes.right.minimum,
			size: rightWidth,
		})
		api.getPanel('left')?.api.setSize({ size: leftWidth })
		api.getPanel('right')?.api.setSize({ size: rightWidth })
		api.getPanel('left')?.api.setVisible(leftVisible.value)
		api.getPanel('right')?.api.setVisible(rightVisible.value)
		updateRightPanelWidth()
		subscriptions.push(
			api.onDidLayoutChange(updateRightPanelWidth),
			api.onDidLayoutChange(scheduleSave),
		)
	}

	function constrainDrop(event: DockviewWillDropEvent | DockviewWillShowOverlayLocationEvent) {
		if (!event.getData()) {
			event.preventDefault()
		}
	}

	function splitDroppedGroup(event: DockviewWillDropEvent) {
		const data = event.getData()
		const target = event.group
		if (
			event.defaultPrevented ||
			event.kind !== 'content' ||
			!target ||
			!data ||
			data.viewId !== event.api.id ||
			event.position === 'center'
		)
			return

		const horizontal = event.position === 'left' || event.position === 'right'
		let movedPanel: ReturnType<DockviewApi['getPanel']>
		const subscription = event.api.onDidMovePanel(({ panel, from }) => {
			if (from.id === data.groupId && (!data.panelId || panel.id === data.panelId)) {
				movedPanel = panel
			}
		})
		queueMicrotask(() => {
			subscription.dispose()
			const moved = movedPanel?.group
			if (!tabsReady || !moved || moved === target || !event.api.groups.includes(target)) return
			const dimension = horizontal ? 'width' : 'height'
			const minimum = horizontal ? 'minimumWidth' : 'minimumHeight'
			const maximum = horizontal ? 'maximumWidth' : 'maximumHeight'
			const total = target.api[dimension] + moved.api[dimension]
			const lower = Math.max(moved[minimum], total - target[maximum])
			const upper = Math.min(moved[maximum], total - target[minimum])
			if (lower > upper) return
			const size = Math.max(lower, Math.min(Math.round(total / 2), upper))
			target.api.setSize({ [dimension]: total - size })
			moved.api.setSize({ [dimension]: size })
		})
	}

	function updateCornerGroups() {
		const groups =
			tabs?.groups
				.filter(
					(group) =>
						group.api.location.type === 'grid' && group.api.isVisible && group.panels.length > 0,
				)
				.map((group) => ({
					id: group.id,
					bounds: group.element.getBoundingClientRect(),
				})) ?? []
		const top = Math.min(...groups.map(({ bounds }) => bounds.top))
		const topGroups = groups.filter(({ bounds }) => Math.abs(bounds.top - top) < 1)
		topGroups.sort((a, b) => a.bounds.left - b.bounds.left)
		topLeftGroupId.value = topGroups[0]?.id
		topRightGroupId.value = topGroups[topGroups.length - 1]?.id
	}

	function onTabsReady({ api }: DockviewReadyEvent) {
		tabs = api
		subscriptions.push(
			registerWrappedTabs(api),
			tabSelection.register(api),
			registerDragPopouts(api, (target, position, sourceWindow) => {
				const release = tabSelection.keepDragForPopout()
				void popoutPanels(target, position, sourceWindow).finally(release)
			}),
			api.onDidAddPopoutGroup(({ window: child }) => {
				updateDetachedTabs()
				review.close()
				const disposeWindow = shortcuts.registerWindow(child)
				const disposeTheme = configureReviewPopout(child)
				popoutCleanups.set(child, () => {
					disposeWindow()
					disposeTheme()
				})
			}),
			api.onDidRemovePopoutGroup(() => {
				updateDetachedTabs()
				review.close()
				const windows = new Set(api.getPopouts().map((popout) => popout.window))
				for (const [child, cleanup] of popoutCleanups) {
					if (windows.has(child)) continue
					cleanup()
					popoutCleanups.delete(child)
				}
			}),
			api.onWillShowOverlay(constrainDrop),
			api.onWillDrop(constrainDrop),
			api.onWillDrop(splitDroppedGroup),
			api.onDidLayoutChange(updateCornerGroups),
			api.onDidLayoutChange(updateDetachedTabs),
			api.onDidLayoutChange(scheduleSave),
			api.onDidActivePanelChange(scheduleSave),
		)
		tabsReady = true
		syncTabs()
		updateCornerGroups()
	}

	function openTab(tab: ProjectReviewTab) {
		if (!getTabs().includes(tab)) return
		const panel = tabs?.getPanel(tab)
		if (!panel) return
		if (panel.api.location.type === 'popout') {
			if (!panel.api.isVisible) {
				panel.group.model.openPanel(panel, { skipSetGroupActive: true })
			}
			panel.api.getWindow().focus()
			return
		}
		panel.api.setActive()
	}

	function popoutTab(
		tab: ProjectReviewTab,
		position?: DockviewPopoutGroupOptions['position'],
		sourceWindow?: Window,
	) {
		const panel = tabs?.getPanel(tab)
		if (panel) return popoutPanels(panel, position, sourceWindow)
	}

	async function popoutPanels(
		target: ReviewPopoutTarget,
		position?: DockviewPopoutGroupOptions['position'],
		sourceWindow?: Window,
	) {
		const panels =
			'panels' in target ? [...target.panels] : (tabSelection.getDragPanels(target.id) ?? [target])
		const panelIds = panels.map((panel) => panel.id)
		if (!tabs || !panels.length || panelIds.some((id) => poppingOut.value.includes(id))) return
		if (target.api.location.type === 'popout' && !position) return
		for (const panel of panels) {
			if (panel.api.location.type === 'grid') popoutSources.set(panel.id, panel.group.id)
		}
		poppingOut.value = [...poppingOut.value, ...panelIds]
		try {
			const opened = await openReviewPopout(
				tabs,
				target,
				{
					popoutUrl: reviewPopoutUrl,
					position,
				},
				sourceWindow,
			)
			if (!opened) notifyPopoutFailure()
		} catch {
			notifyPopoutFailure()
		} finally {
			poppingOut.value = poppingOut.value.filter((id) => !panelIds.includes(id))
		}
	}

	function notifyPopoutFailure() {
		addNotification({
			type: 'error',
			title: formatMessage(projectReviewMessages.popoutFailed),
		})
	}

	function dockTab(tab: ProjectReviewTab) {
		const panel = tabs?.getPanel(tab)
		if (!tabs || !panel || panel.api.location.type !== 'popout') return
		const source = tabs.groups.find((group) => group.id === popoutSources.get(tab))
		const group =
			source?.api.location.type === 'grid' && source.panels.length
				? source
				: (tabs.groups.find((group) => group.api.location.type === 'grid' && group.panels.length) ??
					tabs.addGroup())
		panel.api.moveTo({ group, position: 'center' })
		panel.api.setActive()
		popoutSources.delete(tab)
		scheduleSave()
	}

	function closeExternalPanels() {
		blockedPopoutLayout = undefined
		dismissRestoreNotifications()
		for (const tab of [...detachedTabs.value]) dockTab(tab)
		unavailablePopoutTabs.clear()
		for (const group of tabs?.groups ?? []) {
			if (group.api.location.type === 'popout' && !group.panels.length) tabs?.removeGroup(group)
		}
	}

	function resetPanelLayout() {
		if (!tabs || !tabsReady || restoringPopouts() || poppingOut.value.length) return
		closeExternalPanels()
		const layout = tabs.toJSON()
		const views = projectReviewTabs.filter((tab) => getTabs().includes(tab))
		const groupId = tabs.getPanel('description')?.group.id ?? tabs.groups[0]?.id ?? 'review-tabs'
		tabLayouts.clear()
		popoutSources.clear()
		tabs.fromJSON(
			{
				panels: layout.panels,
				activeGroup: groupId,
				grid: {
					...layout.grid,
					orientation: Orientation.HORIZONTAL,
					root: {
						type: 'branch',
						data: [{ type: 'leaf', data: { id: groupId, views, activeView: views[0] } }],
					},
				},
			},
			{ reuseExistingPanels: true },
		)
		leftWidth = workspacePanelSizes.left.default
		rightWidth = workspacePanelSizes.right.default
		leftVisible.value = true
		rightVisible.value = true
		for (const side of ['left', 'right'] as const) {
			const panel = columns?.getPanel(side)
			panel?.api.setVisible(true)
			panel?.api.setSize({ size: workspacePanelSizes[side].default })
		}
		updateRightPanelWidth()
		scheduleSave()
	}

	function revealSlot(slot: ProjectReviewSlot) {
		if (slot === 'left' || slot === 'right') {
			const visible = slot === 'left' ? leftVisible : rightVisible
			if (!visible.value) toggleSidebar(slot)
		} else {
			openTab(slot)
		}
	}

	function resetActiveTabs() {
		for (const group of tabs?.groups ?? []) {
			const firstPanel = group.panels[0]
			if (firstPanel) {
				group.model.openPanel(firstPanel, {
					skipSetGroupActive: true,
				})
			}
		}
	}

	function toggleSidebar(side: 'left' | 'right') {
		const visible = side === 'left' ? leftVisible : rightVisible
		const panel = columns?.getPanel(side)
		if (!panel || !columns) return
		if (visible.value) {
			if (side === 'left') leftWidth = panel.api.width
			else rightWidth = panel.api.width
		}
		visible.value = !visible.value
		panel.api.setVisible(visible.value)
		updateRightPanelWidth()
		scheduleSave()
	}

	function onDividerDoubleClick(event: MouseEvent) {
		if (!(event.target instanceof HTMLElement)) return
		const sash = event.target.closest('.dv-sash:not(.dv-disabled)')
		const container = sash?.parentElement
		const split = container?.parentElement
		if (!sash || !container || !split) return
		const index = Array.from(container.children).indexOf(sash)
		const columnSplit = split
			.closest('.project-review-columns')
			?.querySelector('.dv-split-view-container')
		if (split === columnSplit && columns) {
			const side = index === 0 ? 'left' : 'right'
			const panel = columns.getPanel(side)
			if (!panel?.api.isVisible) return
			const otherSide = columns.getPanel(side === 'left' ? 'right' : 'left')
			const otherWidth = otherSide?.api.isVisible ? otherSide.api.width : 0
			panel.api.setSize({
				size: Math.max(
					workspacePanelSizes[side].minimum,
					Math.min(
						workspacePanelSizes[side].default,
						columns.width - otherWidth - workspacePanelSizes.center.minimum,
					),
				),
			})
		} else if (tabs && split.closest('.project-review-tabs')) {
			const branch = split.parentElement
			if (!branch?.classList.contains('dv-branch-node')) return
			const layout = tabs.toJSON()
			let node = layout.grid.root
			for (const childIndex of getGridLocation(branch)) {
				if (!Array.isArray(node.data) || !node.data[childIndex]) return
				node = node.data[childIndex]
			}
			if (!Array.isArray(node.data)) return
			const before = node.data[index]
			const after = node.data[index + 1]
			if (!before?.size || !after?.size) return
			const total = before.size + after.size
			if (tabs.getPopouts().length) {
				const horizontal = split.classList.contains('dv-horizontal')
				const sizes: { id: string; size: number }[] = []
				function collectSizes(node: typeof layout.grid.root, scale: number) {
					if (Array.isArray(node.data)) {
						for (const child of node.data) collectSizes(child, scale)
					} else {
						const id = node.data.id
						const group = tabs?.groups.find((group) => group.id === id)
						if (group)
							sizes.push({
								id: group.id,
								size: (horizontal ? group.api.width : group.api.height) * scale,
							})
					}
				}
				collectSizes(before, total / 2 / before.size)
				collectSizes(after, total / 2 / after.size)
				for (const { id, size } of sizes) {
					tabs.groups
						.find((group) => group.id === id)
						?.api.setSize(horizontal ? { width: size } : { height: size })
				}
			} else {
				before.size = Math.round(total / 2)
				after.size = total - before.size
				tabs.fromJSON(layout, { reuseExistingPanels: true })
			}
			updateCornerGroups()
		} else {
			return
		}
		event.preventDefault()
		event.stopPropagation()
		scheduleSave()
	}

	function syncTabs() {
		const ready = getReady() && documentLoaded.value
		const visibleTabs = getTabs()
		const nextTabLayoutKey = workspaceTabLayoutKey(visibleTabs)
		const titles = new Map(visibleTabs.map((tab) => [tab, getTitle(tab)]))
		if (!tabs || !tabsReady || restoringPopouts() || !ready) return
		if (currentTabLayoutKey !== nextTabLayoutKey) {
			if (currentTabLayoutKey) tabLayouts.set(currentTabLayoutKey, currentTabLayout(tabs))
			blockedPopoutLayout = undefined
			dismissRestoreNotifications()
			currentTabLayoutKey = nextTabLayoutKey
			const savedTabLayout = tabLayouts.get(nextTabLayoutKey)
			if (savedTabLayout && !tabs.getPopouts().length) {
				try {
					restoreTabLayout(savedTabLayout)
					if (restoringPopouts()) return
				} catch {
					tabs.clear()
					tabLayouts.delete(nextTabLayoutKey)
				}
			}
		}
		for (const tab of projectReviewTabs) {
			const panel = tabs.getPanel(tab)
			if (!visibleTabs.includes(tab)) {
				if (panel?.api.location.type === 'popout') {
					const group = panel.group
					unavailablePopoutTabs.set(tab, {
						group,
						index: group.panels.indexOf(panel),
					})
					removeReviewPopoutPanel(panel)
				} else if (panel) {
					tabs.removePanel(panel)
					popoutSources.delete(tab)
				}
				continue
			}
			if (!panel) {
				const previous = unavailablePopoutTabs.get(tab)
				const popoutGroup =
					previous &&
					tabs.groups.includes(previous.group) &&
					previous.group.api.location.type === 'popout' &&
					!previous.group.api.getWindow().closed
						? previous.group
						: undefined
				const referencePanel =
					tabs.getPanel('description')?.api.location.type === 'grid'
						? tabs.getPanel('description')
						: tabs.panels.find((panel) => panel.api.location.type === 'grid')
				tabs.addPanel({
					id: tab,
					component: 'ProjectReviewPanel',
					title: titles.get(tab),
					params: { tab, slot: tab },
					renderer: 'always',
					minimumHeight: 120,
					inactive: true,
					position: popoutGroup
						? { referenceGroup: popoutGroup.id, index: previous?.index }
						: referencePanel
							? { referencePanel: referencePanel.id }
							: undefined,
				})
				unavailablePopoutTabs.delete(tab)
			} else {
				panel.api.setTitle(titles.get(tab) ?? tab)
			}
		}
		if (!tabs.activePanel) tabs.getPanel('description')?.api.setActive()
	}

	watchEffect(syncTabs)

	/** Capture the popouts before Dockview's unload handlers return their panels to the grid. */
	function onBeforeUnload() {
		saveLayout()
		leavingPage = true
		clearTimeout(saveTimer)
	}

	function onPageShow() {
		leavingPage = false
	}

	function resumeAfterCancelledUnload(event: Event) {
		if (event.currentTarget === window) leavingPage = false
	}

	function preventPopoutRefresh(event: KeyboardEvent) {
		if (event.currentTarget === window) return
		if (
			event.key !== 'F5' &&
			!(
				(event.ctrlKey || event.metaKey) &&
				!event.altKey &&
				(event.code === 'KeyR' || event.key.toLowerCase() === 'r')
			)
		)
			return
		event.preventDefault()
		event.stopImmediatePropagation()
	}

	function onDocumentLoad() {
		documentLoaded.value = true
	}

	subscriptions.push(
		{
			dispose: shortcuts.listen('keydown', preventPopoutRefresh, {
				capture: true,
			}),
		},
		{
			dispose: shortcuts.listen('pointerdown', resumeAfterCancelledUnload, {
				capture: true,
			}),
		},
		{
			dispose: shortcuts.listen('keydown', resumeAfterCancelledUnload, {
				capture: true,
			}),
		},
	)
	window.addEventListener('beforeunload', onBeforeUnload, { capture: true })
	window.addEventListener('load', onDocumentLoad)
	window.addEventListener('pageshow', onPageShow)
	window.addEventListener('pagehide', onBeforeUnload)
	onBeforeUnmount(() => {
		saveLayout()
		tabsReady = false
		clearTimeout(saveTimer)
		dismissRestoreNotifications()
		window.removeEventListener('load', onDocumentLoad)
		window.removeEventListener('beforeunload', onBeforeUnload, true)
		window.removeEventListener('pageshow', onPageShow)
		window.removeEventListener('pagehide', onBeforeUnload)
		subscriptions.forEach((subscription) => subscription.dispose())
		popoutCleanups.forEach((cleanup) => cleanup())
	})

	return {
		resetPanelLayout,
		selectedTabs: tabSelection.selectedTabs,
		closeExternalPanels,
		hasPopouts,
		detachedTabs,
		popoutTab,
		dockTab,
		poppingOut,
		leftVisible,
		openTab,
		revealSlot,
		resetActiveTabs,
		rightVisible,
		visibleRightWidth,
		topLeftGroupId,
		topRightGroupId,
		onColumnsReady,
		onTabsReady,
		onDividerDoubleClick,
		toggleSidebar,
	}
}
