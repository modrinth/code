import { createContext } from '@modrinth/ui'

export type UnsavedChangesController = {
	hasChanges: () => boolean
	getOriginal: () => Record<string, unknown>
	getModified: () => Record<string, unknown>
	isSaving: () => boolean
	reset: () => void
	save: () => void | Promise<void>
}

export interface AppSettingsModalContext {
	close: () => boolean
	registerUnsavedChangesController: (controller: UnsavedChangesController | null) => void
}

export const [injectAppSettingsModal, provideAppSettingsModal] =
	createContext<AppSettingsModalContext>('AppSettingsModal', 'appSettingsModal')
