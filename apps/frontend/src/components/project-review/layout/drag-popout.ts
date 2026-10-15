import type {
	DockviewApi,
	DockviewGroupPanel,
	DockviewIDisposable,
	DockviewPopoutGroupOptions,
} from 'dockview-vue'

import type { ReviewPopoutTarget } from './popout-window'

export function registerDragPopouts(
	api: DockviewApi,
	popout: (
		target: ReviewPopoutTarget,
		position: NonNullable<DockviewPopoutGroupOptions['position']>,
		sourceWindow: Window,
	) => void,
): DockviewIDisposable {
	let clearDrag: (() => void) | undefined

	function onDragStart(
		target: ReviewPopoutTarget,
		group: DockviewGroupPanel,
		nativeEvent: DragEvent | PointerEvent,
	) {
		clearDrag?.()
		if ('pointerId' in nativeEvent) return
		const eventTarget = nativeEvent.target as HTMLElement | null
		const ownerWindow = eventTarget?.ownerDocument.defaultView
		const dock = group.element.closest('.dv-popout-window, .project-review-tabs')
		if (!eventTarget || !ownerWindow || !dock) return
		const source = eventTarget
		const sourceWindow = ownerWindow
		const bounds = dock.getBoundingClientRect()
		const windowOrigin = {
			left: nativeEvent.screenX - nativeEvent.clientX - sourceWindow.screenX,
			top: nativeEvent.screenY - nativeEvent.clientY - sourceWindow.screenY,
		}
		const panelIds = new Set(
			'panels' in target ? target.panels.map((panel) => panel.id) : [target.id],
		)
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

		function isWorkspaceBackground(event: DragEvent) {
			const element = event.target as Element | null
			return (
				!!element?.closest('.project-review') &&
				!element.closest(
					'.project-review-tabs, .dv-popout-window, .project-review-dock, .dv-drop-target, .dv-drop-target-container',
				)
			)
		}

		function onDragOver(event: DragEvent) {
			if (event.defaultPrevented || !isWorkspaceBackground(event)) return
			event.preventDefault()
			if (event.dataTransfer) event.dataTransfer.dropEffect = 'move'
		}

		function observeDrop(event: DragEvent) {
			nativeDrop = event
			dropCleanupTimer = setTimeout(() => onDragEnd(event), 0)
		}

		function onBackgroundDrop(event: DragEvent) {
			if (event.defaultPrevented || cancelled || moved || !isWorkspaceBackground(event)) return
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
		api.onWillDragPanel(({ panel, nativeEvent }) => {
			onDragStart(panel, panel.group, nativeEvent)
		}),
		api.onWillDragGroup(({ group, nativeEvent }) => {
			onDragStart(group, group, nativeEvent)
		}),
	]
	return {
		dispose() {
			clearDrag?.()
			subscriptions.forEach((subscription) => subscription.dispose())
		},
	}
}
