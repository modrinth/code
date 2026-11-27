import { StorageSerializers, useResizeObserver, useStorage } from '@vueuse/core'
import { computed, type MaybeRefOrGetter, onScopeDispose, type Ref, ref, toValue } from 'vue'

const ROW_DRAG_DISTANCE = 92
const OVERDRAG_FLASH_MS = 500
const RESIZING_BODY_CLASS = 'recent-worlds-resizing'

export function useJumpBackInResize({
	grid,
	itemCount,
	maxItems,
}: {
	grid: Ref<HTMLElement | undefined>
	itemCount: MaybeRefOrGetter<number>
	maxItems: number
}) {
	const columnCount = ref(1)
	useResizeObserver(grid, () => {
		if (!grid.value) return
		columnCount.value = getComputedStyle(grid.value).gridTemplateColumns.split(' ').length
	})

	const storedLimit = useStorage<number | null>('modrinth-jump-back-in-count', null, localStorage, {
		serializer: StorageSerializers.number,
	})
	const visibleItemLimit = computed(() =>
		storedLimit.value !== null && Number.isFinite(storedLimit.value)
			? Math.max(1, Math.min(storedLimit.value, maxItems))
			: maxItems,
	)
	const maxVisibleItems = computed(() => Math.min(toValue(itemCount), maxItems))
	const minVisibleItems = computed(() => Math.min(columnCount.value, maxVisibleItems.value))
	const visibleItemCount = computed(() =>
		Math.max(minVisibleItems.value, Math.min(visibleItemLimit.value, maxVisibleItems.value)),
	)
	const canResize = computed(() => maxVisibleItems.value > minVisibleItems.value)
	const resizing = ref(false)
	const showOverdrag = ref(false)

	let dragStartY = 0
	let dragStartCount = 0
	let wasOverdragging = false
	let overdragTimeout: ReturnType<typeof setTimeout> | undefined

	function setVisibleItemLimit(count: number) {
		const clamped = Math.max(minVisibleItems.value, Math.min(count, maxVisibleItems.value))
		storedLimit.value = clamped >= maxVisibleItems.value ? null : clamped
	}

	function getRowResizeTarget(count: number, rowDelta: number) {
		return rowDelta < 0
			? (Math.ceil(count / columnCount.value) + rowDelta) * columnCount.value
			: count + rowDelta * columnCount.value
	}

	function clearOverdragFlash() {
		showOverdrag.value = false
		clearTimeout(overdragTimeout)
		overdragTimeout = undefined
	}

	function flashOverdrag() {
		showOverdrag.value = true
		clearTimeout(overdragTimeout)
		overdragTimeout = setTimeout(() => {
			showOverdrag.value = false
			overdragTimeout = undefined
		}, OVERDRAG_FLASH_MS)
	}

	function isOutOfRange(target: number) {
		return target < minVisibleItems.value || target > maxVisibleItems.value
	}

	function adjustVisibleItemLimit(rowDelta: number) {
		const target = getRowResizeTarget(visibleItemCount.value, rowDelta)
		if (isOutOfRange(target)) flashOverdrag()
		setVisibleItemLimit(target)
	}

	function onResizePointerDown(event: PointerEvent) {
		if (!canResize.value) return
		event.preventDefault()
		resizing.value = true
		wasOverdragging = false
		clearOverdragFlash()
		dragStartY = event.clientY
		dragStartCount = visibleItemCount.value
		document.body.classList.add(RESIZING_BODY_CLASS)
		;(event.currentTarget as HTMLElement).setPointerCapture(event.pointerId)
	}

	function onResizePointerMove(event: PointerEvent) {
		if (!resizing.value) return
		const rowDelta = Math.round((event.clientY - dragStartY) / ROW_DRAG_DISTANCE)
		const target = getRowResizeTarget(dragStartCount, rowDelta)
		const isOverdragging = isOutOfRange(target)
		if (isOverdragging && !wasOverdragging) flashOverdrag()
		wasOverdragging = isOverdragging
		setVisibleItemLimit(target)
	}

	function endResize(event?: PointerEvent) {
		if (!resizing.value) return
		resizing.value = false
		wasOverdragging = false
		clearOverdragFlash()
		document.body.classList.remove(RESIZING_BODY_CLASS)
		const target = event?.currentTarget as HTMLElement | undefined
		if (event && target?.hasPointerCapture(event.pointerId)) {
			target.releasePointerCapture(event.pointerId)
		}
	}

	onScopeDispose(() => {
		document.body.classList.remove(RESIZING_BODY_CLASS)
		clearOverdragFlash()
	})

	return {
		minVisibleItems,
		maxVisibleItems,
		visibleItemCount,
		canResize,
		resizing,
		showOverdrag,
		setVisibleItemLimit,
		adjustVisibleItemLimit,
		onResizePointerDown,
		onResizePointerMove,
		endResize,
	}
}
