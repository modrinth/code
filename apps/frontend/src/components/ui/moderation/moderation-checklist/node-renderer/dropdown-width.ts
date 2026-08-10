const DROPDOWN_TRIGGER_CHROME_PX = 16 * 2 + 10 + 20 + 2
const dropdownMinWidthCache = new Map<string, string>()
let measureElement: HTMLSpanElement | null = null

function measureLabelWidth(label: string): number {
	if (typeof document === 'undefined') return 0
	if (!measureElement) {
		measureElement = document.createElement('span')
		measureElement.className = 'min-w-0 truncate text-primary font-semibold leading-tight'
		Object.assign(measureElement.style, {
			position: 'absolute',
			visibility: 'hidden',
			whiteSpace: 'nowrap',
			left: '-9999px',
			top: '0',
		})
		document.body.appendChild(measureElement)
	}
	measureElement.textContent = label
	return measureElement.getBoundingClientRect().width
}

export function getDropdownMinWidth(options: { label: string }[]): string {
	const key = options.map((option) => option.label).join(' ')
	const cached = dropdownMinWidthCache.get(key)
	if (cached) return cached
	const maxLabelWidth = Math.max(0, ...options.map((option) => measureLabelWidth(option.label)))
	const result = `${Math.ceil(maxLabelWidth) + DROPDOWN_TRIGGER_CHROME_PX}px`
	dropdownMinWidthCache.set(key, result)
	return result
}
