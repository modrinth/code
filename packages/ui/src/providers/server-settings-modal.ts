import type { ServerSettingsTabId } from '#ui/layouts/shared/server-settings'

import { createContext } from './create-context'

export interface ServerSettingsModalContext {
	openServerSettings: (options?: { tabId?: ServerSettingsTabId }) => void
}

export const [injectServerSettingsModal, provideServerSettingsModal] =
	createContext<ServerSettingsModalContext>('root.vue', 'serverSettingsModalContext')
