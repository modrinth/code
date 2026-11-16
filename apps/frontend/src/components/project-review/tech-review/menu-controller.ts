type TechReviewMenuOptions = {
	document: Document
	trigger: Node | null
	getPanel: () => Node | null
	hide: () => void
	onEscape: () => void
}

type MenuEntry = TechReviewMenuOptions & { parent?: MenuEntry }

type MenuState = {
	entries: MenuEntry[]
	dispose: () => void
}

export function createTechReviewMenuController() {
	const documentMenus = new WeakMap<Document, MenuState>()
	const menuStates = new Set<MenuState>()

	function ancestors(entry: MenuEntry): MenuEntry[] {
		return [entry, ...(entry.parent ? ancestors(entry.parent) : [])]
	}

	function getState(ownerDocument: Document): MenuState {
		const existing = documentMenus.get(ownerDocument)
		if (existing) return existing

		function onPointerDown(event: PointerEvent) {
			const target = event.target as Node | null
			if (!target) return
			const retained = new Set(
				state.entries.flatMap((entry) =>
					entry.trigger?.contains(target) || entry.getPanel()?.contains(target)
						? ancestors(entry)
						: [],
				),
			)
			for (const entry of [...state.entries].reverse()) {
				if (!retained.has(entry)) entry.hide()
			}
		}

		function onKeydown(event: KeyboardEvent) {
			if (event.key !== 'Escape') return
			const topmost = state.entries[state.entries.length - 1]
			if (!topmost) return
			event.preventDefault()
			event.stopImmediatePropagation()
			topmost.onEscape()
		}

		const state: MenuState = {
			entries: [],
			dispose: () => {
				if (documentMenus.get(ownerDocument) !== state) return
				ownerDocument.removeEventListener('pointerdown', onPointerDown, true)
				ownerDocument.removeEventListener('keydown', onKeydown, true)
				documentMenus.delete(ownerDocument)
				menuStates.delete(state)
			},
		}
		ownerDocument.addEventListener('pointerdown', onPointerDown, true)
		ownerDocument.addEventListener('keydown', onKeydown, true)
		documentMenus.set(ownerDocument, state)
		menuStates.add(state)
		return state
	}

	function registerMenu(options: TechReviewMenuOptions) {
		const state = getState(options.document)
		const parent = [...state.entries]
			.reverse()
			.find((entry) => options.trigger && entry.getPanel()?.contains(options.trigger))
		const entry: MenuEntry = { ...options, parent }
		const retained = new Set(ancestors(entry))
		state.entries.push(entry)
		for (const other of [...state.entries].reverse()) {
			if (!retained.has(other)) other.hide()
		}
		let registered = true

		return {
			zIndex: 100000 + ancestors(entry).length - 1,
			unregister: () => {
				if (!registered) return
				registered = false
				const descendants = state.entries.filter((other) => ancestors(other).includes(entry))
				state.entries = state.entries.filter((other) => other !== entry)
				for (const child of descendants.reverse()) {
					if (child !== entry) child.hide()
				}
				if (!state.entries.length) state.dispose()
			},
		}
	}

	function dismissMenus(ownerDocument?: Document) {
		const states = ownerDocument ? [documentMenus.get(ownerDocument)] : [...menuStates]
		for (const state of states) {
			for (const entry of [...(state?.entries ?? [])].reverse()) entry.hide()
		}
	}

	return { register: registerMenu, dismiss: dismissMenus }
}
