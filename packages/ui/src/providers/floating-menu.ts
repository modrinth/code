const hideCallbacks = new Set<() => void>()

export function registerFloatingMenu(hide: () => void): () => void {
	hideCallbacks.add(hide)
	return () => {
		hideCallbacks.delete(hide)
	}
}

export function dismissFloatingMenus() {
	for (const hide of [...hideCallbacks]) {
		hide()
	}
}
