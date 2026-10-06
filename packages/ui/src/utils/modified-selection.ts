export interface ModifiedSelection<Id> {
	selectedIds: Set<Id>
	anchorId: Id | undefined
}

export function getModifiedSelection<Id>(
	orderedIds: readonly Id[],
	selectedIds: ReadonlySet<Id>,
	clickedId: Id,
	anchorId: Id | undefined,
	modifiers: Pick<MouseEvent, 'shiftKey' | 'ctrlKey' | 'metaKey'>,
): ModifiedSelection<Id> | undefined {
	const clickedIndex = orderedIds.indexOf(clickedId)
	if (clickedIndex === -1 || (!modifiers.shiftKey && !modifiers.ctrlKey && !modifiers.metaKey)) {
		return undefined
	}

	const nextSelectedIds = new Set(selectedIds)
	if (modifiers.shiftKey) {
		const anchorIndex = anchorId === undefined ? -1 : orderedIds.indexOf(anchorId)
		const rangeAnchorIndex = anchorIndex === -1 ? clickedIndex : anchorIndex
		const start = Math.min(rangeAnchorIndex, clickedIndex)
		const end = Math.max(rangeAnchorIndex, clickedIndex)
		const shouldSelect = !selectedIds.has(clickedId)

		for (let index = start; index <= end; index++) {
			if (shouldSelect) {
				nextSelectedIds.add(orderedIds[index])
			} else {
				nextSelectedIds.delete(orderedIds[index])
			}
		}

		return { selectedIds: nextSelectedIds, anchorId: anchorIndex === -1 ? clickedId : undefined }
	}

	if (nextSelectedIds.has(clickedId)) {
		nextSelectedIds.delete(clickedId)
	} else {
		nextSelectedIds.add(clickedId)
	}

	return { selectedIds: nextSelectedIds, anchorId: clickedId }
}
