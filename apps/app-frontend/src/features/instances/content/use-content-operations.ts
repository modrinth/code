import type { ContentItem } from '@modrinth/ui'
import { computed, ref } from 'vue'

import { contentKey } from './content-items'

export interface ContentOperation {
	keys: string[]
	originalFileName: string
	blocksList: boolean
}

export type ContentBusyListener = (
	fileName: string,
	state: { installing: boolean; disabled: boolean },
) => void

export function useContentOperations(onBusyChange: ContentBusyListener) {
	const activeKeys = ref(new Set<string>())
	const installingKeys = ref(new Set<string>())
	const blockingCount = ref(0)

	function keysOf(item: ContentItem) {
		return [contentKey(item), item.file_path, item.file_name].filter((key): key is string => !!key)
	}

	function has(item: ContentItem) {
		return keysOf(item).some((key) => activeKeys.value.has(key))
	}

	function isInstalling(item: ContentItem) {
		return keysOf(item).some((key) => installingKeys.value.has(key))
	}

	function begin(item: ContentItem, blocksList = true): ContentOperation | null {
		if (has(item)) return null
		const keys = keysOf(item)
		activeKeys.value = new Set([...activeKeys.value, ...keys])
		if (blocksList) {
			blockingCount.value++
			installingKeys.value = new Set([...installingKeys.value, ...keys])
		}
		onBusyChange(item.file_name, { installing: blocksList, disabled: true })
		return { keys, originalFileName: item.file_name, blocksList }
	}

	function finish(operation: ContentOperation) {
		const nextActive = new Set(activeKeys.value)
		const nextInstalling = new Set(installingKeys.value)
		for (const key of operation.keys) {
			nextActive.delete(key)
			nextInstalling.delete(key)
		}
		activeKeys.value = nextActive
		installingKeys.value = nextInstalling
		if (operation.blocksList) blockingCount.value = Math.max(0, blockingCount.value - 1)
		onBusyChange(operation.originalFileName, { installing: false, disabled: false })
	}

	return {
		begin,
		finish,
		has,
		isInstalling,
		hasActive: computed(() => activeKeys.value.size > 0),
		isBlocking: computed(() => blockingCount.value > 0),
	}
}

export type ContentOperations = ReturnType<typeof useContentOperations>
