import { useEventListener, useResizeObserver } from '@vueuse/core'
import { type Ref, watch } from 'vue'

import { injectReviewPanelVisible } from '../layout/context'

export function useTracesNotificationOffset(
	paneRef: Ref<HTMLElement | undefined>,
	enabled: () => boolean,
	containerRef: Ref<HTMLElement | undefined> = paneRef,
) {
	const panelVisible = injectReviewPanelVisible()

	watch(
		[paneRef, panelVisible, enabled],
		([pane, visible, active], _, onCleanup) => {
			if (!pane || !visible || !active || pane.ownerDocument !== document) return
			const body = pane.ownerDocument.body
			const updateOffset = () => {
				if (pane.ownerDocument !== document || !pane.getClientRects().length) {
					body.style.removeProperty('--project-review-traces-panel-offset')
					return
				}
				const viewport = pane.ownerDocument.defaultView
				if (!viewport) return
				const offset = `${Math.max(0, viewport.innerWidth - pane.getBoundingClientRect().left)}px`
				if (body.style.getPropertyValue('--project-review-traces-panel-offset') !== offset) {
					body.style.setProperty('--project-review-traces-panel-offset', offset)
				}
			}
			const { stop } = useResizeObserver([containerRef, paneRef], updateOffset)
			const stopResize = useEventListener(pane.ownerDocument.defaultView, 'resize', updateOffset)
			updateOffset()
			onCleanup(() => {
				stop()
				stopResize()
				body.style.removeProperty('--project-review-traces-panel-offset')
			})
		},
		{ flush: 'post' },
	)
}
