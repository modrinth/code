import { createTextVNode, createVNode, Fragment, isVNode, toDisplayString, type VNode } from 'vue'

/**
 * Checks whether a specific child is a VNode. If not, converts it to a display
 * string and then creates text VNode for the result.
 *
 * @param child Child to normalize.
 * @returns Either the original VNode or a text VNode containing child converted
 * to a display string.
 */
function normalizeChild(child: unknown): VNode {
	return isVNode(child) ? child : createTextVNode(toDisplayString(child))
}

/**
 * Takes in an array of VNodes and other children. It then converts each child
 * that is not already a VNode to a display string, and creates a text VNode for
 * that string.
 *
 * @param children Children to normalize.
 * @returns A single VNode (or Fragment VNode when there are multiple children).
 */
export function normalizeChildren(children: unknown | unknown[]): VNode {
	const normalized = Array.isArray(children)
		? children.map(normalizeChild)
		: [normalizeChild(children)]

	if (normalized.length === 0) {
		return createTextVNode('')
	}

	if (normalized.length === 1) {
		return normalized[0]!
	}

	return createVNode(Fragment, null, normalized)
}
