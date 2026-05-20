import type { ServerSettingsTabId } from '#ui/layouts/shared/server-settings'

import { createContext } from './create-context'

export interface BrowseServerContentArgs {
	serverId: string
	worldId: string | null
	type: 'mod' | 'plugin' | 'datapack'
}

export interface ServerSettingsModalContext {
	openServerSettings: (options?: { tabId?: ServerSettingsTabId }) => void
	browseServerContent?: (args: BrowseServerContentArgs) => void | Promise<void>
}

export const [injectServerSettingsModal, provideServerSettingsModal] =
	createContext<ServerSettingsModalContext>('root.vue', 'serverSettingsModalContext')
