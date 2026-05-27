import { ref, type ComputedRef, type Ref } from 'vue'

type MaybeReadonlyRef<T> = Ref<T> | ComputedRef<T>

export function useSkinPreviewControls({
	initialRotation,
	onClickWithoutDrag,
}: {
	initialRotation: MaybeReadonlyRef<number | undefined>
	onClickWithoutDrag: () => void
}) {
	const modelRotation = ref((initialRotation.value ?? 15.75) + Math.PI)
	const isDragging = ref(false)
	const previousX = ref(0)
	const hasDragged = ref(false)

	function onPointerDown(event: PointerEvent) {
		;(event.currentTarget as HTMLElement).setPointerCapture(event.pointerId)
		isDragging.value = true
		previousX.value = event.clientX
		hasDragged.value = false
	}

	function onPointerMove(event: PointerEvent) {
		if (!isDragging.value) return
		const deltaX = event.clientX - previousX.value
		modelRotation.value += deltaX * 0.01
		previousX.value = event.clientX
		hasDragged.value = true
	}

	function onPointerUp(event: PointerEvent) {
		isDragging.value = false

		const target = event.currentTarget as HTMLElement
		if (target.hasPointerCapture(event.pointerId)) {
			target.releasePointerCapture(event.pointerId)
		}
	}

	function onCanvasClick() {
		if (!hasDragged.value) {
			onClickWithoutDrag()
		}

		hasDragged.value = false
	}

	function ignoreControlClick(event: MouseEvent) {
		event.stopPropagation()
	}

	return {
		ignoreControlClick,
		modelRotation,
		onCanvasClick,
		onPointerDown,
		onPointerMove,
		onPointerUp,
	}
}
