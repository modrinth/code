import type {
	DockviewApi,
	DockviewGroupPanel,
	DockviewIDisposable,
	DockviewPopoutGroupOptions,
} from 'dockview-vue'

import type { ReviewPopoutTarget } from './popout-window'

type WindowOrigin = { left: number; top: number; width: number; height: number }
type DropTarget = { element: Element; clientX: number; clientY: number }

export function registerDragPopouts(
	api: DockviewApi,
	popout: (
		target: ReviewPopoutTarget,
		position: NonNullable<DockviewPopoutGroupOptions['position']>,
		sourceWindow: Window,
	) => void,
): DockviewIDisposable {
	const windowCleanups = new Map<Window, () => void>()
	const origins = new Map<Window, WindowOrigin>()
	const windowOrder = new Map<Window, number>()
	let focusSequence = 0
	let clearDrag: (() => void) | undefined
	let cancelDrag: (() => void) | undefined

	function trackWindows() {
		const windows = new Set([window, ...api.getPopouts().map((entry) => entry.window)])
		for (const [child, cleanup] of windowCleanups) {
			if (windows.has(child)) continue
			cleanup()
			windowCleanups.delete(child)
			origins.delete(child)
			windowOrder.delete(child)
		}
		for (const child of windows) {
			if (windowCleanups.has(child)) continue
			function recordOrigin(event: PointerEvent) {
				origins.set(child, {
					left: event.screenX - event.clientX - child.screenX,
					top: event.screenY - event.clientY - child.screenY,
					width: child.innerWidth,
					height: child.innerHeight,
				})
			}
			function onFocus() {
				windowOrder.set(child, ++focusSequence)
			}
			function onKeydown(event: KeyboardEvent) {
				if (event.key !== 'Escape' || !cancelDrag) return
				event.preventDefault()
				event.stopImmediatePropagation()
				cancelDrag()
			}
			child.addEventListener('pointermove', recordOrigin, true)
			child.addEventListener('pointerdown', recordOrigin, true)
			child.addEventListener('focus', onFocus)
			child.addEventListener('keydown', onKeydown, true)
			windowCleanups.set(child, () => {
				child.removeEventListener('pointermove', recordOrigin, true)
				child.removeEventListener('pointerdown', recordOrigin, true)
				child.removeEventListener('focus', onFocus)
				child.removeEventListener('keydown', onKeydown, true)
			})
		}
	}

	function windowPoint(child: Window, event: PointerEvent) {
		const origin = origins.get(child)
		const border = Math.max(0, (child.outerWidth - child.innerWidth) / 2)
		const calibrated = origin?.width === child.innerWidth && origin.height === child.innerHeight
		return {
			clientX: event.screenX - child.screenX - (calibrated ? origin.left : border),
			clientY:
				event.screenY -
				child.screenY -
				(calibrated ? origin.top : Math.max(0, child.outerHeight - child.innerHeight - border)),
		}
	}

	function findDropTarget(sourceWindow: Window, event: PointerEvent): DropTarget | undefined {
		const windows = [...windowCleanups.keys()].sort(
			(a, b) => (windowOrder.get(b) ?? 0) - (windowOrder.get(a) ?? 0),
		)
		for (const child of windows) {
			if (child.closed) continue
			const point = child === sourceWindow ? event : windowPoint(child, event)
			if (
				point.clientX < 0 ||
				point.clientY < 0 ||
				point.clientX >= child.innerWidth ||
				point.clientY >= child.innerHeight
			)
				continue
			if (child === sourceWindow) return
			const element = child.document.elementFromPoint(point.clientX, point.clientY)
			if (!element?.closest('.project-review-tabs, .dv-popout-window')) return
			return { element, clientX: point.clientX, clientY: point.clientY }
		}
	}

	function onDragStart(
		target: ReviewPopoutTarget,
		group: DockviewGroupPanel,
		nativeEvent: DragEvent | PointerEvent,
	) {
		cancelDrag?.()
		if (!('pointerId' in nativeEvent)) return
		const source = nativeEvent.target as HTMLElement | null
		const ownerWindow = source?.ownerDocument.defaultView
		const dock = group.element.closest('.dv-popout-window, .project-review-tabs')
		if (!source || !ownerWindow || !dock) return
		const sourceWindow = ownerWindow
		const captureElement = source.ownerDocument.documentElement
		const pointerId = nativeEvent.pointerId
		const bounds = dock.getBoundingClientRect()
		const panelIds = new Set(
			'panels' in target ? target.panels.map((panel) => panel.id) : [target.id],
		)
		let remoteTarget: DropTarget | undefined
		let lastEvent = nativeEvent
		let moved = false
		let dropped = false
		const movedSubscription = api.onDidMovePanel((event) => {
			if (panelIds.has(event.panel.id)) moved = true
		})
		const dropSubscription = api.onDidDrop(() => {
			dropped = true
		})
		windowOrder.set(sourceWindow, ++focusSequence)

		/** Reuse Dockview's drop targets and shared panel transfer across documents. */
		function dispatchDropEvent(type: string, destination: DropTarget, event: PointerEvent) {
			destination.element.dispatchEvent(
				new DragEvent(type, {
					bubbles: true,
					cancelable: true,
					view: destination.element.ownerDocument.defaultView,
					clientX: destination.clientX,
					clientY: destination.clientY,
					screenX: event.screenX,
					screenY: event.screenY,
					ctrlKey: event.ctrlKey,
					shiftKey: event.shiftKey,
					altKey: event.altKey,
					metaKey: event.metaKey,
				}),
			)
		}

		function clearRemoteTarget() {
			if (remoteTarget) dispatchDropEvent('dragleave', remoteTarget, lastEvent)
			remoteTarget = undefined
		}

		function updateRemoteTarget(event: PointerEvent) {
			const destination = findDropTarget(sourceWindow, event)
			if (destination?.element !== remoteTarget?.element) {
				clearRemoteTarget()
				if (destination) dispatchDropEvent('dragenter', destination, event)
			}
			remoteTarget = destination
			if (destination) dispatchDropEvent('dragover', destination, event)
		}

		function cancelDockview() {
			sourceWindow.dispatchEvent(new PointerEvent('pointercancel', { pointerId }))
		}

		function onPointerMove(event: PointerEvent) {
			if (event.pointerId !== pointerId) return
			lastEvent = event
			updateRemoteTarget(event)
		}

		function onPointerUp(event: PointerEvent) {
			if (event.pointerId !== pointerId || !clearDrag) return
			lastEvent = event
			updateRemoteTarget(event)
			const outside =
				event.clientX < bounds.left ||
				event.clientX > bounds.right ||
				event.clientY < bounds.top ||
				event.clientY > bounds.bottom
			if (!remoteTarget && !outside) {
				queueMicrotask(() => clearDrag?.())
				return
			}
			event.stopImmediatePropagation()
			if (remoteTarget) dispatchDropEvent('drop', remoteTarget, event)
			const shouldPopout = !moved && !dropped
			clearDrag?.()
			cancelDockview()
			if (shouldPopout) {
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
		}

		function onPointerCancel(event: PointerEvent) {
			if (event.pointerId === pointerId) cancelDrag?.()
		}

		function onWindowClose() {
			cancelDrag?.()
		}

		clearDrag = () => {
			clearDrag = undefined
			cancelDrag = undefined
			sourceWindow.removeEventListener('pointermove', onPointerMove, true)
			sourceWindow.removeEventListener('pointerup', onPointerUp, true)
			sourceWindow.removeEventListener('pointercancel', onPointerCancel, true)
			sourceWindow.removeEventListener('lostpointercapture', onPointerCancel, true)
			sourceWindow.removeEventListener('beforeunload', onWindowClose)
			clearRemoteTarget()
			movedSubscription.dispose()
			dropSubscription.dispose()
			if (captureElement.hasPointerCapture(pointerId))
				captureElement.releasePointerCapture(pointerId)
		}
		cancelDrag = () => {
			clearDrag?.()
			cancelDockview()
		}
		sourceWindow.addEventListener('pointermove', onPointerMove, true)
		sourceWindow.addEventListener('pointerup', onPointerUp, true)
		sourceWindow.addEventListener('pointercancel', onPointerCancel, true)
		sourceWindow.addEventListener('lostpointercapture', onPointerCancel, true)
		sourceWindow.addEventListener('beforeunload', onWindowClose)
		captureElement.setPointerCapture(pointerId)
	}

	trackWindows()
	const subscriptions = [
		api.onDidAddPopoutGroup(trackWindows),
		api.onDidRemovePopoutGroup(trackWindows),
		api.onWillDragPanel(({ panel, nativeEvent }) => {
			onDragStart(panel, panel.group, nativeEvent)
		}),
		api.onWillDragGroup(({ group, nativeEvent }) => {
			onDragStart(group, group, nativeEvent)
		}),
	]
	return {
		dispose() {
			cancelDrag?.()
			windowCleanups.forEach((cleanup) => cleanup())
			subscriptions.forEach((subscription) => subscription.dispose())
		},
	}
}
