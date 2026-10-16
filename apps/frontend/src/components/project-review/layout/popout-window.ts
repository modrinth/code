import type {
	DockviewApi,
	DockviewGroupPanel,
	DockviewPopoutGroupOptions,
	IDockviewPanel,
	SerializedDockview,
} from 'dockview-vue'

export type ReviewPopoutTarget = IDockviewPanel | DockviewGroupPanel
export type ReviewPopoutLayout = NonNullable<SerializedDockview['popoutGroups']>[number]

/** Dispose an unavailable panel while keeping its popout group available for returning tabs. */
export function removeReviewPopoutPanel(panel: IDockviewPanel) {
	const group = panel.group
	group.model.removePanel(panel)
	group.model.renderContainer.detatch(panel)
	panel.dispose()
}

/** Restore one saved window without replacing any live popouts. */
export async function restoreReviewPopout(api: DockviewApi, popout: ReviewPopoutLayout) {
	type GridNode = SerializedDockview['grid']['root']
	function dockedLayout(node: GridNode): GridNode | undefined {
		if (Array.isArray(node.data)) {
			const children = node.data.map(dockedLayout).filter((child): child is GridNode => !!child)
			return children.length ? { ...node, data: children } : undefined
		}
		const views = node.data.views.filter((id) => api.getPanel(id)?.api.location.type === 'grid')
		if (!views.length) return
		return {
			...node,
			data: {
				...node.data,
				views,
				activeView: views.includes(node.data.activeView ?? '') ? node.data.activeView : views[0],
			},
		}
	}
	function firstPanel(node: GridNode): IDockviewPanel | undefined {
		if (!Array.isArray(node.data)) return api.getPanel(node.data.views[0])
		return node.data.length ? firstPanel(node.data[0]) : undefined
	}
	function prepareGroups(node: GridNode) {
		if (Array.isArray(node.data)) {
			for (const child of node.data) prepareGroups(child)
			return
		}
		const panels = node.data.views
			.map((id) => api.getPanel(id))
			.filter((panel): panel is IDockviewPanel => !!panel && panel.api.location.type === 'grid')
		if (!panels.length) return
		const existing = panels[0].group
		const group =
			existing.panels.length === panels.length && panels.every((panel) => panel.group === existing)
				? existing
				: api.addGroup()
		for (const panel of panels) {
			if (panel.group !== group) panel.api.moveTo({ group, position: 'center' })
		}
		if (node.data.activeView) api.getPanel(node.data.activeView)?.api.setActive()
	}
	function restoreGrid(node: GridNode, horizontal: boolean) {
		if (!Array.isArray(node.data)) return
		const children = node.data
		for (let index = 1; index < children.length; index++) {
			const group = firstPanel(children[index])?.group
			const reference = firstPanel(children[index - 1])?.group
			if (group && reference)
				group.api.moveTo({ group: reference, position: horizontal ? 'right' : 'bottom' })
		}
		for (const child of children) restoreGrid(child, !horizontal)
	}
	function restoreSizes(node: GridNode, horizontal: boolean, width: number, height: number) {
		if (!Array.isArray(node.data)) {
			firstPanel(node)?.group.api.setSize({ width, height })
			return
		}
		const total = node.data.reduce((sum, child) => sum + (child.size ?? 1), 0)
		for (const child of node.data) {
			const ratio = (child.size ?? 1) / total
			restoreSizes(
				child,
				!horizontal,
				horizontal ? width * ratio : width,
				horizontal ? height : height * ratio,
			)
		}
	}
	try {
		const root = dockedLayout(popout.grid?.root ?? { type: 'leaf', data: popout.data! })
		if (!root) return true
		const panel = firstPanel(root)
		if (!panel) return false
		prepareGroups(root)
		const opened = await api.addPopoutGroup(panel.group, {
			position: popout.position ?? undefined,
			popoutUrl: popout.url,
		})
		if (opened && popout.grid) {
			const horizontal = popout.grid.orientation === 'HORIZONTAL'
			restoreGrid(root, horizontal)
			const child = panel.api.getWindow()
			restoreSizes(root, horizontal, child.innerWidth, child.innerHeight)
		}
		return opened
	} catch {
		return false
	}
}

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
