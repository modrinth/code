import { createContext } from '@modrinth/ui'

export interface AppSettingsModalContext {
	close: () => boolean
}

export const [injectAppSettingsModal, provideAppSettingsModal] =
	createContext<AppSettingsModalContext>('AppSettingsModal', 'appSettingsModal')
