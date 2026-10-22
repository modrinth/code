import { onBeforeUnmount, type Ref, watch } from 'vue'

import { injectWorkbenchPanel } from '../providers/workbench'

/**
 * Lays out a grid or dock of a workbench nested in another's panel while that panel is popped
 * out. Dockview only follows the size of views within the main document, so in a popout window a
 * nested workbench would keep the size it first had; this follows it with the window's own
 * observer, and lays it out once more on its return.
 */
export function useDetachedLayout(
	element: Readonly<Ref<HTMLElement | null | undefined>>,
	layout: (width: number, height: number) => void,
) {
	const panel = injectWorkbenchPanel(null)
	let observer: ResizeObserver | undefined
	let detached = false
	let timer: ReturnType<typeof setTimeout> | undefined

	function observe() {
		observer?.disconnect()
		observer = undefined
		const target = element.value
		const view = target?.ownerDocument.defaultView
		if (!target || !view) return
		if (target.ownerDocument === document) {
			if (detached && target.clientWidth > 0 && target.clientHeight > 0) {
				layout(target.clientWidth, target.clientHeight)
			}
			detached = false
			return
		}
		detached = true
		const owner = target.ownerDocument
		observer = new view.ResizeObserver(([entry]) => {
			if (target.ownerDocument !== owner) {
				observe()
				return
			}
			const width = Math.round(entry.contentRect.width)
			const height = Math.round(entry.contentRect.height)
			if (width > 0 && height > 0) layout(width, height)
		})
		observer.observe(target)
	}

	if (panel) {
		watch(
			[element, panel.window],
			() => {
				clearTimeout(timer)
				timer = setTimeout(observe)
			},
			{ immediate: true, flush: 'post' },
		)
	}

	onBeforeUnmount(() => {
		clearTimeout(timer)
		observer?.disconnect()
	})
}
