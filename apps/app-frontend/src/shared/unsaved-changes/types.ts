export type UnsavedChangesController = {
	hasChanges: () => boolean
	getOriginal: () => Record<string, unknown>
	getModified: () => Record<string, unknown>
	isSaving: () => boolean
	reset: () => void
	save: () => void | Promise<void>
}
