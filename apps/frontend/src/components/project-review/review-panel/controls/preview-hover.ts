const INITIAL_HOVER_DELAY_MS = 750
const HOVER_GRACE_PERIOD_MS = 250

interface Preview {
	show: () => void
	hide: () => void
}

const documents = new WeakMap<Document, ReturnType<typeof createPreviewHover>>()

export function getPreviewHover(document: Document) {
	let state = documents.get(document)
	if (!state) {
		state = createPreviewHover()
		documents.set(document, state)
	}
	return state
}

function createPreviewHover() {
	let current: Preview | undefined
	let timer: ReturnType<typeof setTimeout> | undefined
	let visible = false
	let warmUntil = 0

	function leave(preview: Preview) {
		if (current !== preview) return
		clearTimeout(timer)
		timer = undefined
		if (visible) warmUntil = Date.now() + HOVER_GRACE_PERIOD_MS
		visible = false
		current = undefined
		preview.hide()
	}

	return {
		enter(preview: Preview) {
			if (current === preview) return
			if (current) leave(current)
			current = preview
			const show = () => {
				timer = undefined
				visible = true
				warmUntil = Infinity
				preview.show()
			}
			if (Date.now() < warmUntil) show()
			else timer = setTimeout(show, INITIAL_HOVER_DELAY_MS)
		},
		leave,
		dismiss(preview: Preview) {
			if (current !== preview) return
			leave(preview)
			warmUntil = 0
		},
	}
}
