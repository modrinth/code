import { useIntervalFn } from '@vueuse/core'
import type {
	DockviewApi,
	DockviewGroupPanel,
	DockviewIDisposable,
	DockviewPopoutGroupOptions,
	IDockviewPanel,
	SerializedDockview,
} from 'dockview-vue'
import { ref, type Ref } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { injectNotificationManager } from '#ui/providers/web-notifications'

import type { WorkbenchPopoutOptions } from '../types'
import type { WorkbenchWindows } from './workbench-windows'

export type PopoutTarget = IDockviewPanel | DockviewGroupPanel
type PopoutLayout = NonNullable<SerializedDockview['popoutGroups']>[number]
type GridNode = SerializedDockview['grid']['root']
type GroupNode = Exclude<GridNode['data'], GridNode[]>

const messages = defineMessages({
	popoutFailed: {
		id: 'workbench.popout.failed',
		defaultMessage: 'Could not open the panel window',
	},
	restoreTitle: {
		id: 'workbench.popout.restore-title',
		defaultMessage: 'Popout window closed',
	},
	restoreDescription: {
		id: 'workbench.popout.restore-description',
		defaultMessage: 'Panels from this window have been moved to the main window: {panels}.',
	},
	reopen: {
		id: 'workbench.popout.reopen',
		defaultMessage: 'Reopen window',
	},
	keepClosed: {
		id: 'workbench.popout.keep-closed',
		defaultMessage: 'Keep closed',
	},
})

/** The ids of the panels a saved popout window held. */
function popoutPanelIds(popout: PopoutLayout) {
	const ids: string[] = []
	function collect(node: GridNode) {
		if (Array.isArray(node.data)) node.data.forEach(collect)
		else ids.push(...node.data.views)
	}
	if (popout.grid) collect(popout.grid.root)
	else if (popout.data) ids.push(...popout.data.views)
	return ids
}

/** The layout with its popout windows' panels returned to the grid, beside where they came from. */
export function dockPopoutLayout(layout: SerializedDockview): SerializedDockview {
	const result: SerializedDockview = structuredClone(layout)
	function findGroupPath(node: GridNode, id: string): GridNode[] | undefined {
		if (!Array.isArray(node.data)) return node.data.id === id ? [node] : undefined
		for (const child of node.data) {
			const path = findGroupPath(child, id)
			if (path) return [node, ...path]
		}
	}
	function restoreGroup(group: GroupNode, reference?: string) {
		const path = reference ? findGroupPath(result.grid.root, reference) : undefined
		const existing = path?.at(-1)
		if (existing && !Array.isArray(existing.data)) {
			existing.data.views.push(...group.views)
			existing.data.activeView ??= group.activeView
			for (const node of path ?? []) node.visible = true
			return
		}
		const leaf: GridNode = { type: 'leaf', data: group }
		if (Array.isArray(result.grid.root.data)) result.grid.root.data.push(leaf)
		else result.grid.root = { type: 'branch', data: [result.grid.root, leaf] }
	}
	function restoreGrid(node: GridNode, reference?: string) {
		if (Array.isArray(node.data)) {
			for (const child of node.data) restoreGrid(child, reference)
		} else restoreGroup(node.data, reference)
	}
	for (const popout of result.popoutGroups ?? []) {
		if (popout.data) restoreGroup(popout.data, popout.gridReferenceGroup)
		if (popout.grid) restoreGrid(popout.grid.root, popout.gridReferenceGroup)
	}
	delete result.popoutGroups
	if (result.activeGroup && !findGroupPath(result.grid.root, result.activeGroup)) {
		delete result.activeGroup
	}
	return result
}

/** Disposes a panel that's no longer offered while keeping its popout window open for it to return to. */
function releasePopoutPanel(panel: IDockviewPanel) {
	const group = panel.group
	group.model.removePanel(panel)
	group.model.renderContainer.detatch(panel)
	panel.dispose()
}

/** Reopens one saved window without replacing any live popouts. */
async function restorePopout(api: DockviewApi, popout: PopoutLayout) {
	function dockedLayout(node: GridNode): GridNode | undefined {
		if (Array.isArray(node.data)) {
			const children = node.data.map(dockedLayout).filter((child): child is GridNode => !!child)
			return children.length ? { ...node, data: children } : undefined
		}
		const views = node.data.views.filter((id) => api.getPanel(id)?.api.location.type === 'grid')
		if (!views.length) return
		return {
			...node,
			data: {
				...node.data,
				views,
				activeView: views.includes(node.data.activeView ?? '') ? node.data.activeView : views[0],
			},
		}
	}
	function firstPanel(node: GridNode): IDockviewPanel | undefined {
		if (!Array.isArray(node.data)) return api.getPanel(node.data.views[0])
		return node.data.length ? firstPanel(node.data[0]) : undefined
	}
	function prepareGroups(node: GridNode) {
		if (Array.isArray(node.data)) {
			for (const child of node.data) prepareGroups(child)
			return
		}
		const panels = node.data.views
			.map((id) => api.getPanel(id))
			.filter((panel): panel is IDockviewPanel => !!panel && panel.api.location.type === 'grid')
		if (!panels.length) return
		const existing = panels[0].group
		const group =
			existing.panels.length === panels.length && panels.every((panel) => panel.group === existing)
				? existing
				: api.addGroup()
		for (const panel of panels) {
			if (panel.group !== group) panel.api.moveTo({ group, position: 'center' })
		}
		if (node.data.activeView) api.getPanel(node.data.activeView)?.api.setActive()
	}
	function restoreGrid(node: GridNode, horizontal: boolean) {
		if (!Array.isArray(node.data)) return
		const children = node.data
		for (let index = 1; index < children.length; index++) {
			const group = firstPanel(children[index])?.group
			const reference = firstPanel(children[index - 1])?.group
			if (group && reference) {
				group.api.moveTo({ group: reference, position: horizontal ? 'right' : 'bottom' })
			}
		}
		for (const child of children) restoreGrid(child, !horizontal)
	}
	function restoreSizes(node: GridNode, horizontal: boolean, width: number, height: number) {
		if (!Array.isArray(node.data)) {
			firstPanel(node)?.group.api.setSize({ width, height })
			return
		}
		const total = node.data.reduce((sum, child) => sum + (child.size ?? 1), 0)
		for (const child of node.data) {
			const ratio = (child.size ?? 1) / total
			restoreSizes(
				child,
				!horizontal,
				horizontal ? width * ratio : width,
				horizontal ? height : height * ratio,
			)
		}
	}
	try {
		const root = dockedLayout(popout.grid?.root ?? { type: 'leaf', data: popout.data! })
		if (!root) return true
		const panel = firstPanel(root)
		if (!panel) return false
		prepareGroups(root)
		const opened = await api.addPopoutGroup(panel.group, {
			position: popout.position ?? undefined,
			popoutUrl: popout.url,
		})
		if (opened && popout.grid) {
			const horizontal = popout.grid.orientation === 'HORIZONTAL'
			restoreGrid(root, horizontal)
			const child = panel.api.getWindow()
			restoreSizes(root, horizontal, child.innerWidth, child.innerHeight)
		}
		return opened
	} catch {
		return false
	}
}

/**
 * Dresses a popout window like the workbench: a bare copy of the workbench's root (keeping its
 * classes, so its styles apply) around the popout's content, and the main document's theme,
 * kept in sync while the window is open.
 */
function configurePopoutWindow(child: Window, root: HTMLElement | null) {
	const container = child.document.querySelector<HTMLElement>('.dv-popout-window')
	if (root && container) {
		const frame = root.cloneNode(false) as HTMLElement
		frame.style.height = '100dvh'
		container.style.width = '100%'
		container.style.flex = '1 1 auto'
		container.style.minHeight = '0'
		container.before(frame)
		frame.appendChild(container)
	}

	function syncAppearance() {
		for (const [source, target] of [
			[document.documentElement, child.document.documentElement],
			[document.body, child.document.body],
		] as const) {
			for (const name of ['class', 'style', 'data-theme', 'lang', 'dir']) {
				const value = source.getAttribute(name)
				if (name === 'style' && source === document.body) {
					const targetStyle = (target as HTMLElement).style
					const overflow = targetStyle.overflow
					targetStyle.cssText = value ?? ''
					targetStyle.overflow = overflow
					continue
				}
				if (value === null) target.removeAttribute(name)
				else target.setAttribute(name, value)
			}
		}
	}

	syncAppearance()
	const observer = new MutationObserver(syncAppearance)
	for (const element of [document.documentElement, document.body]) {
		observer.observe(element, {
			attributes: true,
			attributeFilter: ['class', 'style', 'data-theme', 'lang', 'dir'],
		})
	}
	child.focus()
	return () => observer.disconnect()
}

/** Dockview opens popouts through the main window; this uses the gesture's window, keeping its activation. */
function openPopoutGroup(
	api: DockviewApi,
	target: PopoutTarget,
	options: DockviewPopoutGroupOptions,
	sourceWindow = target.api.getWindow(),
) {
	if (sourceWindow === window) return api.addPopoutGroup(target, options)
	const originalOpen = window.open
	window.open = sourceWindow.open.bind(sourceWindow)
	try {
		return api.addPopoutGroup(target, options)
	} finally {
		window.open = originalOpen
	}
}

/**
 * Pops a tab or group out when it's dragged off the workbench (or dropped on its bare
 * background), into a window placed where it was released.
 */
function registerDragPopouts(
	api: DockviewApi,
	root: () => HTMLElement | null,
	popout: (
		target: PopoutTarget,
		position: NonNullable<DockviewPopoutGroupOptions['position']>,
		sourceWindow: Window,
	) => void,
): DockviewIDisposable {
	let clearDrag: (() => void) | undefined

	function onDragStart(
		target: PopoutTarget,
		group: DockviewGroupPanel,
		nativeEvent: DragEvent | PointerEvent,
	) {
		clearDrag?.()
		if ('pointerId' in nativeEvent) return
		const eventTarget = nativeEvent.target as HTMLElement | null
		const ownerWindow = eventTarget?.ownerDocument.defaultView
		const dock = group.element.closest('.dv-popout-window, .workbench-dock')
		if (!eventTarget || !ownerWindow || !dock) return
		const source = eventTarget
		const sourceWindow = ownerWindow
		const bounds = dock.getBoundingClientRect()
		const windowOrigin = {
			left: nativeEvent.screenX - nativeEvent.clientX - sourceWindow.screenX,
			top: nativeEvent.screenY - nativeEvent.clientY - sourceWindow.screenY,
		}
		const panelIds = new Set('panels' in target ? target.panels.map((panel) => panel.id) : [target.id])
		const windows = [window, ...api.getPopouts().map((entry) => entry.window)]
		let dropCleanupTimer: ReturnType<typeof setTimeout> | undefined
		let nativeDrop: DragEvent | undefined
		let moved = false
		let dropped = false
		let cancelled = false
		const movedSubscription = api.onDidMovePanel((event) => {
			if (panelIds.has(event.panel.id)) moved = true
		})
		const dropSubscription = api.onDidDrop(() => {
			dropped = true
		})

		function openPopout(event: DragEvent) {
			popout(
				target,
				{
					left: event.screenX,
					top: event.screenY,
					width: Math.max(400, bounds.width),
					height: Math.max(300, bounds.height),
				},
				sourceWindow,
			)
		}

		function isWorkbenchBackground(event: DragEvent) {
			const element = event.target as Element | null
			const workbench = root()
			return (
				!!workbench &&
				!!element &&
				workbench.contains(element) &&
				!element.closest(
					'.workbench-dock, .dv-popout-window, .dv-drop-target, .dv-drop-target-container',
				)
			)
		}

		function onDragOver(event: DragEvent) {
			if (event.defaultPrevented || !isWorkbenchBackground(event)) return
			event.preventDefault()
			if (event.dataTransfer) event.dataTransfer.dropEffect = 'move'
		}

		function observeDrop(event: DragEvent) {
			nativeDrop = event
			dropCleanupTimer = setTimeout(() => onDragEnd(event), 0)
		}

		function onBackgroundDrop(event: DragEvent) {
			if (event.defaultPrevented || cancelled || moved || !isWorkbenchBackground(event)) return
			event.preventDefault()
			dropped = true
			openPopout(event)
		}

		function onDragEnd(event: DragEvent) {
			const release = nativeDrop ?? event
			const clientX = release.screenX - sourceWindow.screenX - windowOrigin.left
			const clientY = release.screenY - sourceWindow.screenY - windowOrigin.top
			const destination = (nativeDrop?.target as Node | null)?.ownerDocument
			const outside =
				(destination && destination !== source.ownerDocument) ||
				clientX < bounds.left ||
				clientX > bounds.right ||
				clientY < bounds.top ||
				clientY > bounds.bottom
			const shouldPopout =
				!cancelled &&
				!moved &&
				!dropped &&
				(release.screenX !== 0 || release.screenY !== 0) &&
				outside
			clearDrag?.()
			if (shouldPopout) openPopout(release)
		}

		function onKeydown(event: KeyboardEvent) {
			if (event.key === 'Escape') cancelled = true
		}

		function onWindowClose() {
			cancelled = true
			clearDrag?.()
		}

		function cleanup() {
			if (clearDrag === cleanup) clearDrag = undefined
			clearTimeout(dropCleanupTimer)
			source.removeEventListener('dragend', onDragEnd)
			for (const child of windows) {
				child.removeEventListener('dragover', onDragOver)
				child.removeEventListener('drop', observeDrop, true)
				child.removeEventListener('drop', onBackgroundDrop)
				child.removeEventListener('keydown', onKeydown, true)
				child.removeEventListener('beforeunload', onWindowClose)
			}
			movedSubscription.dispose()
			dropSubscription.dispose()
		}
		clearDrag = cleanup
		source.addEventListener('dragend', onDragEnd)
		for (const child of windows) {
			child.addEventListener('dragover', onDragOver)
			child.addEventListener('drop', observeDrop, { capture: true, passive: true })
			child.addEventListener('drop', onBackgroundDrop)
			child.addEventListener('keydown', onKeydown, true)
			child.addEventListener('beforeunload', onWindowClose)
		}
	}

	const subscriptions = [
		api.onWillDragPanel(({ panel, nativeEvent }) => onDragStart(panel, panel.group, nativeEvent)),
		api.onWillDragGroup(({ group, nativeEvent }) => onDragStart(group, group, nativeEvent)),
	]
	return {
		dispose() {
			clearDrag?.()
			subscriptions.forEach((subscription) => subscription.dispose())
		},
	}
}

/**
 * What popouts need from the component setting the workbench up, which can only be had during
 * its setup, while docks (and their popouts) come up later.
 */
export function usePopoutServices() {
	const { addNotification, removeNotification } = injectNotificationManager()
	const { formatMessage } = useVIntl()
	return { addNotification, removeNotification, formatMessage }
}

export type PopoutServices = ReturnType<typeof usePopoutServices>

export interface DockPopoutsOptions {
	api: DockviewApi
	services: PopoutServices
	options: WorkbenchPopoutOptions
	windows: WorkbenchWindows
	root: () => HTMLElement | null
	/** Whether the layout can be changed (e.g. it's loaded, and the page isn't being left). */
	canChange: () => boolean
	titleOf: (id: string) => string
	/** The layout changed in a way that should be saved. */
	onChange: () => void
	/** The panels being dragged along with one (a selection of tabs), if any. */
	dragPanels?: (id: string) => IDockviewPanel[] | undefined
	/** Keeps a dragged selection around while it's popped out, until the returned function runs. */
	keepDrag?: () => () => void
}

export type DockPopouts = ReturnType<typeof createDockPopouts>

/**
 * Popping a dock's panels out into their own windows: dragged off the workbench, or on request.
 * Saved layouts keep the windows; since browsers only open them from a user gesture, reopening
 * them is offered through a notification, while their panels wait in the main window.
 */
export function createDockPopouts(config: DockPopoutsOptions) {
	const { api, options, windows } = config
	const { addNotification, removeNotification, formatMessage } = config.services

	const detached = ref<string[]>([])
	const hasPopouts = ref(false)
	const poppingOut = ref<string[]>([])
	/** Where each popped out panel came from, to return it there. */
	const sources = new Map<string, string>()
	/** Panels no longer offered that were popped out, to return to their window if offered again. */
	const releasedPanels = new Map<string, { group: DockviewGroupPanel; index: number }>()
	const windowCleanups = new Map<Window, () => void>()
	const pending = new Map<PopoutLayout, { notificationId?: string | number; opening: boolean }>()
	/** The saved layout, kept (popouts and all) while its windows wait to be reopened. */
	let blockedLayout: SerializedDockview | undefined
	let blockedBaseline: string | undefined

	const { pause: stopWatchingWindows, resume: watchWindows } = useIntervalFn(
		() => {
			for (const id of detached.value) {
				const panel = api.getPanel(id)
				if (panel?.api.location.type === 'popout' && panel.api.getWindow().closed) dock(id)
			}
		},
		500,
		{ immediate: false },
	)

	function updateDetached() {
		hasPopouts.value = api.getPopouts().length > 0
		detached.value = api.panels
			.filter((panel) => panel.api.location.type === 'popout')
			.map((panel) => panel.id)
		if (detached.value.length) watchWindows()
		else stopWatchingWindows()
	}

	function restoring() {
		return [...pending.values()].some((entry) => entry.opening)
	}

	/** The layout to save: the blocked one while its windows wait, unless the layout has changed since. */
	function currentLayout() {
		const layout = api.toJSON()
		if (blockedLayout && !restoring() && blockedBaseline !== JSON.stringify(layout)) {
			blockedLayout = undefined
			blockedBaseline = undefined
		}
		return blockedLayout ?? layout
	}

	function dismissNotifications() {
		for (const entry of pending.values()) {
			if (entry.notificationId !== undefined) removeNotification(entry.notificationId)
		}
		pending.clear()
	}

	function notifyRestoration(popout: PopoutLayout) {
		const entry = pending.get(popout)
		if (!entry || entry.notificationId !== undefined) return
		const panels = popoutPanelIds(popout).map(config.titleOf).join(', ')
		entry.notificationId = addNotification({
			id: crypto.randomUUID(),
			type: 'neutral',
			containerClass: '!py-1.5 !pr-0.5 !pb-2 !rounded-xl',
			title: formatMessage(messages.restoreTitle),
			text: formatMessage(messages.restoreDescription, { panels }),
			autoCloseMs: null,
			noIcon: true,
			buttons: [
				{
					label: formatMessage(messages.reopen),
					keepOpen: true,
					color: 'green',
					action: () => reopen(popout),
				},
				{
					label: formatMessage(messages.keepClosed),
					keepOpen: true,
					action: () => keepClosed(popout),
				},
			],
		}).id
	}

	function keepClosed(popout: PopoutLayout) {
		const entry = pending.get(popout)
		if (!entry || entry.opening) return
		if (entry.notificationId !== undefined) removeNotification(entry.notificationId)
		pending.delete(popout)
		if (blockedLayout) {
			const remaining = blockedLayout.popoutGroups?.filter((group) => group !== popout)
			blockedLayout = dockPopoutLayout({ ...blockedLayout, popoutGroups: [popout] })
			blockedLayout.popoutGroups = remaining
		}
		if (!pending.size) {
			blockedLayout = undefined
			blockedBaseline = undefined
		}
		config.onChange()
	}

	async function reopen(popout: PopoutLayout) {
		const entry = pending.get(popout)
		if (!entry || entry.opening || !config.canChange()) return
		currentLayout()
		entry.opening = true
		const opened = await restorePopout(api, popout)
		if (pending.get(popout) !== entry) return
		entry.opening = false
		if (opened) {
			if (entry.notificationId !== undefined) removeNotification(entry.notificationId)
			pending.delete(popout)
			if (!pending.size) blockedLayout = undefined
		} else notifyRestoration(popout)
		if (blockedLayout && !restoring()) blockedBaseline = JSON.stringify(api.toJSON())
		config.onChange()
	}

	/** Loads a saved layout, trying to reopen its windows, whose panels wait in the main window. */
	function restore(layout: SerializedDockview) {
		dismissNotifications()
		blockedLayout = undefined
		blockedBaseline = undefined
		if (!layout.popoutGroups?.length) {
			api.fromJSON(layout, { reuseExistingPanels: true })
			return
		}
		api.fromJSON(dockPopoutLayout(layout), { reuseExistingPanels: true })
		blockedLayout = layout
		blockedBaseline = JSON.stringify(api.toJSON())
		for (const popout of layout.popoutGroups) pending.set(popout, { opening: false })
		for (const popout of layout.popoutGroups) void reopen(popout)
	}

	function notifyFailure() {
		addNotification({ type: 'error', title: formatMessage(messages.popoutFailed) })
	}

	async function popout(
		target: PopoutTarget,
		position?: DockviewPopoutGroupOptions['position'],
		sourceWindow?: Window,
	) {
		const panels =
			'panels' in target ? [...target.panels] : (config.dragPanels?.(target.id) ?? [target])
		const ids = panels.map((panel) => panel.id)
		if (!panels.length || ids.some((id) => poppingOut.value.includes(id))) return
		if (target.api.location.type === 'popout' && !position) return
		for (const panel of panels) {
			if (panel.api.location.type === 'grid') sources.set(panel.id, panel.group.id)
		}
		poppingOut.value = [...poppingOut.value, ...ids]
		try {
			const opened = await openPopoutGroup(
				api,
				target,
				{ popoutUrl: options.url, position },
				sourceWindow,
			)
			if (!opened) notifyFailure()
		} catch {
			notifyFailure()
		} finally {
			poppingOut.value = poppingOut.value.filter((id) => !ids.includes(id))
		}
	}

	function popoutPanel(id: string) {
		const panel = api.getPanel(id)
		if (panel) return popout(panel)
	}

	/** Returns a popped out panel to the group it came from, or another one in the main window. */
	function dock(id: string) {
		const panel = api.getPanel(id)
		if (!panel || panel.api.location.type !== 'popout') return
		const source = api.groups.find((group) => group.id === sources.get(id))
		const group =
			source?.api.location.type === 'grid' && source.panels.length
				? source
				: (api.groups.find((entry) => entry.api.location.type === 'grid' && entry.panels.length) ??
					api.addGroup())
		panel.api.moveTo({ group, position: 'center' })
		panel.api.setActive()
		sources.delete(id)
		config.onChange()
	}

	/** Returns every popped out panel to the main window, dropping any windows waiting to reopen. */
	function closeAll() {
		blockedLayout = undefined
		dismissNotifications()
		for (const id of [...detached.value]) dock(id)
		releasedPanels.clear()
		for (const group of api.groups) {
			if (group.api.location.type === 'popout' && !group.panels.length) api.removeGroup(group)
		}
	}

	/** Shows a panel; one in a popout is brought to the front of its window. */
	function reveal(panel: IDockviewPanel) {
		if (panel.api.location.type !== 'popout') return false
		if (!panel.api.isVisible) panel.group.model.openPanel(panel, { skipSetGroupActive: true })
		panel.api.getWindow().focus()
		return true
	}

	/** Takes a panel that's no longer offered out of its window, remembering where to return it. */
	function release(panel: IDockviewPanel) {
		if (panel.api.location.type !== 'popout') return false
		const group = panel.group
		releasedPanels.set(panel.id, { group, index: group.panels.indexOf(panel) })
		releasePopoutPanel(panel)
		return true
	}

	/** Where a panel offered again should go: back to the window it was released from, if still open. */
	function positionFor(id: string) {
		const previous = releasedPanels.get(id)
		releasedPanels.delete(id)
		if (
			previous &&
			api.groups.includes(previous.group) &&
			previous.group.api.location.type === 'popout' &&
			!previous.group.api.getWindow().closed
		) {
			return { referenceGroup: previous.group.id, index: previous.index }
		}
		return undefined
	}

	const subscriptions: DockviewIDisposable[] = [
		registerDragPopouts(api, config.root, (target, position, sourceWindow) => {
			const releaseDrag = config.keepDrag?.()
			void popout(target, position, sourceWindow).finally(() => releaseDrag?.())
		}),
		api.onDidAddPopoutGroup(({ window: child }) => {
			updateDetached()
			const unregister = windows.register(child)
			const removeAppearance = configurePopoutWindow(child, config.root())
			const hostCleanup = options.onWindow?.(child)
			windowCleanups.set(child, () => {
				unregister()
				removeAppearance()
				hostCleanup?.()
			})
		}),
		api.onDidRemovePopoutGroup(() => {
			updateDetached()
			const open = new Set(api.getPopouts().map((entry) => entry.window))
			for (const [child, cleanup] of windowCleanups) {
				if (open.has(child)) continue
				cleanup()
				windowCleanups.delete(child)
			}
		}),
		api.onDidLayoutChange(updateDetached),
	]

	function dispose() {
		stopWatchingWindows()
		dismissNotifications()
		for (const subscription of subscriptions) subscription.dispose()
		for (const cleanup of windowCleanups.values()) cleanup()
		windowCleanups.clear()
	}

	return {
		detached: detached as Ref<string[]>,
		hasPopouts,
		poppingOut,
		restoring,
		currentLayout,
		restore,
		popoutPanel,
		dock,
		closeAll,
		reveal,
		release,
		positionFor,
		dispose,
	}
}
