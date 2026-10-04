import { ref } from 'vue'

import type { FileInfo } from '#ui/layouts/shared/files-tab/providers/file-manager.ts'
import { parentInfoFrom } from '#ui/layouts/shared/files-tab/utils.ts'

import type { Operation } from '../types'

export function useFileUndoRedo(
	renameItem: (file: FileInfo, newName: string) => Promise<FileInfo | null>,
	moveItem: (source: FileInfo, destination: string) => Promise<FileInfo | null>,
	refresh: () => void,
	notify: (title: string, text: string, type: 'success' | 'error') => void,
) {
	const operationHistory = ref<Operation[]>([])
	const redoStack = ref<Operation[]>([])

	function recordOperation(op: Operation) {
		redoStack.value = []
		operationHistory.value.push(op)
	}

	async function undo() {
		const lastOperation = operationHistory.value.pop()
		if (!lastOperation) return

		try {
			switch (lastOperation.type) {
				case 'move':
					await moveItem(lastOperation.newFile, parentInfoFrom(lastOperation.prevFile).path)
					break
				case 'rename':
					await renameItem(lastOperation.newFile, lastOperation.prevFile.name)
					break
			}

			redoStack.value.push(lastOperation)
			refresh()
			notify(
				`${lastOperation.type === 'move' ? 'Move' : 'Rename'} undone`,
				`${lastOperation.prevFile.name} has been restored to its original ${lastOperation.type === 'move' ? 'location' : 'name'}`,
				'success',
			)
		} catch {
			notify('Undo failed', `Failed to undo the last ${lastOperation.type} operation`, 'error')
		}
	}

	async function redo() {
		const lastOperation = redoStack.value.pop()
		if (!lastOperation) return

		try {
			switch (lastOperation.type) {
				case 'move':
					await moveItem(lastOperation.prevFile, parentInfoFrom(lastOperation.newFile).path)
					break
				case 'rename':
					await renameItem(lastOperation.prevFile, lastOperation.newFile.name)
					break
			}

			operationHistory.value.push(lastOperation)
			refresh()
			notify(
				`${lastOperation.type === 'move' ? 'Move' : 'Rename'} redone`,
				`${lastOperation.prevFile.name} has been ${lastOperation.type === 'move' ? 'moved' : 'renamed'} again`,
				'success',
			)
		} catch {
			notify('Redo failed', `Failed to redo the last ${lastOperation.type} operation`, 'error')
		}
	}

	function onKeydown(e: KeyboardEvent) {
		if ((e.ctrlKey || e.metaKey) && !e.shiftKey && e.key === 'z') {
			e.preventDefault()
			undo()
		}
		if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'z') {
			e.preventDefault()
			redo()
		}
	}

	return {
		operationHistory,
		redoStack,
		recordOperation,
		undo,
		redo,
		onKeydown,
	}
}
