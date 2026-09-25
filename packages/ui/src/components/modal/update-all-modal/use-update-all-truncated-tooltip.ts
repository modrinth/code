import { useResizeObserver } from '@vueuse/core'
import { nextTick, ref, watch } from 'vue'

import { truncatedTooltip } from '#ui/utils/truncate'

export function useUpdateAllTruncatedTooltip(text: () => string, enabled = () => true) {
	const element = ref<HTMLElement | null>(null)
	const tooltip = ref<string>()

	function measure() {
		tooltip.value = enabled() ? truncatedTooltip(element, text()) : undefined
	}

	useResizeObserver(element, measure)
	watch([element, text, enabled], () => nextTick(measure), { immediate: true, flush: 'post' })

	return { element, tooltip }
}
