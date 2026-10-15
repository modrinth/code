import type {
	DockviewApi,
	DockviewGroupPanel,
	DockviewPopoutGroupOptions,
	IDockviewPanel,
} from 'dockview-vue'

export type ReviewPopoutTarget = IDockviewPanel | DockviewGroupPanel

export function configureReviewPopout(child: Window) {
	const workspace = document.querySelector<HTMLElement>('.project-review')
	const tabHost = workspace?.querySelector<HTMLElement>('.project-review-tabs')
	const container = child.document.querySelector<HTMLElement>('.dv-popout-window')
	container?.classList.add('project-review-dock')
	if (workspace && tabHost && container) {
		const frame = workspace.cloneNode(false) as HTMLElement
		const tabs = tabHost.cloneNode(false) as HTMLElement
		frame.style.height = '100dvh'
		tabs.style.width = '100%'
		container.before(frame)
		frame.appendChild(tabs)
		tabs.appendChild(container)
	}

	function syncAppearance() {
		for (const [source, target] of [
			[document.documentElement, child.document.documentElement],
			[document.body, child.document.body],
		] as const) {
			for (const name of ['class', 'style', 'data-theme', 'lang', 'dir']) {
				const value = source.getAttribute(name)
				if (name === 'style' && source === document.body) {
					const targetStyle = (target as HTMLElement).style
					const overflow = targetStyle.overflow
					targetStyle.cssText = value ?? ''
					targetStyle.overflow = overflow
					continue
				}
				if (value === null) target.removeAttribute(name)
				else target.setAttribute(name, value)
			}
		}
	}

	syncAppearance()
	const observer = new MutationObserver(syncAppearance)
	for (const element of [document.documentElement, document.body]) {
		observer.observe(element, {
			attributes: true,
			attributeFilter: ['class', 'style', 'data-theme', 'lang', 'dir'],
		})
	}
	child.focus()
	return () => observer.disconnect()
}

/** Dockview opens through the main window; use the gesture's window to retain popup activation. */
export function openReviewPopout(
	api: DockviewApi,
	target: ReviewPopoutTarget,
	options: DockviewPopoutGroupOptions,
	sourceWindow = target.api.getWindow(),
) {
	if (sourceWindow === window) return api.addPopoutGroup(target, options)
	const originalOpen = window.open
	window.open = sourceWindow.open.bind(sourceWindow)
	try {
		return api.addPopoutGroup(target, options)
	} finally {
		window.open = originalOpen
	}
}
