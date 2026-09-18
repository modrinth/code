import type {
	DockviewApi,
	DockviewIDisposable,
	DockviewReadyEvent,
	DockviewWillDropEvent,
	DockviewWillShowOverlayLocationEvent,
	SplitviewApi,
	SplitviewReadyEvent,
} from 'dockview-vue'
import { getGridLocation, LayoutPriority } from 'dockview-vue'
import { onBeforeUnmount, ref, watchEffect } from 'vue'

import { readWorkspaceLayout, saveWorkspaceLayout, workspacePanelSizes } from './layout-storage'
import { type ProjectReviewTab, projectReviewTabs } from './types'
import { useSidebarTransition } from './use-sidebar-transition'

export function useProjectReviewLayout(
	getTitle: (tab: ProjectReviewTab) => string,
	getTabs: () => readonly ProjectReviewTab[],
) {
	const savedLayout = readWorkspaceLayout()
	const leftVisible = ref(savedLayout?.leftVisible ?? true)
	const rightVisible = ref(savedLayout?.rightVisible ?? true)
	const { centerElement, transitionSidebar, finishSidebarTransition } = useSidebarTransition()
	let leftWidth = savedLayout?.leftWidth ?? workspacePanelSizes.left.default
	let rightWidth = savedLayout?.rightWidth ?? workspacePanelSizes.right.default
	let bottomHeight = savedLayout?.bottomHeight ?? workspacePanelSizes.bottom.default
	const topLeftGroupId = ref<string>()
	const topRightGroupId = ref<string>()
	let columns: SplitviewApi | undefined
	let rows: SplitviewApi | undefined
	let tabsReady = false
	let saveTimer: ReturnType<typeof setTimeout> | undefined
	let tabs: DockviewApi | undefined
	const subscriptions: DockviewIDisposable[] = []

	function saveLayout() {
		clearTimeout(saveTimer)
		if (!columns || !rows || !tabs || !tabsReady) return
		const currentLeftWidth = columns.getPanel('left')?.api.width ?? 0
		const currentRightWidth = columns.getPanel('right')?.api.width ?? 0
		const currentBottomHeight = rows.getPanel('bottom')?.api.height ?? 0
		if (leftVisible.value && currentLeftWidth > 0) leftWidth = currentLeftWidth
		if (rightVisible.value && currentRightWidth > 0) rightWidth = currentRightWidth
		if (currentBottomHeight > 0) bottomHeight = currentBottomHeight
		saveWorkspaceLayout({
			leftWidth,
			rightWidth,
			bottomHeight,
			leftVisible: leftVisible.value,
			rightVisible: rightVisible.value,
			tabs: tabs.toJSON(),
		})
	}

	function scheduleSave() {
		clearTimeout(saveTimer)
		saveTimer = setTimeout(saveLayout, 200)
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
		subscriptions.push(api.onDidLayoutChange(scheduleSave))
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
		subscriptions.push(api.onDidLayoutChange(scheduleSave))
	}

	function constrainDrop(event: DockviewWillDropEvent | DockviewWillShowOverlayLocationEvent) {
		if (!event.getData()) {
			event.preventDefault()
		}
	}

	function updateCornerGroups() {
		const groups =
			tabs?.groups.map((group) => ({
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
			api.onWillShowOverlay(constrainDrop),
			api.onWillDrop(constrainDrop),
			api.onDidLayoutChange(updateCornerGroups),
			api.onDidLayoutChange(scheduleSave),
			api.onDidActivePanelChange(scheduleSave),
		)
		let restored = false
		if (savedLayout) {
			try {
				api.fromJSON(savedLayout.tabs)
				restored = true
			} catch {
				api.clear()
			}
		}
		tabsReady = true
		syncTabs()
		if (!restored) api.getPanel('description')?.api.setActive()
		updateCornerGroups()
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
		void transitionSidebar(columns, side, visible.value)
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
			before.size = Math.round(total / 2)
			after.size = total - before.size
			tabs.fromJSON(layout, { reuseExistingPanels: true })
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
		const titles = new Map(visibleTabs.map((tab) => [tab, getTitle(tab)]))
		if (!tabs || !tabsReady) return
		for (const tab of projectReviewTabs) {
			const panel = tabs.getPanel(tab)
			if (!visibleTabs.includes(tab)) {
				if (panel) tabs.removePanel(panel)
				continue
			}
			if (!panel) {
				tabs.addPanel({
					id: tab,
					component: 'ProjectReviewPanel',
					title: titles.get(tab),
					params: { tab, slot: tab },
					renderer: 'always',
					minimumHeight: 120,
					inactive: true,
					position: tab === 'description' ? undefined : { referencePanel: 'description' },
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
	})

	return {
		centerElement,
		finishSidebarTransition,
		leftVisible,
		rightVisible,
		topLeftGroupId,
		topRightGroupId,
		onColumnsReady,
		onRowsReady,
		onTabsReady,
		onDividerDoubleClick,
		toggleSidebar,
	}
}
