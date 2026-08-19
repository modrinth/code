import type {
	ContextMenuAction,
	ContextMenuDivider,
	ContextMenuOption,
	ContextMenuParentAction,
	Point,
} from './types'

export function isAction(option: ContextMenuOption): option is ContextMenuAction {
	return 'name' in option
}

export function isDivider(option: ContextMenuOption): option is ContextMenuDivider {
	return 'type' in option && option.type === 'divider'
}

export function hasChildren(option: ContextMenuOption): option is ContextMenuParentAction {
	return isAction(option) && Boolean(option.children?.length)
}

export function isInstanceLink(value: unknown) {
	if (!value || typeof value !== 'object') return false

	if ('instance' in value) {
		const instance = value.instance
		return Boolean(instance && typeof instance === 'object' && 'link' in instance && instance.link)
	}

	return 'link' in value && Boolean(value.link)
}

export function getFocusableButtons(buttonRefs: Map<number, HTMLElement>) {
	return [...buttonRefs.entries()]
		.sort(([left], [right]) => left - right)
		.map(([, button]) => button)
		.filter((button) => button.offsetParent !== null)
}

export function focusRelativeButton(buttons: HTMLElement[], direction: 1 | -1) {
	if (!buttons.length) return

	const currentIndex = buttons.indexOf(document.activeElement as HTMLElement)
	const nextIndex =
		currentIndex === -1 ? (direction === 1 ? 0 : buttons.length - 1) : currentIndex + direction
	buttons[(nextIndex + buttons.length) % buttons.length]?.focus()
}

export function isPointInTriangle(point: Point, a: Point, b: Point, c: Point) {
	const area = triangleArea(a, b, c)
	const area1 = triangleArea(point, b, c)
	const area2 = triangleArea(a, point, c)
	const area3 = triangleArea(a, b, point)

	return Math.abs(area - (area1 + area2 + area3)) < 0.5
}

function triangleArea(a: Point, b: Point, c: Point) {
	return Math.abs((a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y)) / 2)
}
