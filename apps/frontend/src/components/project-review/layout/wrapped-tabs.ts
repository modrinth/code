import type { DockviewApi, DockviewGroupPanel, DockviewIDisposable } from 'dockview-vue'

/** Keep wrapped tabs inside the native header-space drag and drop target. */
export function registerWrappedTabs(api: DockviewApi): DockviewIDisposable {
	let disposed = false
	const cleanups = new Map<DockviewGroupPanel, () => void>()
	/** Let individual tab targets handle reordering before the enclosing header claims the drop. */
	const overlaySubscription = api.onWillShowOverlay((event) => {
		if (event.kind !== 'header_space') return
		const target = event.nativeEvent.target as Element | null
		if (target?.closest('.dv-tab')) event.preventDefault()
	})
	function wrapGroup(group: DockviewGroupPanel) {
		if (cleanups.has(group)) return
		const tabs = group.model.tabsListElement
		const header = tabs.closest('.dv-tabs-and-actions-container')
		const space = header?.querySelector<HTMLElement>(':scope > .dv-void-container')
		const strip = tabs.parentElement?.classList.contains('dv-scrollable')
			? tabs.parentElement
			: tabs
		if (!space || strip.parentElement !== header) return
		space.appendChild(strip)
		let restoreDraggable: (() => void) | undefined
		/** Keep the browser from picking an adjacent draggable tab when a drag starts in a narrow gap. */
		function onPointerDown(event: PointerEvent) {
			restoreDraggable?.()
			if (event.button !== 0 || (event.target as Element).closest('.dv-tab')) return
			const ownerWindow = tabs.ownerDocument.defaultView
			if (!ownerWindow) return
			const draggable = [...tabs.querySelectorAll<HTMLElement>('.dv-tab')].map(
				(tab) => [tab, tab.draggable] as const,
			)
			for (const [tab] of draggable) tab.draggable = false
			function restore() {
				for (const [tab, enabled] of draggable) tab.draggable = enabled
				ownerWindow?.removeEventListener('pointerup', restore, true)
				ownerWindow?.removeEventListener('pagehide', restore)
				space?.removeEventListener('dragend', restore)
				restoreDraggable = undefined
			}
			restoreDraggable = restore
			ownerWindow.addEventListener('pointerup', restore, { capture: true, once: true })
			ownerWindow.addEventListener('pagehide', restore, { once: true })
			space?.addEventListener('dragend', restore, { once: true })
		}
		function onTabDragStart(event: DragEvent) {
			event.stopPropagation()
		}
		space.addEventListener('pointerdown', onPointerDown)
		tabs.addEventListener('dragstart', onTabDragStart)
		cleanups.set(group, () => {
			restoreDraggable?.()
			space.removeEventListener('pointerdown', onPointerDown)
			tabs.removeEventListener('dragstart', onTabDragStart)
		})
	}
	for (const group of api.groups) wrapGroup(group)
	const subscription = api.onDidAddGroup((group) => {
		queueMicrotask(() => {
			if (!disposed && api.groups.includes(group)) wrapGroup(group)
		})
	})
	const removeSubscription = api.onDidRemoveGroup((group) => {
		cleanups.get(group)?.()
		cleanups.delete(group)
	})
	return {
		dispose() {
			disposed = true
			overlaySubscription.dispose()
			subscription.dispose()
			removeSubscription.dispose()
			for (const cleanup of cleanups.values()) cleanup()
			cleanups.clear()
		},
	}
}
