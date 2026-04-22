import type { Ref } from 'vue'

import { createContext } from '.'

export interface PageContext {
	// pages may render sidebar content in #sidebar-teleport-target instead of in the main layout when true
	hierarchicalSidebarAvailable: Ref<boolean>
	showAds: Ref<boolean>
	featureFlags?: {
		serverRamAsBytesAlwaysOn?: Ref<boolean>
	}
	openExternalUrl: (url: string) => void
}

export const [injectPageContext, providePageContext] = createContext<PageContext>(
	'root',
	'pageContext',
)
