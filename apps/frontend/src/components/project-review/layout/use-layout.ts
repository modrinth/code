import { injectNotificationManager, useVIntl } from '@modrinth/ui'
import { useIntervalFn } from '@vueuse/core'
import type {
	DockviewApi,
	DockviewIDisposable,
	DockviewPopoutGroupOptions,
	DockviewReadyEvent,
	DockviewWillDropEvent,
	DockviewWillShowOverlayLocationEvent,
	SplitviewApi,
	SplitviewReadyEvent,
} from 'dockview-vue'
import { getGridLocation, LayoutPriority } from 'dockview-vue'
import { onBeforeUnmount, ref, watchEffect } from 'vue'

import { projectReviewMessages } from '../messages'
import { injectReviewContext } from '../review-panel/context'
import { injectReviewShortcuts } from '../shortcuts'
import { registerDragPopouts } from './drag-popout'
import {
	dockPopoutLayout,
	readWorkspaceLayout,
	saveWorkspaceLayout,
	workspacePanelSizes,
	workspaceTabLayoutKey,
} from './layout-storage'
import { configureReviewPopout, openReviewPopout, type ReviewPopoutTarget } from './popout-window'
import { type ProjectReviewSlot, type ProjectReviewTab, projectReviewTabs } from './types'

export function useProjectReviewLayout(
	getTitle: (tab: ProjectReviewTab) => string,
	getTabs: () => readonly ProjectReviewTab[],
) {
	const shortcuts = injectReviewShortcuts()
	const review = injectReviewContext()
	const detachedTabs = ref<ProjectReviewTab[]>([])
	const { addNotification } = injectNotificationManager()
	const { formatMessage } = useVIntl()
	const poppingOut = ref<string[]>([])
	const popoutSources = new Map<string, string>()
	const popoutCleanups = new Map<Window, () => void>()
	const savedLayout = readWorkspaceLayout()
	const leftVisible = ref(savedLayout?.leftVisible ?? true)
	const rightVisible = ref(savedLayout?.rightVisible ?? true)
	const bottomVisible = ref(savedLayout?.bottomVisible ?? true)
	let leftWidth = savedLayout?.leftWidth ?? workspacePanelSizes.left.default
	let rightWidth = savedLayout?.rightWidth ?? workspacePanelSizes.right.default
	const visibleRightWidth = ref(rightVisible.value ? rightWidth : 0)
	let bottomHeight = savedLayout?.bottomHeight ?? workspacePanelSizes.bottom.default
	const topLeftGroupId = ref<string>()
	const topRightGroupId = ref<string>()
	let columns: SplitviewApi | undefined
	let rows: SplitviewApi | undefined
	let tabsReady = false
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
		detachedTabs.value = getTabs().filter(
			(tab) => tabs?.getPanel(tab)?.api.location.type === 'popout',
		)
		if (detachedTabs.value.length) checkWindows()
		else stopCheckingWindows()
	}

	function saveLayout() {
		clearTimeout(saveTimer)
		if (!columns || !rows || !tabs || !tabsReady) return
		const currentLeftWidth = columns.getPanel('left')?.api.width ?? 0
		const currentRightWidth = columns.getPanel('right')?.api.width ?? 0
		const currentBottomHeight = rows.getPanel('bottom')?.api.height ?? 0
		if (leftVisible.value && currentLeftWidth > 0) leftWidth = currentLeftWidth
		if (rightVisible.value && currentRightWidth > 0) rightWidth = currentRightWidth
		if (bottomVisible.value && currentBottomHeight > 0) bottomHeight = currentBottomHeight
		const currentTabs = tabs.toJSON()
		if (currentTabLayoutKey) tabLayouts.set(currentTabLayoutKey, currentTabs)
		saveWorkspaceLayout({
			leftWidth,
			rightWidth,
			bottomHeight,
			bottomVisible: bottomVisible.value,
			leftVisible: leftVisible.value,
			rightVisible: rightVisible.value,
			tabs: dockPopoutLayout(currentTabs),
			tabLayouts: Object.fromEntries(
				[...tabLayouts].map(([key, layout]) => [key, dockPopoutLayout(layout)]),
			),
		})
	}

	function scheduleSave() {
		clearTimeout(saveTimer)
		saveTimer = setTimeout(saveLayout, 200)
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

	function onRowsReady({ api }: SplitviewReadyEvent) {
		rows = api
		api.addPanel({
			id: 'tabs',
			component: 'ProjectReviewTabs',
			minimumSize: workspacePanelSizes.tabs.minimum,
			priority: LayoutPriority.High,
		})
		api.addPanel({
			id: 'bottom',
			component: 'ProjectReviewPanel',
			params: { slot: 'bottom' },
			minimumSize: workspacePanelSizes.bottom.minimum,
			size: bottomHeight,
		})
		api.getPanel('bottom')?.api.setVisible(bottomVisible.value)
		subscriptions.push(api.onDidLayoutChange(scheduleSave))
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
			registerDragPopouts(api, (target, position, sourceWindow) => {
				void popoutPanels(target, position, sourceWindow)
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
		currentTabLayoutKey = workspaceTabLayoutKey(getTabs())
		let restored = false
		const savedTabLayout = tabLayouts.get(currentTabLayoutKey)
		if (savedTabLayout) {
			try {
				api.fromJSON(dockPopoutLayout(savedTabLayout))
				restored = true
			} catch {
				api.clear()
				tabLayouts.delete(currentTabLayoutKey)
			}
		}
		tabsReady = true
		syncTabs()
		if (!restored) api.getPanel('description')?.api.setActive()
		updateCornerGroups()
	}

	function openTab(tab: ProjectReviewTab) {
		if (!getTabs().includes(tab)) return
		const panel = tabs?.getPanel(tab)
		panel?.api.setActive()
		if (panel?.api.location.type === 'popout') panel.api.getWindow().focus()
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
		const panels = 'panels' in target ? [...target.panels] : [target]
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
					popoutUrl: '/project-review-popout.html',
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
			text: formatMessage(projectReviewMessages.popoutFailed),
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
		for (const tab of [...detachedTabs.value]) dockTab(tab)
	}

	function revealSlot(slot: ProjectReviewSlot) {
		if (slot === 'left' || slot === 'right') {
			const visible = slot === 'left' ? leftVisible : rightVisible
			if (!visible.value) toggleSidebar(slot)
		} else if (slot === 'bottom') {
			if (!bottomVisible.value) toggleToolsPanel()
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

	function toggleToolsPanel() {
		const panel = rows?.getPanel('bottom')
		if (!panel || !rows) return
		if (bottomVisible.value) bottomHeight = panel.api.height
		bottomVisible.value = !bottomVisible.value
		panel.api.setVisible(bottomVisible.value)
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
		const rowSplit = split
			.closest('.project-review-rows')
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
		} else if (split === rowSplit && rows) {
			rows.getPanel('bottom')?.api.setSize({
				size: Math.max(
					workspacePanelSizes.bottom.minimum,
					Math.min(
						workspacePanelSizes.bottom.default,
						rows.height - workspacePanelSizes.tabs.minimum,
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
		const visibleTabs = getTabs()
		const nextTabLayoutKey = workspaceTabLayoutKey(visibleTabs)
		const titles = new Map(visibleTabs.map((tab) => [tab, getTitle(tab)]))
		if (!tabs || !tabsReady) return
		if (currentTabLayoutKey !== nextTabLayoutKey) {
			if (currentTabLayoutKey) tabLayouts.set(currentTabLayoutKey, tabs.toJSON())
			currentTabLayoutKey = nextTabLayoutKey
			const savedTabLayout = tabLayouts.get(nextTabLayoutKey)
			if (savedTabLayout && !tabs.getPopouts().length) {
				try {
					tabs.fromJSON(dockPopoutLayout(savedTabLayout))
				} catch {
					tabs.clear()
					tabLayouts.delete(nextTabLayoutKey)
				}
			}
		}
		for (const tab of projectReviewTabs) {
			const panel = tabs.getPanel(tab)
			if (!visibleTabs.includes(tab)) {
				if (panel) tabs.removePanel(panel)
				popoutSources.delete(tab)
				continue
			}
			if (!panel) {
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
					position: referencePanel ? { referencePanel: referencePanel.id } : undefined,
				})
			} else {
				panel.api.setTitle(titles.get(tab) ?? tab)
			}
		}
	}

	watchEffect(syncTabs)

	window.addEventListener('pagehide', saveLayout)
	onBeforeUnmount(() => {
		saveLayout()
		tabsReady = false
		window.removeEventListener('pagehide', saveLayout)
		subscriptions.forEach((subscription) => subscription.dispose())
		popoutCleanups.forEach((cleanup) => cleanup())
	})

	return {
		closeExternalPanels,
		detachedTabs,
		popoutTab,
		dockTab,
		poppingOut,
		bottomVisible,
		leftVisible,
		openTab,
		revealSlot,
		resetActiveTabs,
		rightVisible,
		visibleRightWidth,
		topLeftGroupId,
		topRightGroupId,
		onColumnsReady,
		onRowsReady,
		onTabsReady,
		onDividerDoubleClick,
		toggleSidebar,
		toggleToolsPanel,
	}
}
