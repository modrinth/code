import type { Ref } from 'vue'
import { onBeforeUnmount, ref } from 'vue'

import type { ContentCardTableItem } from '../../types'

type Point = { x: number; y: number }

export const ROW_GATHER_DURATION = 500

export interface RowDragGatherItem {
	row: ContentCardTableItem
	rect: { left: number; top: number; width: number; height: number }
}

export function useRowDragGather(rows: Ref<ContentCardTableItem[]>, root: Ref<HTMLElement | null>) {
	const items = ref<RowDragGatherItem[]>([])
	const target = ref<Point>({ x: 0, y: 0 })
	const targetOffset = ref<Point>({ x: 0, y: 0 })
	const isGathering = ref(false)
	let cleanupTimer: number | undefined

	function clearCleanupTimer() {
		if (cleanupTimer === undefined) return
		clearTimeout(cleanupTimer)
		cleanupTimer = undefined
	}

	function clear() {
		clearCleanupTimer()
		items.value = []
		targetOffset.value = { x: 0, y: 0 }
		isGathering.value = false
	}

	function updateTarget(pointer: Point) {
		target.value = {
			x: pointer.x + targetOffset.value.x,
			y: pointer.y + targetOffset.value.y,
		}
	}

	function rowElement(rowId: string) {
		const ownerDocument = root.value?.ownerDocument ?? document
		return Array.from(ownerDocument.querySelectorAll<HTMLElement>('[data-content-card-item]')).find(
			(element) => element.dataset.contentCardItem === rowId,
		)
	}

	function start(rowIds: string[], pointer: Point) {
		clear()
		const ownerWindow = root.value?.ownerDocument.defaultView ?? window
		const reduceMotion = ownerWindow.matchMedia('(prefers-reduced-motion: reduce)').matches
		if (rowIds.length < 2 || reduceMotion) return

		const source = rowElement(rowIds[0])
		if (source) {
			const rect = source.getBoundingClientRect()
			targetOffset.value = {
				x: rect.left + rect.width / 2 - pointer.x,
				y: rect.top + rect.height / 2 - pointer.y,
			}
		}

		const rowsById = new Map(rows.value.map((row) => [row.id, row]))
		items.value = rowIds.flatMap((rowId) => {
			const row = rowsById.get(rowId)
			const element = rowElement(rowId)
			if (!row || !element || element.getClientRects().length === 0) return []
			const rect = element.getBoundingClientRect()
			return [
				{ row, rect: { left: rect.left, top: rect.top, width: rect.width, height: rect.height } },
			]
		})
		updateTarget(pointer)
		isGathering.value = items.value.length > 0
	}

	function finish() {
		isGathering.value = false
		clearCleanupTimer()
		cleanupTimer = window.setTimeout(() => {
			items.value = []
			cleanupTimer = undefined
		}, 400)
	}

	onBeforeUnmount(clearCleanupTimer)

	return { items, target, isGathering, start, updateTarget, clear, finish }
}
