import { onBeforeUnmount, type Ref, ref } from 'vue'

import type { InstanceScreenshot } from '@/helpers/instance'

type Point = {
	x: number
	y: number
}

export type ActiveScreenshotDrag = {
	primarySelectionKey: string
	selectionKeys: string[]
}

export const gatherDuration = 500

export type ScreenshotDragGatherItem = {
	screenshot: InstanceScreenshot
	selectionKey: string
	rect: {
		left: number
		top: number
		width: number
		height: number
	}
}

function getSelectionKey(screenshot: InstanceScreenshot) {
	return JSON.stringify([screenshot.instance_id, screenshot.file_name])
}

export function useScreenshotDragGather(screenshots: Ref<InstanceScreenshot[]>) {
	const items = ref<ScreenshotDragGatherItem[]>([])
	const target = ref<Point>({ x: 0, y: 0 })
	const targetOffset = ref<Point>({ x: 0, y: 0 })
	const isGathering = ref(false)
	let cleanupTimer: number | undefined

	const clearCleanupTimer = () => {
		if (cleanupTimer !== undefined) {
			clearTimeout(cleanupTimer)
			cleanupTimer = undefined
		}
	}

	const clear = () => {
		clearCleanupTimer()
		items.value = []
		targetOffset.value = { x: 0, y: 0 }
		isGathering.value = false
	}

	const updateTarget = (pointer: Point) => {
		target.value = {
			x: pointer.x + targetOffset.value.x,
			y: pointer.y + targetOffset.value.y,
		}
	}

	const start = (drag: ActiveScreenshotDrag | null, pointer: Point) => {
		clear()

		const reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches
		if (!drag || drag.selectionKeys.length < 2 || reduceMotion) return

		const screenshotCards = Array.from(
			document.querySelectorAll<HTMLElement>('[data-screenshot-card]'),
		)
		const sourceCard = screenshotCards.find(
			(card) => card.dataset.selectionKey === drag.primarySelectionKey,
		)

		if (sourceCard) {
			const sourceRect = sourceCard.getBoundingClientRect()
			targetOffset.value = {
				x: sourceRect.left + sourceRect.width / 2 - pointer.x,
				y: sourceRect.top + sourceRect.height / 2 - pointer.y,
			}
		}

		const screenshotsByKey = new Map(
			screenshots.value.map((screenshot) => [getSelectionKey(screenshot), screenshot]),
		)
		items.value = drag.selectionKeys.flatMap((selectionKey) => {
			const screenshot = screenshotsByKey.get(selectionKey)
			const card = screenshotCards.find(
				(candidate) =>
					candidate.dataset.selectionKey === selectionKey && candidate.getClientRects().length > 0,
			)
			if (!screenshot || !card) return []

			const rect = card.getBoundingClientRect()
			return [
				{
					screenshot,
					selectionKey,
					rect: {
						left: rect.left,
						top: rect.top,
						width: rect.width,
						height: rect.height,
					},
				},
			]
		})
		updateTarget(pointer)
		isGathering.value = items.value.length > 0
	}

	const finish = () => {
		isGathering.value = false
		clearCleanupTimer()
		cleanupTimer = window.setTimeout(() => {
			items.value = []
			cleanupTimer = undefined
		}, 400)
	}

	onBeforeUnmount(clearCleanupTimer)

	return {
		items,
		target,
		isGathering,
		start,
		updateTarget,
		clear,
		finish,
	}
}
