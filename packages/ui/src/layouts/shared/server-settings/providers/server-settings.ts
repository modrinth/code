import type { Ref } from 'vue'

import { createContext } from '#ui/providers/create-context'

export interface ServerSettingsBrowseModpacksArgs {
	serverId: string
	worldId: string | null
	from: 'reset-server'
}

export interface ServerSettingsContext {
	siteUrl: Ref<string>
	isApp: Ref<boolean>
	currentUserId: Ref<string | null>
	currentUserRole: Ref<string | null>
	browseModpacks: (args: ServerSettingsBrowseModpacksArgs) => void | Promise<void>
	closeModal?: () => void
	saveBanner?: {
		target: Ref<HTMLElement | null>
		shown: Ref<boolean>
	}
}

export const [injectServerSettings, provideServerSettings] = createContext<ServerSettingsContext>(
	'ServerSettings',
	'serverSettingsContext',
)
