import type {
	ServerInstanceSettingsTabId,
	ServerSettingsTabId,
} from '#ui/layouts/shared/server-settings'

import { createContext } from './create-context'

export interface BrowseServerContentArgs {
	serverId: string
	worldId: string | null
	type: 'mod' | 'plugin' | 'datapack'
}

export interface ServerSettingsModalContext {
	openServerSettings: (options?: { tabId?: ServerSettingsTabId }) => void
	openServerInstanceSettings: (options?: {
		tabId?: ServerInstanceSettingsTabId
		worldId?: string | null
	}) => void
	browseServerContent?: (args: BrowseServerContentArgs) => void | Promise<void>
}

export const [injectServerSettingsModal, provideServerSettingsModal] =
	createContext<ServerSettingsModalContext>('root.vue', 'serverSettingsModalContext')
