import type {
	DockviewApi,
	DockviewIDisposable,
	DockviewReadyEvent,
	DockviewWillDropEvent,
	DockviewWillShowOverlayLocationEvent,
	SplitviewApi,
	SplitviewReadyEvent,
} from 'dockview-vue'
import { LayoutPriority } from 'dockview-vue'
import { onBeforeUnmount, ref, watchEffect } from 'vue'

import { readWorkspaceLayout, saveWorkspaceLayout } from './layout-storage'
import { type ProjectReviewTab, projectReviewTabs } from './types'

export function useProjectReviewLayout(getTitle: (tab: ProjectReviewTab) => string) {
	const savedLayout = readWorkspaceLayout()
	const leftVisible = ref(savedLayout?.leftVisible ?? true)
	const rightVisible = ref(savedLayout?.rightVisible ?? true)
	let leftWidth = savedLayout?.leftWidth ?? 260
	let rightWidth = savedLayout?.rightWidth ?? 300
	let bottomHeight = savedLayout?.bottomHeight ?? 180
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
			minimumSize: 180,
			size: leftWidth,
		})
		api.addPanel({
			id: 'center',
			component: 'ProjectReviewCenter',
			minimumSize: 320,
			priority: LayoutPriority.High,
		})
		api.addPanel({
			id: 'right',
			component: 'ProjectReviewPanel',
			params: { slot: 'right' },
			minimumSize: 200,
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
			minimumSize: 180,
			priority: LayoutPriority.High,
		})
		api.addPanel({
			id: 'bottom',
			component: 'ProjectReviewPanel',
			params: { slot: 'bottom' },
			minimumSize: 100,
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
				if (
					api.panels.length !== projectReviewTabs.length ||
					projectReviewTabs.some((tab) => !api.getPanel(tab))
				) {
					throw new Error('Incomplete saved review layout')
				}
				restored = true
			} catch {
				api.clear()
			}
		}
		if (!restored) {
			for (const tab of projectReviewTabs) {
				api.addPanel({
					id: tab,
					component: 'ProjectReviewPanel',
					title: getTitle(tab),
					params: { tab, slot: tab },
					renderer: 'always',
					minimumHeight: 120,
					position: tab === 'description' ? undefined : { referencePanel: 'description' },
				})
			}
			api.getPanel('description')?.api.setActive()
		}
		for (const tab of projectReviewTabs) {
			api.getPanel(tab)?.api.setTitle(getTitle(tab))
		}
		tabsReady = true
		updateCornerGroups()
	}

	function toggleSidebar(side: 'left' | 'right') {
		const visible = side === 'left' ? leftVisible : rightVisible
		const panel = columns?.getPanel(side)
		if (!panel) return
		if (visible.value) {
			if (side === 'left') leftWidth = panel.api.width
			else rightWidth = panel.api.width
		}
		visible.value = !visible.value
		panel.api.setVisible(visible.value)
		scheduleSave()
	}

	watchEffect(() => {
		for (const tab of projectReviewTabs) {
			const title = getTitle(tab)
			tabs?.getPanel(tab)?.api.setTitle(title)
		}
	})

	window.addEventListener('pagehide', saveLayout)
	onBeforeUnmount(() => {
		saveLayout()
		tabsReady = false
		window.removeEventListener('pagehide', saveLayout)
		subscriptions.forEach((subscription) => subscription.dispose())
	})

	return {
		leftVisible,
		rightVisible,
		topLeftGroupId,
		topRightGroupId,
		onColumnsReady,
		onRowsReady,
		onTabsReady,
		toggleSidebar,
	}
}
