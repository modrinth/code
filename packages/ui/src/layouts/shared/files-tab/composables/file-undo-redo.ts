import { ref } from 'vue'

import type { Operation } from '../types'

export function useFileUndoRedo(
	renameItem: (path: string, newName: string) => Promise<void>,
	moveItem: (source: string, destination: string) => Promise<void>,
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
					await moveItem(
						`${lastOperation.destinationPath}/${lastOperation.fileName}`.replace('//', '/'),
						`${lastOperation.sourcePath}/${lastOperation.fileName}`.replace('//', '/'),
					)
					break
				case 'rename':
					await renameItem(
						`${lastOperation.path}/${lastOperation.newName}`.replace('//', '/'),
						lastOperation.oldName,
					)
					break
			}

			redoStack.value.push(lastOperation)
			refresh()
			notify(
				`${lastOperation.type === 'move' ? 'Move' : 'Rename'} undone`,
				`${lastOperation.fileName} has been restored to its original ${lastOperation.type === 'move' ? 'location' : 'name'}`,
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
					await moveItem(
						`${lastOperation.sourcePath}/${lastOperation.fileName}`.replace('//', '/'),
						`${lastOperation.destinationPath}/${lastOperation.fileName}`.replace('//', '/'),
					)
					break
				case 'rename':
					await renameItem(
						`${lastOperation.path}/${lastOperation.oldName}`.replace('//', '/'),
						lastOperation.newName,
					)
					break
			}

			operationHistory.value.push(lastOperation)
			refresh()
			notify(
				`${lastOperation.type === 'move' ? 'Move' : 'Rename'} redone`,
				`${lastOperation.fileName} has been ${lastOperation.type === 'move' ? 'moved' : 'renamed'} again`,
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
