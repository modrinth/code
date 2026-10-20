import type {
	DockviewApi,
	DockviewGroupPanel,
	DockviewIDisposable,
	DockviewWillDropEvent,
	DockviewWillShowOverlayLocationEvent,
	IDockviewPanel,
} from 'dockview-vue'
import { getGridLocation } from 'dockview-vue'

/**
 * Moves each group's tab strip into the header's free space, where it can wrap onto several
 * lines, while keeping it inside the header's own drag and drop target. Disposing puts the strips
 * back, so the workbench can switch styles.
 */
export function registerWrappedTabs(api: DockviewApi): DockviewIDisposable {
	let disposed = false
	const cleanups = new Map<DockviewGroupPanel, () => void>()

	/** Lets individual tab targets handle reordering before the enclosing header claims the drop. */
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
		if (!header || !space || strip.parentElement !== header) return
		const nextSibling = strip.nextSibling
		space.appendChild(strip)
		let restoreDraggable: (() => void) | undefined

		/** Keeps the browser from picking an adjacent draggable tab when a drag starts in a narrow gap. */
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
			if (strip.parentElement === space && header.isConnected) {
				header.insertBefore(strip, nextSibling?.parentNode === header ? nextSibling : space)
			}
		})
	}

	for (const group of api.groups) wrapGroup(group)
	const addSubscription = api.onDidAddGroup((group) => {
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
			addSubscription.dispose()
			removeSubscription.dispose()
			for (const cleanup of cleanups.values()) cleanup()
			cleanups.clear()
		},
	}
}

/** Ignores drags that carry no panel (e.g. files from outside), so docks don't offer to take them. */
export function registerPanelOnlyDrops(api: DockviewApi): DockviewIDisposable {
	function constrain(event: DockviewWillDropEvent | DockviewWillShowOverlayLocationEvent) {
		if (!event.getData()) event.preventDefault()
	}
	const subscriptions = [api.onWillShowOverlay(constrain), api.onWillDrop(constrain)]
	return { dispose: () => subscriptions.forEach((subscription) => subscription.dispose()) }
}

/** Keeps panels from being dropped beside a group or on the dock's edges, which would split it. */
export function registerNoSplits(api: DockviewApi): DockviewIDisposable {
	return api.onWillShowOverlay((event) => {
		if (event.kind === 'content' || event.kind === 'edge') event.preventDefault()
	})
}

/** Splits a group evenly with the one a panel was dropped beside, rather than taking a sliver. */
export function registerEvenSplits(api: DockviewApi, enabled: () => boolean): DockviewIDisposable {
	return api.onWillDrop((event: DockviewWillDropEvent) => {
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
		let movedPanel: IDockviewPanel | undefined
		const subscription = event.api.onDidMovePanel(({ panel, from }) => {
			if (from.id === data.groupId && (!data.panelId || panel.id === data.panelId)) {
				movedPanel = panel
			}
		})
		queueMicrotask(() => {
			subscription.dispose()
			const moved = movedPanel?.group
			if (!enabled() || !moved || moved === target || !event.api.groups.includes(target)) return
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
	})
}

/**
 * Evens out the two sides of a sash within a dock: the groups' sizes when some are popped out
 * (whose layout can't be reloaded), otherwise its saved sizes, reloaded in place.
 */
export function evenOutDockSash(api: DockviewApi, sash: Element) {
	const container = sash.parentElement
	const split = container?.parentElement
	const branch = split?.parentElement
	if (!container || !split || !branch?.classList.contains('dv-branch-node')) return false
	const index = Array.from(container.children).indexOf(sash)
	const layout = api.toJSON()
	let node = layout.grid.root
	for (const childIndex of getGridLocation(branch)) {
		if (!Array.isArray(node.data) || !node.data[childIndex]) return false
		node = node.data[childIndex]
	}
	if (!Array.isArray(node.data)) return false
	const before = node.data[index]
	const after = node.data[index + 1]
	if (!before?.size || !after?.size) return false
	const total = before.size + after.size

	if (api.getPopouts().length) {
		const horizontal = split.classList.contains('dv-horizontal')
		const sizes: { id: string; size: number }[] = []
		function collectSizes(child: typeof layout.grid.root, scale: number) {
			if (Array.isArray(child.data)) {
				for (const entry of child.data) collectSizes(entry, scale)
				return
			}
			const id = child.data.id
			const group = api.groups.find((candidate) => candidate.id === id)
			if (group) {
				sizes.push({ id, size: (horizontal ? group.api.width : group.api.height) * scale })
			}
		}
		collectSizes(before, total / 2 / before.size)
		collectSizes(after, total / 2 / after.size)
		for (const { id, size } of sizes) {
			api.groups
				.find((group) => group.id === id)
				?.api.setSize(horizontal ? { width: size } : { height: size })
		}
	} else {
		before.size = Math.round(total / 2)
		after.size = total - before.size
		api.fromJSON(layout, { reuseExistingPanels: true })
	}
	return true
}

/** The top-left and top-right groups of a dock's grid, whose tab bars sit in its outer corners. */
export function findCornerGroups(api: DockviewApi) {
	const groups = api.groups
		.filter(
			(group) =>
				group.api.location.type === 'grid' && group.api.isVisible && group.panels.length > 0,
		)
		.map((group) => ({ id: group.id, bounds: group.element.getBoundingClientRect() }))
	const top = Math.min(...groups.map(({ bounds }) => bounds.top))
	const topGroups = groups.filter(({ bounds }) => Math.abs(bounds.top - top) < 1)
	topGroups.sort((a, b) => a.bounds.left - b.bounds.left)
	return { left: topGroups[0]?.id, right: topGroups[topGroups.length - 1]?.id }
}
