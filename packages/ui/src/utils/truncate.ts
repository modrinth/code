import type { Ref } from 'vue'
import { unref } from 'vue'

/**
 * Checks if an element's content is truncated (showing ellipsis).
 * Returns the tooltip text if truncated, undefined otherwise.
 *
 * @param element - HTMLElement, Ref<HTMLElement>, or null
 * @param tooltipText - Text to show in tooltip when truncated
 * @returns The tooltip text if element is truncated, undefined otherwise
 *
 * @example
 * ```vue
 * <span ref="titleRef" class="truncate" v-tooltip="truncatedTooltip(titleRef, project.title)">
 *   {{ project.title }}
 * </span>
 * ```
 */
export function truncatedTooltip(
	element: HTMLElement | Ref<HTMLElement | null> | null | undefined,
	tooltipText: string,
): string | undefined {
	const el = unref(element)
	if (!el) return undefined

	return el.scrollWidth > el.clientWidth ? tooltipText : undefined
}
