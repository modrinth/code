import { onBeforeUnmount, type Ref, ref } from 'vue'

import type { ActiveInstanceGroupDrag } from '@/components/ui/library/use-library'
import type { GameInstance } from '@/helpers/types'

type Point = {
	x: number
	y: number
}

type InstanceDragData = {
	instanceId: string
	fromGroup: string
}

export type InstanceDragGatherItem = {
	instance: GameInstance
	rect: {
		left: number
		top: number
		width: number
		height: number
	}
}

export function useInstanceDragGather(instances: Ref<GameInstance[]>) {
	const items = ref<InstanceDragGatherItem[]>([])
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

	const start = (
		drag: ActiveInstanceGroupDrag | null,
		source: InstanceDragData,
		pointer: Point,
	) => {
		clear()

		const reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches
		if (!drag || drag.instances.length < 2 || reduceMotion) return

		const instanceCards = Array.from(
			document.querySelectorAll<HTMLElement>('[data-library-instance-card]'),
		)
		const sourceCard = instanceCards.find(
			(card) =>
				card.dataset.instanceId === source.instanceId &&
				card.dataset.instanceGroup === source.fromGroup,
		)

		if (sourceCard) {
			const sourceRect = sourceCard.getBoundingClientRect()
			targetOffset.value = {
				x: sourceRect.left + sourceRect.width / 2 - pointer.x,
				y: sourceRect.top + sourceRect.height / 2 - pointer.y,
			}
		}

		items.value = drag.instances.flatMap(({ instanceId, groupId }) => {
			const instance = instances.value.find((candidate) => candidate.id === instanceId)
			const matchingCards = instanceCards.filter(
				(card) => card.dataset.instanceId === instanceId && card.getClientRects().length > 0,
			)
			const card =
				matchingCards.find((candidate) => candidate.dataset.instanceGroup === groupId) ??
				matchingCards[0]
			if (!instance || !card) return []

			const rect = card.getBoundingClientRect()
			return [
				{
					instance,
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
