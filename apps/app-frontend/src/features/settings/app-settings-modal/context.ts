import { createContext } from '@modrinth/ui'

import type { UnsavedChangesController } from '@/shared/unsaved-changes/types'

export interface AppSettingsModalContext {
	close: () => boolean
	registerUnsavedChangesController: (controller: UnsavedChangesController | null) => void
}

export const [injectAppSettingsModal, provideAppSettingsModal] =
	createContext<AppSettingsModalContext>('AppSettingsModal', 'appSettingsModal')
