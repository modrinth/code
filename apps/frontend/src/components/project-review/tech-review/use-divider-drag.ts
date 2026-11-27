import { nextTick, onBeforeUnmount, ref } from 'vue'

interface DividerDragOptions {
	axis: 'x' | 'y'
	direction: 1 | -1
	size: number
	min: number
	max: number
	container: HTMLElement
	property: string
	commit: (size: number) => void
}

export function useDividerDrag() {
	const dragging = ref(false)
	let active:
		| (DividerDragOptions & {
				pointerId: number
				origin: number
				value: number
				previewed?: number
				view: Window
				frame?: number
		  })
		| undefined

	function start(event: PointerEvent, options: DividerDragOptions) {
		if (active) return
		const handle = event.currentTarget as HTMLElement
		const view = handle.ownerDocument.defaultView
		if (!view) return
		const size = Math.round(options.size)
		active = {
			...options,
			size,
			pointerId: event.pointerId,
			origin: options.axis === 'x' ? event.clientX : event.clientY,
			value: size,
			view,
		}
		event.preventDefault()
		handle.setPointerCapture(event.pointerId)
		dragging.value = true
	}

	function move(event: PointerEvent) {
		const drag = active
		if (!drag || drag.pointerId !== event.pointerId) return
		const position = drag.axis === 'x' ? event.clientX : event.clientY
		const requested = drag.size + (position - drag.origin) * drag.direction
		drag.value = Math.round(Math.max(drag.min, Math.min(drag.max, requested)))
		if (drag.frame !== undefined || drag.value === (drag.previewed ?? drag.size)) return
		drag.frame = drag.view.requestAnimationFrame(() => {
			drag.frame = undefined
			if (drag.value === (drag.previewed ?? drag.size)) return
			drag.container.style.setProperty(drag.property, `${drag.value}px`)
			drag.previewed = drag.value
		})
	}

	function stop(event: PointerEvent) {
		const drag = active
		if (!drag || drag.pointerId !== event.pointerId) return
		active = undefined
		dragging.value = false
		if (drag.frame !== undefined) drag.view.cancelAnimationFrame(drag.frame)
		if (drag.value !== drag.size) {
			drag.container.style.setProperty(drag.property, `${drag.value}px`)
			drag.commit(drag.value)
		}
		void nextTick(() => drag.container.style.removeProperty(drag.property))
		const handle = event.currentTarget as HTMLElement
		if (handle.hasPointerCapture(event.pointerId)) handle.releasePointerCapture(event.pointerId)
	}

	onBeforeUnmount(() => {
		if (active?.frame !== undefined) active.view.cancelAnimationFrame(active.frame)
		if (active) active.container.style.removeProperty(active.property)
		active = undefined
	})

	return { dragging, start, move, stop }
}
