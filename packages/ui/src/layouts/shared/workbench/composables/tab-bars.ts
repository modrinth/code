import type { DockviewApi, DockviewGroupPanel, DockviewIDisposable } from 'dockview-vue'

const CHEVRON_DOWN =
	'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m6 9 6 6 6-6"/></svg>'

export interface TabBarsOptions {
	/** Whether groups holding a single tab hide their tab bar at the moment. */
	hideLone: () => boolean
	/** Whether the user hid the group's tab bar. */
	hiddenByUser: (group: DockviewGroupPanel) => boolean
	/** Shows a tab bar the user hid. */
	showHidden: (group: DockviewGroupPanel) => void
	/** The handle's label, while its tab bar is hidden or pulled down. */
	label: (hidden: boolean) => string
}

/**
 * Hides the tab bars of a dock's groups (in its popout windows too) that the user hid, and, as the
 * settings choose, of those holding a single tab, leaving the tab the group's whole height. A
 * handle at the group's top right pulls the tab bar down (e.g. to reach its menus, or to drag
 * another tab in), and folds a lone tab's bar away again.
 */
export function registerTabBars(
	api: DockviewApi,
	options: TabBarsOptions,
): DockviewIDisposable & { update: () => void } {
	const handles = new Map<DockviewGroupPanel, HTMLButtonElement>()
	const pulledDown = new WeakSet<DockviewGroupPanel>()
	let scheduled = false

	function tabBarOf(group: DockviewGroupPanel) {
		return group.element.querySelector<HTMLElement>(':scope > .dv-tabs-and-actions-container')
	}

	function isLone(group: DockviewGroupPanel) {
		return options.hideLone() && group.panels.length === 1
	}

	function toggle(group: DockviewGroupPanel) {
		if (options.hiddenByUser(group)) {
			options.showHidden(group)
			pulledDown.add(group)
		} else if (pulledDown.has(group)) pulledDown.delete(group)
		else pulledDown.add(group)
		update()
	}

	function handleFor(group: DockviewGroupPanel) {
		const existing = handles.get(group)
		if (existing) return existing
		const handle = group.element.ownerDocument.createElement('button')
		handle.type = 'button'
		handle.className = 'workbench-tab-bar-handle'
		handle.innerHTML = CHEVRON_DOWN
		handle.addEventListener('click', (event) => {
			event.stopPropagation()
			toggle(group)
		})
		handle.addEventListener('pointerdown', (event) => event.stopPropagation())
		group.element.appendChild(handle)
		handles.set(group, handle)
		return handle
	}

	function removeHandle(group: DockviewGroupPanel) {
		handles.get(group)?.remove()
		handles.delete(group)
	}

	function update() {
		scheduled = false
		for (const group of handles.keys()) {
			if (!api.groups.includes(group)) removeHandle(group)
		}
		for (const group of api.groups) {
			const occupied = group.panels.length > 0
			const lone = occupied && isLone(group)
			const hidden = occupied && (options.hiddenByUser(group) || (lone && !pulledDown.has(group)))
			if (group.header.hidden !== hidden) {
				group.header.hidden = hidden
				group.relayout()
			}
			if (!hidden && !lone) {
				removeHandle(group)
				continue
			}
			const handle = handleFor(group)
			if (handle.parentElement !== group.element) group.element.appendChild(handle)
			const tabBar = tabBarOf(group)
			handle.style.top = hidden || !tabBar ? '' : `${tabBar.offsetHeight}px`
			handle.classList.toggle('is-pulled-down', !hidden)
			handle.setAttribute('aria-expanded', String(!hidden))
			const label = options.label(hidden)
			handle.setAttribute('aria-label', label)
			handle.title = label
		}
	}

	/** Waits for a move to settle, as moving a group's tabs briefly empties or fills it. */
	function schedule() {
		if (scheduled) return
		scheduled = true
		queueMicrotask(update)
	}

	const subscriptions = [
		api.onDidLayoutChange(schedule),
		api.onDidAddPanel(schedule),
		api.onDidRemovePanel(schedule),
		api.onDidAddGroup(schedule),
		api.onDidRemoveGroup(schedule),
		api.onDidAddPopoutGroup(schedule),
	]
	schedule()

	return {
		update,
		dispose() {
			for (const subscription of subscriptions) subscription.dispose()
			for (const [group, handle] of handles) {
				handle.remove()
				if (group.header.hidden) {
					group.header.hidden = false
					group.relayout()
				}
			}
			handles.clear()
		},
	}
}
