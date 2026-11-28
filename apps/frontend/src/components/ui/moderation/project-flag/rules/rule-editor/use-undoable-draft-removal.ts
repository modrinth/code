import { injectNotificationManager } from '@modrinth/ui'
import { onBeforeUnmount, type Ref, useId, watch } from 'vue'

import { addUndoNotification } from '../undo-notification'

export function useUndoableDraftRemoval<T extends object>(
	items: Ref<T[]>,
	isDisabled: () => boolean | undefined,
	getNotification: (item: T) => { title: string; text?: string },
) {
	const notificationManager = injectNotificationManager()
	const notificationPrefix = `project-flag-draft-removal:${useId()}:`
	let nextRemovalId = 0
	let positions = new WeakMap<T, number>()
	let nextPosition = 0

	function clearNotifications() {
		for (const { id } of [...notificationManager.getNotifications()]) {
			if (typeof id === 'string' && id.startsWith(notificationPrefix)) {
				notificationManager.removeNotification(id)
			}
		}
		positions = new WeakMap()
		nextPosition = 0
	}

	watch([items, isDisabled], clearNotifications, { flush: 'sync' })
	onBeforeUnmount(clearNotifications)

	function rememberPosition(item: T) {
		const position = positions.get(item)
		if (position !== undefined) return position
		const newPosition = nextPosition++
		positions.set(item, newPosition)
		return newPosition
	}

	return (index: number) => {
		if (isDisabled()) return
		const draft = items.value
		const removed = draft[index]
		if (!removed) return
		draft.forEach(rememberPosition)
		const originalPosition = rememberPosition(removed)
		draft.splice(index, 1)
		addUndoNotification(notificationManager, {
			id: `${notificationPrefix}${nextRemovalId++}`,
			...getNotification(removed),
			copyable: false,
			buttons: [
				{
					label: 'Undo',
					action: () => {
						if (isDisabled() || items.value !== draft || draft.includes(removed)) return
						const nextIndex = draft.findIndex(
							(item) => (positions.get(item) ?? Infinity) > originalPosition,
						)
						draft.splice(nextIndex === -1 ? draft.length : nextIndex, 0, removed)
					},
				},
			],
		})
	}
}
