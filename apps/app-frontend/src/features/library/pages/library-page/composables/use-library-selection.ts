import { useEventListener } from '@vueuse/core'
import { computed, type MaybeRefOrGetter, ref, toValue, watch, type WatchSource } from 'vue'

import {
	isUninstalledSharedInstance,
	type LibraryInstance,
} from '@/features/library/library-instance'

import { getLibraryInstanceSelectionKey, type LibraryInstanceSelection } from '../types'

export function useLibrarySelection({
	libraryInstances,
	displayedSelections,
	resetSources,
}: {
	libraryInstances: MaybeRefOrGetter<LibraryInstance[]>
	displayedSelections: MaybeRefOrGetter<LibraryInstanceSelection[]>
	resetSources: WatchSource<unknown>[]
}) {
	const selectedLibraryInstances = ref(new Map<string, LibraryInstanceSelection>())
	const anchorInstance = ref<LibraryInstanceSelection | null>(null)
	const isLibraryInstanceSelectionActive = computed(() => selectedLibraryInstances.value.size > 0)
	const selectedInstanceIds = computed(
		() =>
			new Set(
				[...selectedLibraryInstances.value.values()].map((selection) => selection.instanceId),
			),
	)

	const uninstalledInstanceIds = computed(
		() =>
			new Set(
				toValue(libraryInstances)
					.filter(isUninstalledSharedInstance)
					.map((instance) => instance.id),
			),
	)
	const selectingUninstalledInstances = computed(() => {
		const firstSelection = selectedLibraryInstances.value.values().next().value
		return firstSelection ? uninstalledInstanceIds.value.has(firstSelection.instanceId) : null
	})

	const canSelectLibraryInstance = (instanceId: string) =>
		selectingUninstalledInstances.value === null ||
		selectingUninstalledInstances.value === uninstalledInstanceIds.value.has(instanceId)

	const clearLibraryInstanceSelection = () => {
		selectedLibraryInstances.value = new Map()
	}

	const setSelectedLibraryInstances = (selections: Iterable<LibraryInstanceSelection>) => {
		selectedLibraryInstances.value = new Map(
			[...selections].map((selection) => [getLibraryInstanceSelectionKey(selection), selection]),
		)
	}

	const toggleLibraryInstanceSelection = (selection: LibraryInstanceSelection) => {
		const selectedInstances = new Map(selectedLibraryInstances.value)
		const selectionKey = getLibraryInstanceSelectionKey(selection)

		if (selectedInstances.has(selectionKey)) {
			selectedInstances.delete(selectionKey)
		} else if (canSelectLibraryInstance(selection.instanceId)) {
			selectedInstances.set(selectionKey, selection)
		} else {
			return
		}

		selectedLibraryInstances.value = selectedInstances
	}

	const toggleWithRange = (selection: LibraryInstanceSelection, shiftKey: boolean) => {
		const displayed = toValue(displayedSelections)
		const anchor = anchorInstance.value
		const isSame = (a: LibraryInstanceSelection, b: LibraryInstanceSelection) =>
			a.groupId === b.groupId && a.instanceId === b.instanceId

		if (!shiftKey || !anchor || displayed.length === 0) {
			toggleLibraryInstanceSelection(selection)
			anchorInstance.value = selection
			return
		}

		const anchorIndex = displayed.findIndex((candidate) => isSame(candidate, anchor))
		const targetIndex = displayed.findIndex((candidate) => isSame(candidate, selection))
		if (anchorIndex === -1 || targetIndex === -1) {
			toggleLibraryInstanceSelection(selection)
			return
		}

		const range = displayed.slice(
			Math.min(anchorIndex, targetIndex),
			Math.max(anchorIndex, targetIndex) + 1,
		)
		const nextSelectedInstances = new Map(selectedLibraryInstances.value)
		if (nextSelectedInstances.has(getLibraryInstanceSelectionKey(selection))) {
			for (const candidate of range) {
				nextSelectedInstances.delete(getLibraryInstanceSelectionKey(candidate))
			}
		} else {
			for (const candidate of range) {
				if (!canSelectLibraryInstance(candidate.instanceId)) continue
				nextSelectedInstances.set(getLibraryInstanceSelectionKey(candidate), candidate)
			}
		}

		setSelectedLibraryInstances(nextSelectedInstances.values())
		anchorInstance.value = null
	}

	watch(resetSources, clearLibraryInstanceSelection)

	watch(
		() => toValue(libraryInstances),
		(currentInstances) => {
			if (selectedLibraryInstances.value.size === 0) return

			const instanceIds = new Set(currentInstances.map((instance) => instance.id))
			const selections = [...selectedLibraryInstances.value.values()]
			if (selections.every((selection) => instanceIds.has(selection.instanceId))) return

			setSelectedLibraryInstances(
				selections.filter((selection) => instanceIds.has(selection.instanceId)),
			)
		},
	)

	watch(selectedLibraryInstances, (selectedInstances) => {
		if (
			anchorInstance.value &&
			!selectedInstances.has(getLibraryInstanceSelectionKey(anchorInstance.value))
		) {
			anchorInstance.value = null
		}
	})

	useEventListener(window, 'keydown', (event) => {
		if (
			event.key === 'Escape' &&
			!event.defaultPrevented &&
			isLibraryInstanceSelectionActive.value
		) {
			clearLibraryInstanceSelection()
		}
	})

	return {
		selectedLibraryInstances,
		selectedInstanceIds,
		anchorInstance,
		isLibraryInstanceSelectionActive,
		canSelectLibraryInstance,
		clearLibraryInstanceSelection,
		setSelectedLibraryInstances,
		toggleLibraryInstanceSelection,
		toggleWithRange,
	}
}
