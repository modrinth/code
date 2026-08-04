import type { Ref } from 'vue'

import { createContext } from '#ui/providers'

export interface ServerPanelAdmonitionsContext {
	readonly showInstanceInfo: Ref<boolean>
}

export const [injectServerPanelAdmonitionsContext, provideServerPanelAdmonitionsContext] =
	createContext<ServerPanelAdmonitionsContext>('[id].vue', 'serverPanelAdmonitionsContext')
