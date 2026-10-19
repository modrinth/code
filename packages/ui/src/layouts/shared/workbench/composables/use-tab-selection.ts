import type { DockviewApi, IDockviewPanel } from 'dockview-vue'
import { shallowRef } from 'vue'

import type { WorkbenchWindows } from './use-windows'

/** The attribute on a tab's content naming the panel it belongs to. */
export const WORKBENCH_TAB_ATTRIBUTE = 'data-workbench-panel'

/**
 * Selecting several tabs of a group (shift click for a range, ctrl or cmd click to toggle), which
 * are then dragged together. Spans every dock it's registered with.
 */
export function useWorkbenchSelection(windows: WorkbenchWindows) {
	const selectedPanels = shallowRef<readonly string[]>([])
	const apis = new Set<DockviewApi>()
	let anchor: IDockviewPanel | undefined
	let drag:
		| {
				panel: IDockviewPanel
				panels: IDockviewPanel[]
				pendingPopout: boolean
		  }
		| undefined

	function clear() {
		selectedPanels.value = []
		anchor = undefined
	}

	function findPanel(id: string) {
		for (const api of apis) {
			const panel = api.getPanel(id)
			if (panel) return panel
		}
		return undefined
	}

	function panelAt(event: Event) {
		const target = event.target as Element | null
		const tab = target
			?.closest('.dv-tab')
			?.querySelector<HTMLElement>(`[${WORKBENCH_TAB_ATTRIBUTE}]`)
		const id = tab?.getAttribute(WORKBENCH_TAB_ATTRIBUTE)
		return id ? findPanel(id) : undefined
	}

	windows.listen(
		'pointerdown',
		(event) => {
			if (event.button !== 0) return
			drag = undefined
			const panel = panelAt(event)
			if (!panel) {
				clear()
				return
			}
			const toggle = event.metaKey || event.ctrlKey
			if (event.shiftKey || toggle) {
				event.preventDefault()
				event.stopPropagation()
				const group = panel.group
				const previous = selectedPanels.value.filter((id) => findPanel(id)?.group === group)
				if (event.shiftKey) {
					const start = anchor?.group === group ? anchor : (group.activePanel ?? panel)
					const from = group.panels.indexOf(start)
					const to = group.panels.indexOf(panel)
					const range = group.panels.slice(Math.min(from, to), Math.max(from, to) + 1)
					selectedPanels.value = [
						...new Set([...(toggle ? previous : []), ...range.map((entry) => entry.id)]),
					]
					anchor = start
				} else {
					selectedPanels.value = previous.includes(panel.id)
						? previous.filter((id) => id !== panel.id)
						: [...previous, panel.id]
					anchor = panel
				}
			} else {
				if (!selectedPanels.value.includes(panel.id)) selectedPanels.value = []
				anchor = panel
			}
		},
		{ capture: true },
	)
	windows.listen(
		'click',
		(event) => {
			const panel = panelAt(event)
			if (panel && (event.shiftKey || event.metaKey || event.ctrlKey)) {
				event.preventDefault()
				event.stopPropagation()
			} else if (panel) selectedPanels.value = []
			else clear()
		},
		{ capture: true },
	)
	windows.listen('keydown', (event) => {
		if (event.key === 'Escape') {
			clear()
			drag = undefined
		}
	})
	windows.listen('dragend', () => {
		const ended = drag
		queueMicrotask(() => {
			if (drag === ended && !ended?.pendingPopout) drag = undefined
		})
	})

	return {
		selectedPanels,
		/** The selected panels being dragged along with the given one, if it's part of a selection. */
		getDragPanels(id: string) {
			return drag?.panel.id === id ? drag.panels : undefined
		},
		/** Keeps the dragged selection around while it's popped out, until the returned function runs. */
		keepDragForPopout() {
			const pending = drag
			if (pending) pending.pendingPopout = true
			return () => {
				if (drag === pending) drag = undefined
			}
		},
		register(api: DockviewApi) {
			apis.add(api)
			let disposed = false
			const subscriptions = [
				api.onWillDragPanel(({ panel }) => {
					const panels = panel.group.panels.filter((entry) =>
						selectedPanels.value.includes(entry.id),
					)
					drag =
						panels.length > 1 && panels.includes(panel)
							? { panel, panels, pendingPopout: false }
							: undefined
				}),
				api.onDidMovePanel(({ panel }) => {
					const moved = drag
					if (!moved || moved.panel !== panel) return
					drag = undefined
					queueMicrotask(() => {
						if (disposed || api.getPanel(panel.id) !== panel) return
						const group = panel.group
						const panels = moved.panels.filter((entry) => api.getPanel(entry.id) === entry)
						const before = group.panels.slice(0, group.panels.indexOf(panel))
						const index = before.filter((entry) => !panels.includes(entry)).length
						const order = group.panels.filter((entry) => !panels.includes(entry))
						order.splice(index, 0, ...panels)
						for (const [position, entry] of order.entries()) {
							if (group.panels[position] !== entry) {
								entry.api.moveTo({
									group,
									position: 'center',
									index: position,
									skipSetActive: true,
								})
							}
						}
						panel.api.setActive()
					})
				}),
				api.onDidRemovePanel((panel) => {
					selectedPanels.value = selectedPanels.value.filter((id) => id !== panel.id)
					if (anchor === panel) anchor = undefined
				}),
				api.onDidLayoutFromJSON(clear),
			]
			return {
				dispose() {
					disposed = true
					apis.delete(api)
					for (const subscription of subscriptions) subscription.dispose()
				},
			}
		},
	}
}
