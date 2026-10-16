import type { DockviewApi, IDockviewPanel } from 'dockview-vue'
import { shallowRef } from 'vue'

import { injectReviewShortcuts } from '../shortcuts'

export function useTabSelection() {
	const shortcuts = injectReviewShortcuts()
	const selectedTabs = shallowRef<readonly string[]>([])
	let api: DockviewApi | undefined
	let anchor: IDockviewPanel | undefined
	let drag:
		| {
				panel: IDockviewPanel
				panels: IDockviewPanel[]
				pendingPopout: boolean
		  }
		| undefined
	let disposed = false

	function clear() {
		selectedTabs.value = []
		anchor = undefined
	}

	function panelAt(event: Event) {
		const target = event.target as Element | null
		const tab = target?.closest('.dv-tab')?.querySelector<HTMLElement>('[data-review-tab]')
		return tab?.dataset.reviewTab ? api?.getPanel(tab.dataset.reviewTab) : undefined
	}

	const stopPointer = shortcuts.listen(
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
				const previous = selectedTabs.value.filter((id) => api?.getPanel(id)?.group === group)
				if (event.shiftKey) {
					const start = anchor?.group === group ? anchor : (group.activePanel ?? panel)
					const from = group.panels.indexOf(start)
					const to = group.panels.indexOf(panel)
					const range = group.panels.slice(Math.min(from, to), Math.max(from, to) + 1)
					selectedTabs.value = [
						...new Set([...(toggle ? previous : []), ...range.map((p) => p.id)]),
					]
					anchor = start
				} else {
					selectedTabs.value = previous.includes(panel.id)
						? previous.filter((id) => id !== panel.id)
						: [...previous, panel.id]
					anchor = panel
				}
			} else {
				if (!selectedTabs.value.includes(panel.id)) selectedTabs.value = []
				anchor = panel
			}
		},
		{ capture: true },
	)
	const stopClick = shortcuts.listen(
		'click',
		(event) => {
			const panel = panelAt(event)
			if (panel && (event.shiftKey || event.metaKey || event.ctrlKey)) {
				event.preventDefault()
				event.stopPropagation()
			} else if (panel) selectedTabs.value = []
			else clear()
		},
		{ capture: true },
	)
	const stopKey = shortcuts.listen('keydown', (event) => {
		if (event.key === 'Escape') {
			clear()
			drag = undefined
		}
	})
	const stopDragEnd = shortcuts.listen('dragend', () => {
		const ended = drag
		queueMicrotask(() => {
			if (drag === ended && !ended?.pendingPopout) drag = undefined
		})
	})

	return {
		selectedTabs,
		getDragPanels(id: string) {
			return drag?.panel.id === id ? drag.panels : undefined
		},
		keepDragForPopout() {
			const pending = drag
			if (pending) pending.pendingPopout = true
			return () => {
				if (drag === pending) drag = undefined
			}
		},
		register(target: DockviewApi) {
			api = target
			const subscriptions = [
				target.onWillDragPanel(({ panel }) => {
					const panels = panel.group.panels.filter((p) => selectedTabs.value.includes(p.id))
					drag =
						panels.length > 1 && panels.includes(panel)
							? { panel, panels, pendingPopout: false }
							: undefined
				}),
				target.onDidMovePanel(({ panel }) => {
					const moved = drag
					if (!moved || moved.panel !== panel) return
					drag = undefined
					queueMicrotask(() => {
						if (disposed || target.getPanel(panel.id) !== panel) return
						const group = panel.group
						const panels = moved.panels.filter((p) => target.getPanel(p.id) === p)
						const before = group.panels.slice(0, group.panels.indexOf(panel))
						const index = before.filter((p) => !panels.includes(p)).length
						const order = group.panels.filter((p) => !panels.includes(p))
						order.splice(index, 0, ...panels)
						for (const [index, panel] of order.entries()) {
							if (group.panels[index] !== panel)
								panel.api.moveTo({
									group,
									position: 'center',
									index,
									skipSetActive: true,
								})
						}
						panel.api.setActive()
					})
				}),
				target.onDidRemovePanel((panel) => {
					selectedTabs.value = selectedTabs.value.filter((id) => id !== panel.id)
					if (anchor === panel) anchor = undefined
				}),
				target.onDidLayoutFromJSON(clear),
			]
			return {
				dispose() {
					disposed = true
					drag = undefined
					clear()
					stopPointer()
					stopClick()
					stopKey()
					stopDragEnd()
					for (const subscription of subscriptions) subscription.dispose()
				},
			}
		},
	}
}
