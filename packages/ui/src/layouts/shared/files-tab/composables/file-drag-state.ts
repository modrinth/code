import { ref } from 'vue'

export interface FileDragData {
	name: string
	type: string
	path: string
}

const activeDrag = ref<FileDragData | null>(null)
const dragTarget = ref<string | null>(null)
const ghostEl = ref<HTMLElement | null>(null)
const pointerStartX = ref(0)
const pointerStartY = ref(0)
const dragStarted = ref(false)

const DRAG_THRESHOLD = 5

export const fileDragData = activeDrag
export const fileDragTarget = dragTarget
export const fileDragActive = dragStarted

function createGhost(name: string): HTMLElement {
	const el = document.createElement('div')
	el.className =
		'fixed z-[99999] flex items-center max-w-[500px] gap-3 rounded-lg bg-bg-raised p-3 shadow-lg pointer-events-none text-contrast font-bold truncate'
	el.textContent = name
	el.style.transform = 'translate(-50%, -100%)'
	document.body.appendChild(el)
	return el
}

function findDropTarget(x: number, y: number): string | null {
	const el = document.elementFromPoint(x, y)
	if (!el) return null
	const row = (el as HTMLElement).closest('[data-file-type="directory"]') as HTMLElement | null
	return row?.dataset.filePath ?? null
}

function onPointerMove(e: PointerEvent) {
	if (!activeDrag.value) return

	if (!dragStarted.value) {
		const dx = e.clientX - pointerStartX.value
		const dy = e.clientY - pointerStartY.value
		if (Math.abs(dx) < DRAG_THRESHOLD && Math.abs(dy) < DRAG_THRESHOLD) return
		dragStarted.value = true
		ghostEl.value = createGhost(activeDrag.value.name)
	}

	if (ghostEl.value) {
		ghostEl.value.style.left = `${e.clientX}px`
		ghostEl.value.style.top = `${e.clientY - 10}px`
	}

	const target = findDropTarget(e.clientX, e.clientY)
	if (target !== dragTarget.value) {
		dragTarget.value = target
	}
}

let clickSuppressed = false

export function wasRecentDrag(): boolean {
	return clickSuppressed
}

function cleanup() {
	const wasDrag = dragStarted.value
	if (ghostEl.value) {
		ghostEl.value.remove()
		ghostEl.value = null
	}
	activeDrag.value = null
	dragTarget.value = null
	dragStarted.value = false
	document.removeEventListener('pointermove', onPointerMove)
	document.removeEventListener('pointerup', onPointerUp)
	if (wasDrag) {
		clickSuppressed = true
		requestAnimationFrame(() => {
			clickSuppressed = false
		})
	}
}

let onDropCallback: ((source: FileDragData, destination: string) => void) | null = null

function onPointerUp() {
	if (dragStarted.value && activeDrag.value && dragTarget.value) {
		const src = activeDrag.value
		const dest = dragTarget.value
		const isSelf = dest === src.path
		const isChild = src.type === 'directory' && dest.startsWith(src.path + '/')
		if (!isSelf && !isChild) {
			onDropCallback?.(src, dest)
		}
	}
	cleanup()
}

export function startFileDrag(
	data: FileDragData,
	e: PointerEvent,
	onDrop: (source: FileDragData, destination: string) => void,
) {
	activeDrag.value = data
	pointerStartX.value = e.clientX
	pointerStartY.value = e.clientY
	dragStarted.value = false
	onDropCallback = onDrop

	document.addEventListener('pointermove', onPointerMove)
	document.addEventListener('pointerup', onPointerUp)
}
